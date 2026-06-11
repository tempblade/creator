//! 2D transforms and shape geometry.

use crate::property::Property;
use crate::Color;
use creator_anim::Lerp;
use glam::{Affine2, Mat2, Vec2};
use serde::{Deserialize, Serialize};

/// A 2D affine transform with animatable channels (PLAN.md §4: anchor, position,
/// scale, rotation, skew — opacity lives on the layer). 2.5D/3D is a later
/// extension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub anchor: Property<Vec2>,
    pub position: Property<Vec2>,
    /// Scale where `1.0` == 100%.
    pub scale: Property<Vec2>,
    /// Rotation in degrees (clockwise in a y-down raster space).
    pub rotation: Property<f64>,
    /// Skew angle in degrees (horizontal shear).
    pub skew: Property<f64>,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            anchor: Property::constant(Vec2::ZERO),
            position: Property::constant(Vec2::ZERO),
            scale: Property::constant(Vec2::ONE),
            rotation: Property::constant(0.0),
            skew: Property::constant(0.0),
        }
    }
}

impl Transform {
    /// The local→parent affine at `time`:
    /// `T(position) · R(rotation) · Shear(skew) · S(scale) · T(−anchor)`.
    pub fn matrix(&self, time: f64) -> Affine2 {
        let anchor = self.anchor.eval(time);
        let position = self.position.eval(time);
        let scale = self.scale.eval(time);
        let rot = (self.rotation.eval(time) as f32).to_radians();
        let skew = (self.skew.eval(time) as f32).to_radians();
        // Horizontal shear: x' = x + tan(skew)·y. Column-major: col0=(1,0),
        // col1=(tan,1).
        let shear = Affine2::from_mat2(Mat2::from_cols_array(&[1.0, 0.0, skew.tan(), 1.0]));
        Affine2::from_translation(position)
            * Affine2::from_angle(rot)
            * shear
            * Affine2::from_scale(scale)
            * Affine2::from_translation(-anchor)
    }
}

/// The geometric primitive of a shape layer. All measurements are in the
/// layer's local space (the anchor is applied by the [`Transform`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Geometry {
    /// Axis-aligned rectangle of `size`, centered on the origin, with optional
    /// uniform `corner_radius`.
    Rect { size: Property<Vec2>, corner_radius: Property<f64> },
    /// Ellipse inscribed in `size`, centered on the origin.
    Ellipse { size: Property<Vec2> },
    /// Arbitrary flattened path (polylines). Bézier control later.
    Path { data: Property<PathData> },
}

/// A fill paint: solid or gradient. Coordinates are in the shape's local space.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Paint {
    Solid(Property<Color>),
    /// Linear gradient from `start` to `end` (local space).
    LinearGradient { start: Property<Vec2>, end: Property<Vec2>, stops: Vec<GradientStop> },
    /// Radial gradient centered at `center` with the given `radius` (local space).
    RadialGradient { center: Property<Vec2>, radius: Property<f64>, stops: Vec<GradientStop> },
}

/// A gradient color stop. `offset` is `0..1` along the gradient; the color is
/// animatable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GradientStop {
    pub offset: f32,
    pub color: Property<Color>,
}

impl GradientStop {
    pub fn new(offset: f32, color: Color) -> Self {
        GradientStop { offset, color: Property::constant(color) }
    }
}

/// A fill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fill {
    pub paint: Paint,
}

impl Fill {
    pub fn solid(color: Color) -> Self {
        Fill { paint: Paint::Solid(Property::constant(color)) }
    }
    pub fn linear(start: Vec2, end: Vec2, stops: Vec<GradientStop>) -> Self {
        Fill {
            paint: Paint::LinearGradient {
                start: Property::constant(start),
                end: Property::constant(end),
                stops,
            },
        }
    }
    pub fn radial(center: Vec2, radius: f64, stops: Vec<GradientStop>) -> Self {
        Fill {
            paint: Paint::RadialGradient {
                center: Property::constant(center),
                radius: Property::constant(radius),
                stops,
            },
        }
    }
}

/// A stroke of uniform `width` (local units).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    pub color: Property<Color>,
    pub width: Property<f64>,
}

/// A shape layer's drawable content: a geometry with optional fill and stroke.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    pub geometry: Geometry,
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
}

