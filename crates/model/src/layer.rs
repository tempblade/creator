//! Layers: the drawable/organizational units inside a composition.

use crate::geometry::{Shape, Transform};
use crate::property::Property;
use crate::{Color, CompId, LayerId};
use serde::{Deserialize, Serialize};

/// Track-matte modes (PLAN.md §6). A matted layer takes its coverage from the
/// layer **above** it in the stack (the AE convention); the matte layer itself
/// is not drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatteMode {
    /// Coverage = matte alpha.
    Alpha,
    /// Coverage = 1 − matte alpha.
    AlphaInverted,
    /// Coverage = matte luminance × alpha (linear-light Rec.709 luma).
    Luma,
    /// Coverage = 1 − luminance × alpha.
    LumaInverted,
}

/// Porter-Duff-ish blend modes implemented by the compositor (PLAN.md §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    /// Linear dodge / additive.
    Add,
    Overlay,
    Darken,
    Lighten,
}

/// A text layer. Real glyph layout/rasterization arrives with the Skia
/// `textlayout` backend (PLAN.md §6); the CPU MVP renders a metrics-based
/// placeholder so text-bearing projects still produce frames.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Text {
    pub content: String,
    pub font_family: String,
    pub font_size: Property<f64>,
    pub color: Property<Color>,
    /// Letter spacing in local units.
    pub tracking: Property<f64>,
    /// Line height multiplier.
    pub line_height: Property<f64>,
}

impl Text {
    pub fn new(content: impl Into<String>) -> Self {
        Text {
            content: content.into(),
            font_family: "sans-serif".into(),
            font_size: Property::constant(48.0),
            color: Property::constant(Color::WHITE),
            tracking: Property::constant(0.0),
            line_height: Property::constant(1.2),
        }
    }
}

/// A per-layer effect. Native filters first; OpenFX plugins map into this list
/// later (PLAN.md §4/§8).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Effect {
    pub enabled: bool,
    pub kind: EffectKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EffectKind {
    /// Separable approximate Gaussian blur; `radius` in pixels.
    GaussianBlur { radius: Property<f64> },
    /// Blend the layer toward `color` by `amount` (0..1).
    Tint { color: Property<Color>, amount: Property<f64> },
}

/// What a layer draws / represents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LayerKind {
    Shape(Shape),
    Text(Text),
    /// A nested composition (precomp). Renders the referenced comp to its own
    /// surface, then composites it like any other layer.
    Precomp(CompId),
    /// Transform-only layer (an "object"/null); useful as a parent so a group of
    /// layers inherits one transform.
    Null,
}

/// A layer inside a composition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub enabled: bool,
    pub transform: Transform,
    /// Layer opacity, 0..1.
    pub opacity: Property<f64>,
    pub blend: BlendMode,
    pub kind: LayerKind,
    /// Optional transform parent within the same composition. Parent cycles
    /// (of any length) are detected and broken during evaluation, so a malformed
    /// chain yields a bounded transform rather than a hang. (PLAN.md
    /// "group/precomp": grouping is expressed as parented layers / precomps.)
    pub parent: Option<LayerId>,
    /// Visibility window `[start, end]` in seconds. `None` == always visible.
    pub time_range: Option<(f64, f64)>,
    /// Ordered effect chain (applied after the layer is drawn).
    pub effects: Vec<Effect>,
    /// Track matte: take coverage from the layer above (defaulted so older
    /// project files still load).
    #[serde(default)]
    pub matte: Option<MatteMode>,
}

impl Layer {
    /// A layer of the given kind with sane defaults.
    pub fn new(name: impl Into<String>, kind: LayerKind) -> Self {
        Layer {
            name: name.into(),
            enabled: true,
            transform: Transform::default(),
            opacity: Property::constant(1.0),
            blend: BlendMode::Normal,
            kind,
            parent: None,
            time_range: None,
            effects: Vec::new(),
            matte: None,
        }
    }

    /// Is this layer visible at `time` (enabled and inside its time range)?
    pub fn visible_at(&self, time: f64) -> bool {
        if !self.enabled {
            return false;
        }
        match self.time_range {
            Some((start, end)) => time >= start && time < end,
            None => true,
        }
    }
}
