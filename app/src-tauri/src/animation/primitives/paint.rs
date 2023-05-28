use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Color {
    value: (u8, u8, u8, f32),
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8, alpha: f32) -> Color {
        Color {
            value: (red, green, blue, alpha),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PaintStyle {
    Fill(FillStyle),
    Stroke(StrokeStyle),
    StrokeAndFill(StrokeAndFillStyle),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Paint {
    pub style: PaintStyle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextPaint {
    pub style: PaintStyle,
    pub align: TextAlign,
    pub font_name: String,
    pub size: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrokeStyle {
    pub color: Color,
    pub width: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrokeAndFillStyle {
    pub stroke: StrokeStyle,
    pub fill: FillStyle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FillStyle {
    pub color: Color,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontDefinition {
    pub family_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Font {
    pub glyph_count: i32,
    pub weight: i32,
    pub style: String,
}