impl Shape {
    /// A filled rectangle.
    pub fn rect(size: Vec2, fill: Color) -> Self {
        Shape {
            geometry: Geometry::Rect {
                size: Property::constant(size),
                corner_radius: Property::constant(0.0),
            },
            fill: Some(Fill::solid(fill)),
            stroke: None,
        }
    }
    /// A filled ellipse.
    pub fn ellipse(size: Vec2, fill: Color) -> Self {
        Shape {
            geometry: Geometry::Ellipse { size: Property::constant(size) },
            fill: Some(Fill::solid(fill)),
            stroke: None,
        }
    }
}

/// A vector path: one or more cubic-Bézier subpaths in local space.
///
/// Each [`PathPoint`] carries an anchor plus in/out tangent handles (relative to
/// the anchor, the Lottie/AE convention); a segment between consecutive points
/// is the cubic `anchor[i] → anchor[i]+out[i] → anchor[i+1]+in[i+1] →
/// anchor[i+1]`. Zero handles give straight lines. Rasterization consumes the
/// [`PathData::flatten`] polyline form.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PathData {
    pub subpaths: Vec<SubPath>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubPath {
    pub closed: bool,
    pub points: Vec<PathPoint>,
}

/// A path vertex: an anchor with cubic tangent handles relative to it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PathPoint {
    pub anchor: Vec2,
    /// Incoming tangent handle, relative to `anchor`.
    pub in_handle: Vec2,
    /// Outgoing tangent handle, relative to `anchor`.
    pub out_handle: Vec2,
}

impl PathPoint {
    /// A corner (no curvature).
    pub fn corner(anchor: Vec2) -> Self {
        PathPoint { anchor, in_handle: Vec2::ZERO, out_handle: Vec2::ZERO }
    }
    /// A smooth point with mirrored tangents (`out` and `in = -out`).
    pub fn smooth(anchor: Vec2, out_handle: Vec2) -> Self {
        PathPoint { anchor, in_handle: -out_handle, out_handle }
    }
}

/// A flattened polyline (the rasterizer's input form).
#[derive(Debug, Clone, PartialEq)]
pub struct Polyline {
    pub closed: bool,
    pub points: Vec<Vec2>,
}

/// Cubic flattening subdivisions per curved segment.
const FLATTEN_STEPS: usize = 24;

impl PathData {
    /// A straight-segment polygon/polyline from corner points.
    pub fn polygon(points: impl IntoIterator<Item = Vec2>, closed: bool) -> Self {
        PathData {
            subpaths: vec![SubPath {
                closed,
                points: points.into_iter().map(PathPoint::corner).collect(),
            }],
        }
    }

    /// Flatten all subpaths to polylines by sampling each cubic segment. Straight
    /// segments (zero handles on both ends) emit just their endpoints.
    pub fn flatten(&self) -> Vec<Polyline> {
        self.subpaths.iter().map(|sp| sp.flatten()).collect()
    }
}

impl SubPath {
    fn flatten(&self) -> Polyline {
        let n = self.points.len();
        let mut out = Vec::new();
        if n == 0 {
            return Polyline { closed: self.closed, points: out };
        }
        out.push(self.points[0].anchor);
        let seg_count = if self.closed { n } else { n - 1 };
        for i in 0..seg_count {
            let a = self.points[i];
            let b = self.points[(i + 1) % n];
            let p0 = a.anchor;
            let p1 = a.anchor + a.out_handle;
            let p2 = b.anchor + b.in_handle;
            let p3 = b.anchor;
            // Straight segment: just emit the endpoint.
            if a.out_handle == Vec2::ZERO && b.in_handle == Vec2::ZERO {
                out.push(p3);
            } else {
                for s in 1..=FLATTEN_STEPS {
                    let t = s as f32 / FLATTEN_STEPS as f32;
                    out.push(cubic_at(p0, p1, p2, p3, t));
                }
            }
        }
        // For a closed path the last pushed point coincides with the start; the
        // rasterizer treats the polyline as implicitly closed, so drop the dup.
        if self.closed && out.len() > 1 && out[out.len() - 1] == out[0] {
            out.pop();
        }
        Polyline { closed: self.closed, points: out }
    }
}

