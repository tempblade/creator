//! `creator-export` — encode rendered frames.
//!
//! Image sequences first (PNG sRGB-8, EXR linear-`f32`), then video via an
//! **ffmpeg sidecar** (PLAN.md §9). The sidecar keeps the core license-clean and
//! server-robust: we shell out to a system `ffmpeg`, building the argument list
//! here (and exposing it for tests) rather than linking libav.

use creator_render::{CpuTarget, RenderTarget};
use std::path::Path;
use std::process::Command;

/// Errors writing exported frames/video.
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("image encode error: {0}")]
    Image(#[from] image::ImageError),
    #[error("exr encode error: {0}")]
    Exr(#[from] exr::error::Error),
    #[error("could not build image buffer from pixels")]
    Buffer,
}

/// Still-frame output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameFormat {
    /// 8-bit sRGB PNG.
    Png,
    /// Linear `f32` OpenEXR (HDR, matches the internal pixel format).
    Exr,
}

impl FrameFormat {
    pub fn extension(self) -> &'static str {
        match self {
            FrameFormat::Png => "png",
            FrameFormat::Exr => "exr",
        }
    }
}

/// Write a single frame to `path` in the given format.
pub fn write_frame(target: &CpuTarget, path: impl AsRef<Path>, format: FrameFormat) -> Result<(), ExportError> {
    match format {
        FrameFormat::Png => write_png(target, path),
        FrameFormat::Exr => write_exr(target, path),
    }
}

/// Write an 8-bit sRGB PNG.
pub fn write_png(target: &CpuTarget, path: impl AsRef<Path>) -> Result<(), ExportError> {
    let bytes = target.to_srgba8();
    let img = image::RgbaImage::from_raw(target.width(), target.height(), bytes)
        .ok_or(ExportError::Buffer)?;
    img.save(path)?;
    Ok(())
}

/// Encode an 8-bit sRGB PNG into memory (e.g. for a viewport data URL — no
/// temp file round-trip).
pub fn encode_png(target: &CpuTarget) -> Result<Vec<u8>, ExportError> {
    let bytes = target.to_srgba8();
    let img = image::RgbaImage::from_raw(target.width(), target.height(), bytes)
        .ok_or(ExportError::Buffer)?;
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png)?;
    Ok(out.into_inner())
}

/// Write a linear `f32` RGBA OpenEXR.
pub fn write_exr(target: &CpuTarget, path: impl AsRef<Path>) -> Result<(), ExportError> {
    let (w, h) = (target.width() as usize, target.height() as usize);
    let linear = target.to_linear_f32(); // straight-alpha linear RGBA, row-major
    exr::prelude::write_rgba_file(path, w, h, |x, y| {
        let i = (y * w + x) * 4;
        (linear[i], linear[i + 1], linear[i + 2], linear[i + 3])
    })?;
    Ok(())
}

/// Build the `frame_00042.png` style file name for a sequence.
pub fn frame_filename(prefix: &str, frame: u64, format: FrameFormat) -> String {
    format!("{prefix}_{frame:05}.{}", format.extension())
}

// --- ffmpeg sidecar ---------------------------------------------------------

/// Video codec → ffmpeg encoder + pixel-format choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodec {
    /// H.264 / MP4 (`libx264`, `yuv420p`).
    H264,
    /// Apple ProRes (`prores_ks`).
    ProRes,
    /// VP9 / WebM (`libvpx-vp9`).
    Vp9,
}

impl VideoCodec {
    fn args(self) -> Vec<&'static str> {
        match self {
            VideoCodec::H264 => vec!["-c:v", "libx264", "-pix_fmt", "yuv420p"],
            VideoCodec::ProRes => vec!["-c:v", "prores_ks", "-profile:v", "3"],
            VideoCodec::Vp9 => vec!["-c:v", "libvpx-vp9", "-pix_fmt", "yuv420p"],
        }
    }
}

/// Build the ffmpeg `Command` that encodes a numbered PNG sequence into a video.
///
/// `input_pattern` is an ffmpeg-style pattern such as
/// `frames/frame_%05d.png`. The command is returned unspawned so callers (and
/// tests) can inspect or further configure it. Running it requires `ffmpeg` on
/// the `PATH`.
pub fn ffmpeg_command(
    input_pattern: &str,
    fps: f64,
    output: &str,
    codec: VideoCodec,
) -> Command {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .args(["-framerate", &format!("{fps}")])
        .args(["-i", input_pattern]);
    cmd.args(codec.args());
    cmd.arg(output);
    cmd
}

/// Encode a PNG sequence to video by spawning the ffmpeg sidecar. Returns an
/// error (rather than panicking) if `ffmpeg` is not installed.
pub fn encode_video(
    input_pattern: &str,
    fps: f64,
    output: &str,
    codec: VideoCodec,
) -> Result<std::process::ExitStatus, ExportError> {
    let status = ffmpeg_command(input_pattern, fps, output, codec).status()?;
    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use creator_render::Color;

    fn red_target() -> CpuTarget {
        let mut t = CpuTarget::new(4, 4);
        t.clear(Color::from_srgb8(255, 0, 0, 255));
        t
    }

    #[test]
    fn writes_png() {
        let dir = std::env::temp_dir().join("creator_export_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("f.png");
        write_frame(&red_target(), &path, FrameFormat::Png).unwrap();
        // read it back via the image crate to confirm it's a valid PNG.
        let img = image::open(&path).unwrap().to_rgba8();
        assert_eq!(img.dimensions(), (4, 4));
        assert_eq!(img.get_pixel(0, 0).0, [255, 0, 0, 255]);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn encodes_png_in_memory() {
        let bytes = encode_png(&red_target()).unwrap();
        assert_eq!(&bytes[1..4], b"PNG");
        let img = image::load_from_memory(&bytes).unwrap().to_rgba8();
        assert_eq!(img.get_pixel(0, 0).0, [255, 0, 0, 255]);
    }

    #[test]
    fn writes_exr() {
        let dir = std::env::temp_dir().join("creator_export_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("f.exr");
        write_frame(&red_target(), &path, FrameFormat::Exr).unwrap();
        assert!(path.exists() && std::fs::metadata(&path).unwrap().len() > 0);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn frame_filenames() {
        assert_eq!(frame_filename("frame", 42, FrameFormat::Png), "frame_00042.png");
        assert_eq!(frame_filename("frame", 7, FrameFormat::Exr), "frame_00007.exr");
    }

    #[test]
    fn ffmpeg_command_args() {
        let cmd = ffmpeg_command("frames/frame_%05d.png", 30.0, "out.mp4", VideoCodec::H264);
        let args: Vec<String> = cmd.get_args().map(|a| a.to_string_lossy().into_owned()).collect();
        assert!(args.contains(&"-framerate".to_string()));
        assert!(args.contains(&"libx264".to_string()));
        assert_eq!(args.last().unwrap(), "out.mp4");
    }
}
