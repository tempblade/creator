//! Render targets: the CPU-accessible surface the software rasterizer writes to.
//!
//! Pixels are stored as **premultiplied linear `f32` RGBA**, the natural form
//! for compositing (PLAN.md §6: "composite in linear light"). `f32` stands in
//! for the eventual `F16` storage; the math is identical and the readout paths
//! (sRGB-8 for PNG, linear for EXR) are where precision is chosen.
//!
//! GPU window/offscreen targets (Metal/Vulkan via Skia) live in `creator-gpu`
//! and present the same scene; this trait is the seam for the CPU backend.

use creator_model::{linear_to_srgb, Color};

/// A surface the CPU rasterizer can read and write. Pixels are premultiplied
/// linear RGBA in row-major order.
pub trait RenderTarget {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn pixels(&self) -> &[[f32; 4]];
    fn pixels_mut(&mut self) -> &mut [[f32; 4]];
}

/// An in-memory CPU render target.
#[derive(Debug, Clone)]
pub struct CpuTarget {
    width: u32,
    height: u32,
    /// Premultiplied linear RGBA, `width * height`, row-major.
    pixels: Vec<[f32; 4]>,
}

impl CpuTarget {
    /// A fully transparent target.
    pub fn new(width: u32, height: u32) -> Self {
        CpuTarget {
            width,
            height,
            pixels: vec![[0.0; 4]; (width as usize) * (height as usize)],
        }
    }

    /// Build a target directly from premultiplied linear RGBA pixels (e.g. a
    /// motion-blur accumulator). Panics if the buffer length doesn't match.
    pub fn from_premultiplied(width: u32, height: u32, pixels: Vec<[f32; 4]>) -> Self {
        assert_eq!(
            pixels.len(),
            (width as usize) * (height as usize),
            "pixel buffer length must equal width*height"
        );
        CpuTarget { width, height, pixels }
    }

    /// Fill the whole target with a solid (straight-alpha) color, premultiplied.
    pub fn clear(&mut self, color: Color) {
        let p = color.premultiplied();
        for px in &mut self.pixels {
            *px = p;
        }
    }

    /// Encode to 8-bit straight-alpha sRGB RGBA (for PNG). Un-premultiplies,
    /// applies the linear→sRGB transfer to RGB, and clamps.
    pub fn to_srgba8(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.pixels.len() * 4);
        for px in &self.pixels {
            let a = px[3];
            let (r, g, b) = if a > 0.0 {
                (px[0] / a, px[1] / a, px[2] / a)
            } else {
                (0.0, 0.0, 0.0)
            };
            out.push((linear_to_srgb(r).clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
            out.push((linear_to_srgb(g).clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
            out.push((linear_to_srgb(b).clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
            out.push((a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
        }
        out
    }

    /// Straight-alpha **linear** RGBA `f32` channels (for EXR / HDR export).
    pub fn to_linear_f32(&self) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.pixels.len() * 4);
        for px in &self.pixels {
            let a = px[3];
            if a > 0.0 {
                out.extend_from_slice(&[px[0] / a, px[1] / a, px[2] / a, a]);
            } else {
                out.extend_from_slice(&[0.0, 0.0, 0.0, 0.0]);
            }
        }
        out
    }

    /// Sample a single pixel's premultiplied linear RGBA (for tests).
    pub fn pixel(&self, x: u32, y: u32) -> [f32; 4] {
        self.pixels[(y as usize) * (self.width as usize) + (x as usize)]
    }
}

impl RenderTarget for CpuTarget {
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
    fn pixels(&self) -> &[[f32; 4]] {
        &self.pixels
    }
    fn pixels_mut(&mut self) -> &mut [[f32; 4]] {
        &mut self.pixels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_and_readout_opaque() {
        let mut t = CpuTarget::new(2, 2);
        t.clear(Color::from_srgb8(255, 0, 0, 255));
        let rgba = t.to_srgba8();
        assert_eq!(&rgba[0..4], &[255, 0, 0, 255]);
    }

    #[test]
    fn transparent_reads_back_zero() {
        let t = CpuTarget::new(1, 1);
        assert_eq!(t.to_srgba8(), vec![0, 0, 0, 0]);
    }
}
