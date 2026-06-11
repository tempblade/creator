//! `creator-model` — the document model.
//!
//! Project → compositions → layers, with typed animatable [`Property`]s, stable
//! IDs via `slotmap`, and a versioned serde JSON project format (PLAN.md §4).
//! This crate is pure data + evaluation of single properties; flattening a whole
//! scene at a time lives in `creator-engine`.

mod color;
mod geometry;
mod layer;
mod property;

pub use color::{linear_to_srgb, srgb_to_linear, Color};
pub use geometry::{
    Fill, GradientStop, Geometry, Paint, PathData, PathPoint, Polyline, Shape, Stroke, SubPath,
    Transform,
};
pub use layer::{BlendMode, Effect, EffectKind, Layer, LayerKind, MatteMode, Text};
pub use property::{Keyframe, Property};

pub use glam::{Affine2, Vec2, Vec3};

use serde::{Deserialize, Serialize};
use slotmap::SlotMap;

slotmap::new_key_type! {
    /// Stable composition handle.
    pub struct CompId;
    /// Stable layer handle (unique within its composition).
    pub struct LayerId;
}

/// Current on-disk project format version. Bump on breaking changes and add a
/// migration in [`Project::from_json`].
pub const FORMAT_VERSION: u32 = 1;

/// A whole project document.
///
/// `SlotMap` does not implement `PartialEq`, so neither does `Project`; compare
/// projects by round-tripping through [`Project::to_json`] when needed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// On-disk format version (see [`FORMAT_VERSION`]).
    pub version: u32,
    pub name: String,
    pub compositions: SlotMap<CompId, Composition>,
    /// The composition shown/rendered by default.
    pub root: Option<CompId>,
}

impl Default for Project {
    fn default() -> Self {
        Project {
            version: FORMAT_VERSION,
            name: "Untitled".into(),
            compositions: SlotMap::with_key(),
            root: None,
        }
    }
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Project { name: name.into(), ..Default::default() }
    }

    /// Insert a composition, returning its stable id. The first composition
    /// added becomes the `root` if none is set.
    pub fn add_composition(&mut self, comp: Composition) -> CompId {
        let id = self.compositions.insert(comp);
        if self.root.is_none() {
            self.root = Some(id);
        }
        id
    }

    pub fn composition(&self, id: CompId) -> Option<&Composition> {
        self.compositions.get(id)
    }
    pub fn composition_mut(&mut self, id: CompId) -> Option<&mut Composition> {
        self.compositions.get_mut(id)
    }

    /// Serialize to pretty JSON (diffable; PLAN.md §1 "serde + JSON").
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize, applying version migrations.
    pub fn from_json(s: &str) -> Result<Project, ProjectError> {
        // Peek at the version before full decode so we can migrate.
        let probe: VersionProbe = serde_json::from_str(s)?;
        if probe.version > FORMAT_VERSION {
            return Err(ProjectError::FutureVersion {
                found: probe.version,
                supported: FORMAT_VERSION,
            });
        }
        // v1 is current; future migrations slot in here as a version ladder.
        let project: Project = serde_json::from_str(s)?;
        Ok(project)
    }
}

#[derive(Deserialize)]
struct VersionProbe {
    #[serde(default)]
    version: u32,
}

/// Errors loading a project. Hand-rolled `Display`/`Error` to keep the model
/// crate free of an error-derive dependency.
#[derive(Debug)]
pub enum ProjectError {
    Json(serde_json::Error),
    FutureVersion { found: u32, supported: u32 },
}

impl std::fmt::Display for ProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectError::Json(e) => write!(f, "invalid project JSON: {e}"),
            ProjectError::FutureVersion { found, supported } => write!(
                f,
                "project format v{found} is newer than supported v{supported}; upgrade the app"
            ),
        }
    }
}
impl std::error::Error for ProjectError {}
impl From<serde_json::Error> for ProjectError {
    fn from(e: serde_json::Error) -> Self {
        ProjectError::Json(e)
    }
}

