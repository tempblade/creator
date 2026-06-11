//! `creator` — headless renderer.
//!
//! Loads a project, evaluates each requested frame, rasterizes it with the CPU
//! backend, and encodes a PNG/EXR sequence. Because this drives the *same*
//! `creator-engine` as the editor, headless output matches the editor preview
//! (PLAN.md §10). GPU backends (Vulkan/Metal) are provided by `creator-gpu`
//! (Skia) and are not part of this build configuration.

mod sample;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use creator_export::{frame_filename, write_frame, FrameFormat};
use creator_model::{CompId, Project};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "creator", version, about = "Headless motion-design renderer")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Render a project to an image sequence.
    Render(RenderArgs),
    /// Write an example project to a file.
    Sample {
        /// Output path for the example project JSON.
        out: PathBuf,
    },
    /// Print information about a project.
    Info {
        /// Project file (.ctor / .json).
        project: PathBuf,
    },
}

#[derive(Parser)]
struct RenderArgs {
    /// Project file (.ctor / .json).
    project: PathBuf,
    /// Frame range "A-B" (inclusive). Defaults to the whole composition.
    #[arg(long)]
    frames: Option<String>,
    /// Output directory for the frame sequence.
    #[arg(long, default_value = "frames")]
    out: PathBuf,
    /// Output image format.
    #[arg(long, value_enum, default_value_t = FormatArg::Png)]
    format: FormatArg,
    /// Render backend.
    #[arg(long, value_enum, default_value_t = BackendArg::Cpu)]
    backend: BackendArg,
    /// Worker thread count (defaults to all cores).
    #[arg(long)]
    threads: Option<usize>,
    /// Composition name to render (defaults to the project's root comp).
    #[arg(long)]
    comp: Option<String>,
}

