//! `creator-gpu` — GPU surface/context management per backend.
//!
//! The engine renders the same [`creator_render::RenderTree`] to any surface;
//! this crate owns the **backend abstraction** over `{window surface, offscreen
//! image}` on Metal/Vulkan/D3D/GL, plus the always-available CPU fallback
//! (PLAN.md §6/§10).
//!
//! ## Status
//!
//! The real GPU backends are built on **`skia-safe`** with the `metal`/`vulkan`
//! feature flags, which pull a native Skia build/binary that is provisioned
//! separately (not available in a headless CI image). They therefore live behind
//! the `skia` cargo feature and are *not* in the default build. What ships in the
//! default build is the abstraction (this module) and a CPU-backed
//! implementation that bridges to `creator-render` — enough for the CLI's
//! `--backend cpu` path and for code that wants to be backend-agnostic.
//!
//! See [`Backend`] for what each phase wires up.

use creator_render::{render, CpuTarget, RenderTree};

/// Which GPU/CPU backend a context targets (PLAN.md §1 "GPU backends").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    /// CPU raster — always available (CI / serverless, no GPU).
    Cpu,
    /// Vulkan (Linux/Windows; offscreen on headless servers with a GPU).
    Vulkan,
    /// Metal (macOS only).
    Metal,
    /// Direct3D (Windows).
    D3D,
    /// OpenGL (portable fallback).
    Gl,
}

impl BackendKind {
    pub fn is_gpu(self) -> bool {
        !matches!(self, BackendKind::Cpu)
    }
}

/// Errors creating contexts/surfaces.
#[derive(Debug)]
pub enum GpuError {
    /// The requested backend exists in the design but isn't compiled into this
    /// build (e.g. Vulkan without the `skia` feature).
    NotCompiled(BackendKind),
    /// Backend present but initialization failed.
    Init(String),
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuError::NotCompiled(k) => write!(
                f,
                "the {k:?} backend is not compiled into this build; rebuild `creator-gpu` with the \
                 `skia` feature (and a provisioned Skia) to enable it"
            ),
            GpuError::Init(m) => write!(f, "backend init failed: {m}"),
        }
    }
}
impl std::error::Error for GpuError {}

/// A drawable surface — a window swapchain image or an offscreen target. The
/// engine renders into it; for export/headless it is read back to a
/// [`CpuTarget`].
pub trait Surface {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    /// Render a resolved tree into this surface.
    fn draw(&mut self, tree: &RenderTree);
    /// Copy the rendered pixels into a CPU target (premultiplied linear RGBA).
    fn read_back(&self) -> CpuTarget;
}

/// A backend: makes surfaces. A window-surface constructor is added with the
/// viewport-bridge work (PLAN.md §7) and is intentionally absent here.
pub trait Backend {
    fn kind(&self) -> BackendKind;
    /// Allocate an offscreen surface (the easy, not-real-time path — PLAN.md §7
    /// "decouple interactive viewport from final render").
    fn create_offscreen(&self, width: u32, height: u32) -> Result<Box<dyn Surface>, GpuError>;
}

/// Select a backend by kind. Only [`BackendKind::Cpu`] is available without the
/// `skia` feature; the GPU kinds return [`GpuError::NotCompiled`] so callers can
/// surface a precise message (as the CLI does).
pub fn select(kind: BackendKind) -> Result<Box<dyn Backend>, GpuError> {
    match kind {
        BackendKind::Cpu => Ok(Box::new(CpuBackend)),
        #[cfg(feature = "skia")]
        other => skia_backend::select(other),
        #[cfg(not(feature = "skia"))]
        other => Err(GpuError::NotCompiled(other)),
    }
}

/// The always-available CPU backend, bridging to `creator-render`.
pub struct CpuBackend;

impl Backend for CpuBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::Cpu
    }
    fn create_offscreen(&self, width: u32, height: u32) -> Result<Box<dyn Surface>, GpuError> {
        Ok(Box::new(CpuSurface(CpuTarget::new(width, height))))
    }
}

struct CpuSurface(CpuTarget);

impl Surface for CpuSurface {
    fn width(&self) -> u32 {
        use creator_render::RenderTarget;
        self.0.width()
    }
    fn height(&self) -> u32 {
        use creator_render::RenderTarget;
        self.0.height()
    }
    fn draw(&mut self, tree: &RenderTree) {
        render(tree, &mut self.0);
    }
    fn read_back(&self) -> CpuTarget {
        self.0.clone()
    }
}

/// Skia-on-Vulkan offscreen backend. Compiled only with the `skia` feature;
/// see `skia_backend.rs` for the ash + DirectContext + RenderTree translation.
#[cfg(feature = "skia")]
mod skia_backend;

#[cfg(test)]
mod tests {
    use super::*;
    use creator_render::{Color, RenderTarget};

    #[test]
    fn cpu_backend_round_trips_a_render() {
        let backend = select(BackendKind::Cpu).unwrap();
        assert_eq!(backend.kind(), BackendKind::Cpu);
        let mut surface = backend.create_offscreen(8, 8).unwrap();
        let tree = RenderTree::new(8, 8, Color::from_srgb8(20, 20, 20, 255));
        surface.draw(&tree);
        let cpu = surface.read_back();
        assert_eq!((cpu.width(), cpu.height()), (8, 8));
    }

    #[test]
    fn gpu_backends_report_not_compiled_by_default() {
        for k in [BackendKind::Vulkan, BackendKind::Metal] {
            assert!(matches!(select(k), Err(GpuError::NotCompiled(_))));
        }
    }
}