/// Motion-blur settings for a composition (temporal supersampling).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MotionBlur {
    /// Temporal samples per frame (`>= 2` to have any effect).
    pub samples: u32,
    /// Shutter angle in degrees, `0..360`. `180` = shutter open for half the
    /// frame interval (the film-standard default).
    pub shutter_angle: f64,
}

impl Default for MotionBlur {
    fn default() -> Self {
        MotionBlur { samples: 16, shutter_angle: 180.0 }
    }
}

/// A composition: a canvas with a timeline and an ordered stack of layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composition {
    pub name: String,
    pub width: u32,
    pub height: u32,
    /// Frames per second.
    pub frame_rate: f64,
    /// Duration in seconds.
    pub duration: f64,
    /// Solid background (use alpha 0 for a transparent comp).
    pub background: Color,
    pub layers: SlotMap<LayerId, Layer>,
    /// Draw order, **back-to-front** (last entry is topmost).
    pub order: Vec<LayerId>,
    /// Optional motion blur. `None` == off. Defaulted so older project files
    /// (without the field) still load.
    #[serde(default)]
    pub motion_blur: Option<MotionBlur>,
}

impl Composition {
    pub fn new(name: impl Into<String>, width: u32, height: u32, frame_rate: f64, duration: f64) -> Self {
        Composition {
            name: name.into(),
            width,
            height,
            frame_rate,
            duration,
            background: Color::TRANSPARENT,
            layers: SlotMap::with_key(),
            order: Vec::new(),
            motion_blur: None,
        }
    }

    /// Add a layer to the top of the stack, returning its id.
    pub fn add_layer(&mut self, layer: Layer) -> LayerId {
        let id = self.layers.insert(layer);
        self.order.push(id);
        id
    }

    pub fn layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.get(id)
    }
    pub fn layer_mut(&mut self, id: LayerId) -> Option<&mut Layer> {
        self.layers.get_mut(id)
    }

    /// Total frame count (`duration * frame_rate`, rounded).
    pub fn frame_count(&self) -> u64 {
        (self.duration * self.frame_rate).round().max(0.0) as u64
    }

    /// Convert a frame index to a time in seconds.
    pub fn frame_to_time(&self, frame: u64) -> f64 {
        if self.frame_rate > 0.0 {
            frame as f64 / self.frame_rate
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_project() -> (Project, CompId) {
        let mut project = Project::new("Demo");
        let mut comp = Composition::new("Main", 200, 100, 30.0, 2.0);
        comp.background = Color::from_srgb8(10, 12, 20, 255);
        let mut layer = Layer::new("Box", LayerKind::Shape(Shape::rect(Vec2::new(40.0, 40.0), Color::WHITE)));
        layer.transform.position = Property::animated(vec![
            Keyframe::new(0.0, Vec2::new(20.0, 50.0)),
            Keyframe::new(2.0, Vec2::new(180.0, 50.0)),
        ]);
        comp.add_layer(layer);
        let id = project.add_composition(comp);
        (project, id)
    }

    #[test]
    fn json_round_trip() {
        let (project, _) = sample_project();
        let json = project.to_json().unwrap();
        // `Project` has no `PartialEq` (SlotMap); compare re-serialized JSON,
        // which is a stronger structural check anyway.
        let back = Project::from_json(&json).unwrap();
        assert_eq!(json, back.to_json().unwrap());
    }

    #[test]
    fn root_set_to_first_comp() {
        let (project, id) = sample_project();
        assert_eq!(project.root, Some(id));
    }

    #[test]
    fn frame_math() {
        let comp = Composition::new("c", 10, 10, 30.0, 2.0);
        assert_eq!(comp.frame_count(), 60);
        assert!((comp.frame_to_time(30) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn future_version_rejected() {
        let json = r#"{"version":9999,"name":"x","compositions":{},"root":null}"#;
        assert!(matches!(Project::from_json(json), Err(ProjectError::FutureVersion { .. })));
    }
}