/// Evaluate a cubic Bézier at parameter `t ∈ [0,1]`.
fn cubic_at(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t)
}

impl Lerp for PathData {
    /// Interpolate point-wise (anchors and both handles) when the two paths
    /// share the same topology (subpath count, per-subpath point counts,
    /// closedness); otherwise hold the start path.
    fn mix(&self, other: &Self, t: f64) -> Self {
        if self.subpaths.len() != other.subpaths.len() {
            return self.clone();
        }
        let compatible = self
            .subpaths
            .iter()
            .zip(&other.subpaths)
            .all(|(a, b)| a.closed == b.closed && a.points.len() == b.points.len());
        if !compatible {
            return self.clone();
        }
        let subpaths = self
            .subpaths
            .iter()
            .zip(&other.subpaths)
            .map(|(a, b)| SubPath {
                closed: a.closed,
                points: a
                    .points
                    .iter()
                    .zip(&b.points)
                    .map(|(p, q)| PathPoint {
                        anchor: p.anchor.mix(&q.anchor, t),
                        in_handle: p.in_handle.mix(&q.in_handle, t),
                        out_handle: p.out_handle.mix(&q.out_handle, t),
                    })
                    .collect(),
            })
            .collect();
        PathData { subpaths }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_transform_maps_point_unchanged() {
        let t = Transform::default();
        let m = t.matrix(0.0);
        let p = m.transform_point2(Vec2::new(3.0, 4.0));
        assert!((p - Vec2::new(3.0, 4.0)).length() < 1e-6);
    }

    #[test]
    fn translation_then_anchor() {
        let t = Transform {
            position: Property::constant(Vec2::new(100.0, 50.0)),
            anchor: Property::constant(Vec2::new(10.0, 10.0)),
            ..Default::default()
        };
        let m = t.matrix(0.0);
        // the anchor point maps to the position.
        let p = m.transform_point2(Vec2::new(10.0, 10.0));
        assert!((p - Vec2::new(100.0, 50.0)).length() < 1e-5);
    }

    #[test]
    fn rotation_90_degrees() {
        let t = Transform { rotation: Property::constant(90.0), ..Default::default() };
        let m = t.matrix(0.0);
        let p = m.transform_vector2(Vec2::new(1.0, 0.0));
        // +x rotates toward +y (y-down clockwise).
        assert!((p - Vec2::new(0.0, 1.0)).length() < 1e-5, "got {p:?}");
    }

    #[test]
    fn path_morph_compatible() {
        let a = PathData::polygon([Vec2::ZERO, Vec2::new(2.0, 0.0)], true);
        let b = PathData::polygon([Vec2::ZERO, Vec2::new(4.0, 0.0)], true);
        let m = a.mix(&b, 0.5);
        assert_eq!(m.subpaths[0].points[1].anchor, Vec2::new(3.0, 0.0));
    }

    #[test]
    fn path_morph_incompatible_holds() {
        let a = PathData::polygon([Vec2::ZERO], true);
        let b = PathData { subpaths: vec![] };
        assert_eq!(a.mix(&b, 0.5), a);
    }

    #[test]
    fn flatten_straight_polygon() {
        let p = PathData::polygon([Vec2::ZERO, Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)], true);
        let polys = p.flatten();
        assert_eq!(polys.len(), 1);
        assert!(polys[0].closed);
        // straight triangle -> 3 vertices (closing duplicate dropped)
        assert_eq!(polys[0].points.len(), 3);
    }

    #[test]
    fn flatten_curve_subdivides_and_keeps_endpoints() {
        let sp = SubPath {
            closed: false,
            points: vec![
                PathPoint { anchor: Vec2::ZERO, in_handle: Vec2::ZERO, out_handle: Vec2::new(0.0, 12.0) },
                PathPoint { anchor: Vec2::new(10.0, 0.0), in_handle: Vec2::new(0.0, 12.0), out_handle: Vec2::ZERO },
            ],
        };
        let poly = PathData { subpaths: vec![sp] }.flatten();
        assert!(poly[0].points.len() > 10, "curve should subdivide");
        assert_eq!(poly[0].points[0], Vec2::ZERO);
        assert_eq!(*poly[0].points.last().unwrap(), Vec2::new(10.0, 0.0));
    }
}
