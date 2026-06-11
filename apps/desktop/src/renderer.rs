//! The viewport render thread.
//!
//! Skia's `DirectContext` is single-threaded (not `Send`), so the GPU backend
//! lives on one dedicated thread for the app's lifetime. Tauri commands
//! evaluate the scene to a [`RenderTree`] (pure and `Send`) and ship it over a
//! channel; the thread rasterizes — Vulkan when available, CPU otherwise — and
//! replies with the pixels. This is also the seam where a future tier-1/2
//! viewport presents to a native surface instead of replying with a buffer
//! (PLAN.md §7).

use creator_render::{render as cpu_render, CpuTarget, RenderTree};
use std::sync::mpsc;

pub struct Renderer {
    tx: Option<mpsc::Sender<Job>>,
    handle: Option<std::thread::JoinHandle<()>>,
    /// Which rasterizer the thread settled on ("vulkan" or "cpu").
    pub backend: &'static str,
}

struct Job {
    tree: RenderTree,
    reply: mpsc::SyncSender<CpuTarget>,
}

impl Renderer {
    /// Spawn the render thread. Tries the Vulkan backend first; falls back to
    /// the CPU rasterizer (same output, slower) if unavailable.
    pub fn start() -> Renderer {
        let (tx, rx) = mpsc::channel::<Job>();
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let handle = std::thread::Builder::new()
            .name("creator-render".into())
            .spawn(move || render_loop(rx, ready_tx))
            .expect("spawning render thread");
        let backend = ready_rx.recv().unwrap_or("cpu");
        Renderer { tx: Some(tx), handle: Some(handle), backend }
    }

    /// Render a resolved tree, blocking until the frame is ready. `None` only
    /// if the render thread died.
    pub fn render(&self, tree: RenderTree) -> Option<CpuTarget> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.tx.as_ref()?.send(Job { tree, reply: reply_tx }).ok()?;
        reply_rx.recv().ok()
    }
}

impl Drop for Renderer {
    /// Deterministic shutdown: close the channel so the loop exits, then JOIN
    /// so the Vulkan/Skia teardown completes before the process can exit
    /// (exiting mid-teardown aborts inside the driver).
    fn drop(&mut self) {
        drop(self.tx.take());
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn render_loop(rx: mpsc::Receiver<Job>, ready: mpsc::SyncSender<&'static str>) {
    #[cfg(feature = "gpu")]
    let mut gpu = GpuState::create();
    #[cfg(feature = "gpu")]
    let _ = ready.send(if gpu.is_some() { "vulkan" } else { "cpu" });
    #[cfg(not(feature = "gpu"))]
    let _ = ready.send("cpu");

    while let Ok(job) = rx.recv() {
        #[cfg(feature = "gpu")]
        let target = match gpu.as_mut().and_then(|g| g.render(&job.tree)) {
            Some(t) => t,
            None => render_cpu(&job.tree),
        };
        #[cfg(not(feature = "gpu"))]
        let target = render_cpu(&job.tree);

        // Receiver gone (command timed out / app closing) is fine to ignore.
        let _ = job.reply.send(target);
    }
}

fn render_cpu(tree: &RenderTree) -> CpuTarget {
    let mut target = CpuTarget::new(tree.width, tree.height);
    cpu_render(tree, &mut target);
    target
}

#[cfg(test)]
mod tests {
    use super::*;
    use creator_render::{
        Color, NodeContent, RenderNode, RenderTarget as _, ResolvedGeometry, ResolvedPaint,
        ResolvedShape,
    };
    use glam::{Affine2, Vec2};

    #[test]
    fn render_thread_round_trips_a_frame() {
        // Exercises the full scrub plumbing: channel -> (Vulkan or CPU
        // fallback) rasterize -> readback. On a Vulkan machine this runs the
        // GPU path.
        let renderer = Renderer::start();
        let mut tree = RenderTree::new(16, 16, Color::TRANSPARENT);
        let shape = ResolvedShape {
            geometry: ResolvedGeometry::Rect { size: Vec2::new(10.0, 10.0), corner_radius: 0.0 },
            fill: Some(ResolvedPaint::Solid(Color::WHITE)),
            stroke: None,
        };
        let node = RenderNode::leaf(NodeContent::Shape(shape))
            .with_transform(Affine2::from_translation(Vec2::new(8.0, 8.0)));
        tree.root = RenderNode::group(vec![node]);

        let target = renderer.render(tree).expect("render thread alive");
        assert_eq!((target.width(), target.height()), (16, 16));
        let center = target.pixel(8, 8);
        assert!(center[3] > 0.9, "center should be covered (backend: {})", renderer.backend);
        let corner = target.pixel(0, 0);
        assert!(corner[3] < 0.05, "corner should be empty");
    }
}

#[cfg(feature = "gpu")]
use gpu_state::GpuState;

#[cfg(feature = "gpu")]
mod gpu_state {
    use super::*;
    use creator_gpu::{select, Backend, BackendKind, Surface};

    /// The Vulkan backend plus its current offscreen surface, recreated when
    /// the requested frame size changes (comp switches).
    ///
    /// Field order matters: fields drop in declaration order, and the surface
    /// (which holds Skia GPU resources) must be released BEFORE the backend
    /// tears down the Vulkan device — `surface` first.
    pub struct GpuState {
        surface: Option<(Box<dyn Surface>, (u32, u32))>,
        backend: Box<dyn Backend>,
    }

    impl GpuState {
        pub fn create() -> Option<GpuState> {
            match select(BackendKind::Vulkan) {
                Ok(backend) => {
                    eprintln!("[creator] viewport renderer: Skia/Vulkan");
                    Some(GpuState { surface: None, backend })
                }
                Err(e) => {
                    eprintln!("[creator] Vulkan unavailable ({e}); viewport renderer: CPU");
                    None
                }
            }
        }

        pub fn render(&mut self, tree: &RenderTree) -> Option<CpuTarget> {
            let size = (tree.width, tree.height);
            if self.surface.as_ref().map(|(_, s)| *s) != Some(size) {
                let surface = self.backend.create_offscreen(size.0, size.1).ok()?;
                self.surface = Some((surface, size));
            }
            let (surface, _) = self.surface.as_mut()?;
            surface.draw(tree);
            Some(surface.read_back())
        }
    }
}
