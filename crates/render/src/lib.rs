//! `creator-render` — the render tree and a CPU rasterizer.
//!
//! The **render tree** ([`RenderTree`]) is the concrete-value language produced
//! by `creator-engine`'s `eval(scene, t)` and consumed by a backend. It carries
//! no animation or time — every value is already resolved for one instant. This
//! is the seam that lets the engine render the *same* scene to a live viewport,
//! an offscreen export, or a headless CPU buffer (PLAN.md §6/§7/§10).
//!
//! This crate ships the always-available **CPU raster** backend ([`render`] +
//! [`CpuTarget`]); the GPU (Skia Metal/Vulkan) targets live in `creator-gpu`.

mod raster;
mod target;

pub use raster::render;
pub use target::{CpuTarget, RenderTarget};

// Re-export the model value types the render tree is expressed in.
pub use creator_model::{BlendMode, Color, MatteMode, Polyline};

use glam::{Affine2, Vec2};

/// A fully-resolved scene ready to draw: a size, a background, and a root node.
#[derive(Debug, Clone)]
pub struct RenderTree {
    pub width: u32,
    pub height: u32,
    /// Solid backdrop composited first (alpha 0 == transparent).
    pub background: Color,
    pub root: RenderNode,
}

impl RenderTree {
    pub fn new(width: u32, height: u32, background: Color) -> Self {
        RenderTree { width, height, background, root: RenderNode::group(Vec::new()) }
    }
}

/// One node of the render tree. `transform` maps this node's local space into
/// its parent's space. `opacity`/`blend`/`effects` apply to the node *as a
/// whole* — for groups that means the children are flattened first.
#[derive(Debug, Clone)]
pub struct RenderNode {
    pub transform: Affine2,
    pub opacity: f32,
    pub blend: BlendMode,
    pub effects: Vec<ResolvedEffect>,
    /// Track matte: render `node` (in the same parent space as this node) to its
    /// own buffer and use it as per-pixel coverage for this node's pixels.
    pub matte: Option<(MatteMode, Box<RenderNode>)>,
    pub content: NodeContent,
}

impl RenderNode {
    pub fn group(children: Vec<RenderNode>) -> Self {
        RenderNode {
            transform: Affine2::IDENTITY,
            opacity: 1.0,
            blend: BlendMode::Normal,
            effects: Vec::new(),
            matte: None,
            content: NodeContent::Group(children),
        }
    }
    pub fn leaf(content: NodeContent) -> Self {
        RenderNode {
            transform: Affine2::IDENTITY,
            opacity: 1.0,
            blend: BlendMode::Normal,
            effects: Vec::new(),
            matte: None,
            content,
        }
    }
    pub fn with_transform(mut self, t: Affine2) -> Self {
        self.transform = t;
        self
    }
    pub fn with_opacity(mut self, o: f32) -> Self {
        self.opacity = o;
        self
    }
    pub fn with_blend(mut self, b: BlendMode) -> Self {
        self.blend = b;
        self
    }

    /// Whether this node must be rendered to its own offscreen buffer before
    /// compositing. Groups isolate when their group-level opacity/blend/effects
    /// would otherwise be (incorrectly) applied per-child; leaves only need
    /// isolation for effects (which read neighboring pixels).
    pub fn needs_isolation(&self) -> bool {
        // A matte multiplies this node's pixels before compositing — always
        // requires an offscreen.
        if self.matte.is_some() {
            return true;
        }
        let modified = self.opacity < 1.0
            || self.blend != BlendMode::Normal
            || !self.effects.is_empty();
        match &self.content {
            NodeContent::Group(_) => modified,
            // A shape with BOTH fill and stroke is two paints. If the node has a
            // non-default opacity/blend, those paints must be composited as a
            // unit (isolate) — otherwise the stroke blends against the
            // already-blended fill, which differs from "apply to the node as a
            // whole" for non-Normal/translucent nodes. Single-paint leaves are
            // identical inline vs isolated, so they only isolate for effects.
            NodeContent::Shape(s) if s.fill.is_some() && s.stroke.is_some() => modified,
            _ => !self.effects.is_empty(),
        }
    }
}

/// The drawable payload of a node.
#[derive(Debug, Clone)]
pub enum NodeContent {
    Group(Vec<RenderNode>),
    Shape(ResolvedShape),
    Text(ResolvedText),
}

/// A shape with concrete geometry and paints.
#[derive(Debug, Clone)]
pub struct ResolvedShape {
    pub geometry: ResolvedGeometry,
    pub fill: Option<ResolvedPaint>,
    pub stroke: Option<ResolvedStroke>,
}

/// A resolved fill paint, sampled per pixel in the shape's local space.
#[derive(Debug, Clone)]
pub enum ResolvedPaint {
    Solid(Color),
    /// Linear gradient along `start → end` with sorted `(offset, color)` stops.
    Linear { start: Vec2, end: Vec2, stops: Vec<(f32, Color)> },
    /// Radial gradient from `center` out to `radius` with sorted stops.
    Radial { center: Vec2, radius: f32, stops: Vec<(f32, Color)> },
}

impl ResolvedPaint {
    /// A convenience constructor for an opaque solid.
    pub fn solid(color: Color) -> Self {
        ResolvedPaint::Solid(color)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedStroke {
    pub color: Color,
    pub width: f32,
}

/// Concrete geometry, centered on the local origin.
#[derive(Debug, Clone)]
pub enum ResolvedGeometry {
    Rect { size: Vec2, corner_radius: f32 },
    Ellipse { size: Vec2 },
    /// Flattened path (the engine flattens model Béziers to polylines).
    Path { polylines: Vec<Polyline> },
}

/// Resolved text. The CPU backend renders a metrics-based placeholder box (real
/// glyph layout arrives with the Skia `textlayout` backend — PLAN.md §6).
#[derive(Debug, Clone)]
pub struct ResolvedText {
    pub content: String,
    pub font_size: f32,
    pub color: Color,
    pub tracking: f32,
    pub line_height: f32,
}

/// A resolved per-layer effect.
#[derive(Debug, Clone, Copy)]
pub enum ResolvedEffect {
    GaussianBlur { radius: f32 },
    Tint { color: Color, amount: f32 },
}
