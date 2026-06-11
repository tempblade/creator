//! Linear-light RGBA color.
//!
//! Per PLAN.md §6/§9, the engine composites in **linear light**, so colors are
//! stored as linear (not sRGB-encoded) `f32` with straight (non-premultiplied)
//! alpha. The display/sRGB transfer is applied only at readout.

use creator_anim::Lerp;
use serde::{Deserialize, Serialize};

/// Linear RGBA, straight alpha. Components are unbounded above `1.0` to allow
/// HDR / over-range values, and may be interpolated/extrapolated freely.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

    /// Construct from linear components.
    pub const fn linear(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    /// Construct from 8-bit **sRGB** channels (the usual `#rrggbbaa` authoring
    /// form); RGB are converted to linear, alpha stays linear.
    pub fn from_srgb8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color {
            r: srgb_to_linear(r as f32 / 255.0),
            g: srgb_to_linear(g as f32 / 255.0),
            b: srgb_to_linear(b as f32 / 255.0),
            a: a as f32 / 255.0,
        }
    }

    /// Parse `#rgb`, `#rgba`, `#rrggbb`, or `#rrggbbaa` (sRGB).
    pub fn from_hex(hex: &str) -> Option<Self> {
        let h = hex.strip_prefix('#').unwrap_or(hex);
        // We byte-slice below; reject non-ASCII so a multibyte char can't make a
        // slice boundary land mid-char (which would panic).
        if !h.is_ascii() {
            return None;
        }
        let n = |s: &str| u8::from_str_radix(s, 16).ok();
        match h.len() {
            3 => {
                let r = n(&h[0..1])?;
                let g = n(&h[1..2])?;
                let b = n(&h[2..3])?;
                Some(Color::from_srgb8(r * 17, g * 17, b * 17, 255))
            }
            4 => {
                let r = n(&h[0..1])?;
                let g = n(&h[1..2])?;
                let b = n(&h[2..3])?;
                let a = n(&h[3..4])?;
                Some(Color::from_srgb8(r * 17, g * 17, b * 17, a * 17))
            }
            6 => Some(Color::from_srgb8(n(&h[0..2])?, n(&h[2..4])?, n(&h[4..6])?, 255)),
            8 => Some(Color::from_srgb8(
                n(&h[0..2])?,
                n(&h[2..4])?,
                n(&h[4..6])?,
                n(&h[6..8])?,
            )),
            _ => None,
        }
    }

    /// Encode to 8-bit sRGB channels (clamped), applying the display transfer.
    pub fn to_srgb8(self) -> [u8; 4] {
        [
            (linear_to_srgb(self.r).clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (linear_to_srgb(self.g).clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (linear_to_srgb(self.b).clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (self.a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        ]
    }

    /// Premultiplied linear RGBA, the natural form for compositing.
    pub fn premultiplied(self) -> [f32; 4] {
        [self.r * self.a, self.g * self.a, self.b * self.a, self.a]
    }
}

impl Lerp for Color {
    fn mix(&self, other: &Self, t: f64) -> Self {
        let t = t as f32;
        Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }
}

/// sRGB → linear (IEC 61966-2-1).
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// linear → sRGB.
pub fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srgb_round_trip() {
        for v in [0u8, 1, 64, 128, 200, 255] {
            let c = Color::from_srgb8(v, v, v, 255);
            let back = c.to_srgb8();
            assert!((back[0] as i32 - v as i32).abs() <= 1, "{v} -> {}", back[0]);
        }
    }

    #[test]
    fn transfer_endpoints() {
        assert!((srgb_to_linear(0.0)).abs() < 1e-9);
        assert!((srgb_to_linear(1.0) - 1.0).abs() < 1e-6);
        assert!((linear_to_srgb(0.0)).abs() < 1e-9);
        assert!((linear_to_srgb(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn hex_parsing() {
        assert_eq!(Color::from_hex("#000000ff"), Some(Color::BLACK));
        assert_eq!(Color::from_hex("#ffffff"), Some(Color::WHITE));
        assert_eq!(Color::from_hex("fff"), Some(Color::WHITE));
        assert!(Color::from_hex("nope").is_none());
    }

    #[test]
    fn hex_rgba_shorthand() {
        // #rgba documented form: "#f008" -> opaque-scaled with ~53% alpha.
        let c = Color::from_hex("#f008").unwrap();
        assert!((c.r - 1.0).abs() < 1e-6 && c.g < 1e-6 && c.b < 1e-6);
        assert!((c.a - (0x88 as f32 / 255.0)).abs() < 1e-6);
    }

    #[test]
    fn hex_rejects_multibyte_without_panicking() {
        // "éa" is 3 bytes; byte-slicing would split 'é' and panic without the
        // ASCII guard. Must return None instead.
        assert_eq!(Color::from_hex("éa"), None);
        assert_eq!(Color::from_hex("#café"), None);
    }

    #[test]
    fn mix_in_linear() {
        let m = Color::BLACK.mix(&Color::WHITE, 0.5);
        assert!((m.r - 0.5).abs() < 1e-6);
        assert!((m.a - 1.0).abs() < 1e-6);
    }
}