#[derive(Clone, Copy, ValueEnum)]
enum FormatArg {
    Png,
    Exr,
}
impl From<FormatArg> for FrameFormat {
    fn from(f: FormatArg) -> Self {
        match f {
            FormatArg::Png => FrameFormat::Png,
            FormatArg::Exr => FrameFormat::Exr,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum BackendArg {
    /// CPU raster — always available (CI / serverless).
    Cpu,
    /// Offscreen Vulkan (requires the Skia-backed `creator-gpu`).
    Vulkan,
    /// Metal, macOS only (requires the Skia-backed `creator-gpu`).
    Metal,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Render(args) => render(args),
        Cmd::Sample { out } => write_sample(&out),
        Cmd::Info { project } => info(&project),
    }
}

fn write_sample(out: &Path) -> Result<()> {
    let project = sample::build_sample_project();
    let json = project.to_json().context("serializing sample project")?;
    std::fs::write(out, json).with_context(|| format!("writing {}", out.display()))?;
    println!("Wrote example project to {}", out.display());
    Ok(())
}

fn load_project(path: &Path) -> Result<Project> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    Project::from_json(&text).with_context(|| format!("parsing {}", path.display()))
}

fn resolve_comp(project: &Project, name: Option<&str>) -> Result<CompId> {
    match name {
        Some(n) => project
            .compositions
            .iter()
            .find(|(_, c)| c.name == n)
            .map(|(id, _)| id)
            .with_context(|| format!("no composition named \"{n}\"")),
        None => project.root.context("project has no root composition"),
    }
}

fn info(path: &Path) -> Result<()> {
    let project = load_project(path)?;
    println!("Project: {}  (format v{})", project.name, project.version);
    println!("Compositions: {}", project.compositions.len());
    for (id, c) in project.compositions.iter() {
        let root = if project.root == Some(id) { " [root]" } else { "" };
        println!(
            "  - {}{}: {}x{} @ {}fps, {:.2}s ({} frames), {} layers",
            c.name,
            root,
            c.width,
            c.height,
            c.frame_rate,
            c.duration,
            c.frame_count(),
            c.layers.len()
        );
    }
    Ok(())
}

fn render(args: RenderArgs) -> Result<()> {
    match args.backend {
        BackendArg::Cpu => {}
        BackendArg::Vulkan => {
            #[cfg(not(feature = "gpu"))]
            bail!(
                "this binary was built without the `gpu` feature; rebuild with \
                 `cargo build -p creator-cli --features gpu` or use `--backend cpu`."
            );
        }
        BackendArg::Metal => bail!("the metal backend is macOS-only; use cpu or vulkan."),
    }

    let project = load_project(&args.project)?;
    let comp_id = resolve_comp(&project, args.comp.as_deref())?;
    let comp = project.composition(comp_id).context("composition vanished")?;

    let last_frame = comp.frame_count().saturating_sub(1);
    let (start, end) = match &args.frames {
        Some(spec) => parse_frame_range(spec)?,
        None => (0, last_frame),
    };
    if start > end {
        bail!("invalid frame range: {start}-{end}");
    }

    std::fs::create_dir_all(&args.out).with_context(|| format!("creating {}", args.out.display()))?;

    let format: FrameFormat = args.format.into();
    let frames: Vec<u64> = (start..=end).collect();
    let total = frames.len();
    println!(
        "Rendering \"{}\" ({}x{}) frames {start}-{end} ({total}) -> {} [{}]",
        comp.name,
        comp.width,
        comp.height,
        args.out.display(),
        format.extension()
    );

    let started = Instant::now();

    if matches!(args.backend, BackendArg::Vulkan) {
        #[cfg(feature = "gpu")]
        render_frames_gpu(&project, comp_id, &frames, &args.out, format)?;
        #[cfg(not(feature = "gpu"))]
        unreachable!("guarded above");
    } else {
        let pool = build_pool(args.threads)?;
        let done = AtomicUsize::new(0);
        let out_dir = args.out.clone();
        let result: Result<()> = pool.install(|| {
            frames.par_iter().try_for_each(|&frame| -> Result<()> {
                let target = creator_engine::render_frame(&project, comp_id, frame);
                let name = frame_filename("frame", frame, format);
                write_frame(&target, out_dir.join(&name), format)
                    .with_context(|| format!("writing {name}"))?;
                let n = done.fetch_add(1, Ordering::Relaxed) + 1;
                if n % 25 == 0 || n == total {
                    println!("  {n}/{total}");
                }
                Ok(())
            })
        });
        result?;
    }

    println!(
        "Done: {total} frames in {:.2}s ({:.1} fps)",
        started.elapsed().as_secs_f64(),
        total as f64 / started.elapsed().as_secs_f64().max(1e-9)
    );
    Ok(())
}

/// Render frames on the Skia/Vulkan offscreen backend (sequential — one shared
/// GPU surface; the GPU itself provides the parallelism). Motion blur averages
/// the exact same shutter instants as the CPU path
/// (`creator_engine::shutter_sample_times`), accumulated in fixed order.
#[cfg(feature = "gpu")]
fn render_frames_gpu(
    project: &creator_model::Project,
    comp_id: creator_model::CompId,
    frames: &[u64],
    out: &std::path::Path,
    format: FrameFormat,
) -> Result<()> {
    use creator_render::{CpuTarget, RenderTarget as _};

    let comp = project.composition(comp_id).context("composition vanished")?;
    let backend = creator_gpu::select(creator_gpu::BackendKind::Vulkan)
        .map_err(anyhow::Error::new)?;
    let mut surface = backend
        .create_offscreen(comp.width, comp.height)
        .map_err(anyhow::Error::new)?;
    println!("  (Vulkan offscreen: {}x{})", comp.width, comp.height);

    let total = frames.len();
    for (i, &frame) in frames.iter().enumerate() {
        let time = comp.frame_to_time(frame);
        let target = match comp.motion_blur {
            Some(mb) if mb.samples >= 2 && mb.shutter_angle > 0.0 && comp.frame_rate > 0.0 => {
                let times = creator_engine::shutter_sample_times(time, comp.frame_rate, mb);
                let count = (comp.width as usize) * (comp.height as usize);
                let mut acc = vec![[0.0f32; 4]; count];
                for &st in &times {
                    let tree = creator_engine::eval(project, comp_id, st);
                    surface.draw(&tree);
                    let sample = surface.read_back();
                    for (a, p) in acc.iter_mut().zip(sample.pixels()) {
                        a[0] += p[0];
                        a[1] += p[1];
                        a[2] += p[2];
                        a[3] += p[3];
                    }
                }
                let inv = 1.0 / times.len() as f32;
                for a in &mut acc {
                    a[0] *= inv;
                    a[1] *= inv;
                    a[2] *= inv;
                    a[3] *= inv;
                }
                CpuTarget::from_premultiplied(comp.width, comp.height, acc)
            }
            _ => {
                let tree = creator_engine::eval(project, comp_id, time);
                surface.draw(&tree);
                surface.read_back()
            }
        };
        let name = frame_filename("frame", frame, format);
        write_frame(&target, out.join(&name), format).with_context(|| format!("writing {name}"))?;
        let n = i + 1;
        if n % 25 == 0 || n == total {
            println!("  {n}/{total}");
        }
    }
    Ok(())
}

fn build_pool(threads: Option<usize>) -> Result<rayon::ThreadPool> {
    let mut builder = rayon::ThreadPoolBuilder::new();
    if let Some(n) = threads {
        builder = builder.num_threads(n.max(1));
    }
    builder.build().context("building thread pool")
}

/// Parse `"A-B"` (inclusive) or a single `"A"`.
fn parse_frame_range(spec: &str) -> Result<(u64, u64)> {
    let spec = spec.trim();
    match spec.split_once('-') {
        Some((a, b)) => {
            let a: u64 = a.trim().parse().with_context(|| format!("bad frame {a:?}"))?;
            let b: u64 = b.trim().parse().with_context(|| format!("bad frame {b:?}"))?;
            Ok((a, b))
        }
        None => {
            let f: u64 = spec.parse().with_context(|| format!("bad frame {spec:?}"))?;
            Ok((f, f))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ranges() {
        assert_eq!(parse_frame_range("0-120").unwrap(), (0, 120));
        assert_eq!(parse_frame_range("  5 - 9 ").unwrap(), (5, 9));
        assert_eq!(parse_frame_range("7").unwrap(), (7, 7));
        assert!(parse_frame_range("x-y").is_err());
    }
}
