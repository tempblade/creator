//! `creator-media` — media import.
//!
//! Images now (via the `image` crate); frame-accurate video decode arrives later
//! via `ffmpeg-next` (PLAN.md §9). Imported pixels are converted to the engine's
//! internal form — **premultiplied linear RGBA `f32`** — so they composite
//! correctly alongside rendered layers.

use creator_model::srgb_to_linear;
use std::path::Path;

/// An image decoded into the engine's linear pixel format.
#[derive(Debug, Clone)]
pub struct ImportedImage {
    pub width: u32,
    pub height: u32,
    /// Premultiplied linear RGBA, row-major (`width * height`).
    pub pixels: Vec<[f32; 4]>,
}

/// Errors importing media.
#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("failed to decode image: {0}")]
    Decode(#[from] image::ImageError),
}

/// Load and decode an image file (PNG/JPEG/…), converting sRGB→linear and
/// premultiplying alpha.
pub fn load_image(path: impl AsRef<Path>) -> Result<ImportedImage, MediaError> {
    let img = image::open(path)?.to_rgba8();
    let (width, height) = img.dimensions();
    let mut pixels = Vec::with_capacity((width * height) as usize);
    for px in img.pixels() {
        let a = px[3] as f32 / 255.0;
        let r = srgb_to_linear(px[0] as f32 / 255.0);
        let g = srgb_to_linear(px[1] as f32 / 255.0);
        let b = srgb_to_linear(px[2] as f32 / 255.0);
        // store premultiplied
        pixels.push([r * a, g * a, b * a, a]);
    }
    Ok(ImportedImage { width, height, pixels })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_written_png() {
        // Write a 2x2 opaque-red PNG, then import it.
        let dir = std::env::temp_dir().join("creator_media_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("red.png");
        let mut buf = image::RgbaImage::new(2, 2);
        for p in buf.pixels_mut() {
            *p = image::Rgba([255, 0, 0, 255]);
        }
        buf.save(&path).unwrap();

        let img = load_image(&path).unwrap();
        assert_eq!((img.width, img.height), (2, 2));
        // sRGB 255 -> linear 1.0; premultiplied with alpha 1.
        let p = img.pixels[0];
        assert!((p[0] - 1.0).abs() < 1e-4 && p[1] < 1e-4 && (p[3] - 1.0).abs() < 1e-4);

        let _ = std::fs::remove_file(&path);
    }
}
