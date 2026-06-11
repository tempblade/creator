//! The CPU software rasterizer.
//!
//! Per frame: clear to the background, then walk the render tree back-to-front.
//! Each leaf is rasterized by inverse-mapping supersampled pixel positions into
//! the shape's local space and testing coverage; groups (and effect-bearing
//! leaves) render to an offscreen buffer that is then composited with the
//! node's opacity/blend/effects. Compositing is done in **premultiplied linear
//! light** (PLAN.md §6).

use crate::target::{CpuTarget, RenderTarget};
use crate::{
    NodeContent, Polyline, RenderNode, RenderTree, ResolvedEffect, ResolvedGeometry, ResolvedPaint,
    ResolvedShape,
};
use creator_model::{BlendMode, Color, MatteMode};
use glam::{Affine2, Vec2};
use rayon::prelude::*;

/// Supersampling grid resolution per axis (`SS*SS` samples per pixel).
const SS: u32 = 4;

/// Below this many bounding-box pixels a shape rasterizes serially — rayon's
/// dispatch overhead outweighs the work for tiny shapes.
const PARALLEL_MIN_AREA: usize = 4096;

/// Render a fully-resolved [`RenderTree`] into `target`. The target's size must
/// match the tree's.
pub fn render(tree: &RenderTree, target: &mut CpuTarget) {
    debug_assert_eq!((tree.width, tree.height), (target.width(), target.height()));
    target.clear(tree.background);
    render_node(&tree.root, target, Affine2::IDENTITY);
}

fn render_node(node: &RenderNode, target: &mut CpuTarget, parent_xf: Affine2) {
    let world = parent_xf * node.transform;

    if node.needs_isolation() {
        // Draw the node's content into a transparent offscreen at opacity 1 /
        // Normal blend, run effects on it, then composite as a whole.
        let mut layer = CpuTarget::new(target.width(), target.height());
        draw_content(&node.content, &mut layer, world);
        for fx in &node.effects {
            apply_effect(&mut layer, *fx);
        }
        // Track matte: render the matte node (sharing this node's parent space)
        // to its own buffer, then scale this layer's premultiplied pixels by the
        // per-pixel coverage it defines.
        if let Some((mode, matte_node)) = &node.matte {
            let mut matte = CpuTarget::new(target.width(), target.height());
            render_node(matte_node, &mut matte, parent_xf);
            apply_matte(&mut layer, &matte, *mode);
        }
        composite(target.pixels_mut(), layer.pixels(), node.opacity, node.blend);
    } else {
        // Inline: leaves draw directly with their own opacity/blend; groups
        // recurse (their group-level opacity/blend/effects are all defaults).
        match &node.content {
            NodeContent::Group(children) => {
                for child in children {
                    render_node(child, target, world);
                }
            }
            _ => draw_leaf(&node.content, target, world, node.opacity, node.blend),
        }
    }
}

/// Draw a node's content into `target` at opacity 1 / Normal (used inside an
/// isolated offscreen). Groups recurse; leaves rasterize.
fn draw_content(content: &NodeContent, target: &mut CpuTarget, world: Affine2) {
    match content {
        NodeContent::Group(children) => {
            for child in children {
                render_node(child, target, world);
            }
        }
        _ => draw_leaf(content, target, world, 1.0, BlendMode::Normal),
    }
}

fn draw_leaf(content: &NodeContent, target: &mut CpuTarget, world: Affine2, opacity: f32, blend: BlendMode) {
    match content {
        NodeContent::Shape(shape) => draw_shape(shape, target, world, opacity, blend),
        NodeContent::Text(text) => {
            // Placeholder: a box sized by crude text metrics in the text color.
            let w = text.content.chars().count().max(1) as f32 * text.font_size * 0.55
                + text.tracking * text.content.chars().count() as f32;
            let lines = text.content.split('\n').count().max(1) as f32;
            let h = text.font_size * text.line_height * lines;
            let shape = ResolvedShape {
                geometry: ResolvedGeometry::Rect { size: Vec2::new(w, h), corner_radius: 0.0 },
                fill: Some(ResolvedPaint::Solid(text.color)),
                stroke: None,
            };
            draw_shape(&shape, target, world, opacity, blend);
        }
        NodeContent::Group(_) => unreachable!("groups handled by render_node"),
    }
}

// --- shape rasterization ----------------------------------------------------

fn draw_shape(shape: &ResolvedShape, target: &mut CpuTarget, world: Affine2, opacity: f32, blend: BlendMode) {
    let inv = world.inverse();
    let stroke_half = shape.stroke.map(|s| s.width * 0.5).unwrap_or(0.0);
    let bounds = local_bounds(&shape.geometry, stroke_half);
    let (min, max) = transformed_aabb(world, bounds);

    let (tw, th) = (target.width() as i32, target.height() as i32);
    let x0 = (min.x.floor() as i32).clamp(0, tw);
    let y0 = (min.y.floor() as i32).clamp(0, th);
    let x1 = (max.x.ceil() as i32).clamp(0, tw);
    let y1 = (max.y.ceil() as i32).clamp(0, th);
    if x0 >= x1 || y0 >= y1 {
        return;
    }

    let stroke = shape.stroke;
    let fill = shape.fill.as_ref();
    let geom = &shape.geometry;
    let width = target.width() as usize;
    let step = 1.0 / SS as f32;

    // One row of the bounding box. Each pixel is computed independently and
    // read-modify-writes only its own slot, so rows parallelize with no change
    // in output (per-pixel math is identical regardless of scheduling).
    let raster_row = |y: i32, row: &mut [[f32; 4]]| {
        for x in x0..x1 {
            // Supersample fill and stroke coverage together.
            let mut fill_cov = 0u32;
            let mut stroke_cov = 0u32;
            for sy in 0..SS {
                for sx in 0..SS {
                    let p = Vec2::new(
                        x as f32 + (sx as f32 + 0.5) * step,
                        y as f32 + (sy as f32 + 0.5) * step,
                    );
                    let local = inv.transform_point2(p);
                    if fill.is_some() && inside_fill(geom, local) {
                        fill_cov += 1;
                    }
                    if let Some(s) = stroke {
                        if inside_stroke(geom, local, s.width) {
                            stroke_cov += 1;
                        }
                    }
                }
            }
            let total = (SS * SS) as f32;
            let xi = x as usize;
            // Fill underneath, stroke on top (centered on the edge).
            if let Some(paint) = fill {
                if fill_cov > 0 {
                    // Sample the paint at the pixel center (gradients vary across
                    // the shape; coverage from supersampling handles the edge).
                    let center = inv.transform_point2(Vec2::new(x as f32 + 0.5, y as f32 + 0.5));
                    let c = sample_paint(paint, center);
                    let src = premul_coverage(c, fill_cov as f32 / total, opacity);
                    row[xi] = blend_pixel(row[xi], src, blend);
                }
            }
            if let Some(s) = stroke {
                if stroke_cov > 0 {
                    let src = premul_coverage(s.color, stroke_cov as f32 / total, opacity);
                    row[xi] = blend_pixel(row[xi], src, blend);
                }
            }
        }
    };

    let rows = &mut target.pixels_mut()[(y0 as usize) * width..(y1 as usize) * width];
    let area = ((y1 - y0) as usize) * ((x1 - x0) as usize);
    if area >= PARALLEL_MIN_AREA {
        rows.par_chunks_mut(width)
            .enumerate()
            .for_each(|(i, row)| raster_row(y0 + i as i32, row));
    } else {
        // Tiny shapes: rayon dispatch overhead outweighs the work.
        rows.chunks_mut(width)
            .enumerate()
            .for_each(|(i, row)| raster_row(y0 + i as i32, row));
    }
}

/// Premultiplied source contribution for a covered pixel.
fn premul_coverage(color: Color, coverage: f32, opacity: f32) -> [f32; 4] {
    let a = color.a * coverage * opacity;
    [color.r * a, color.g * a, color.b * a, a]
}

/// Sample a fill paint at a local-space point.
fn sample_paint(paint: &ResolvedPaint, p: Vec2) -> Color {
    match paint {
        ResolvedPaint::Solid(c) => *c,
        ResolvedPaint::Linear { start, end, stops } => {
            let axis = *end - *start;
            let len2 = axis.length_squared();
            let t = if len2 > 0.0 {
                ((p - *start).dot(axis) / len2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            sample_stops(stops, t)
        }
        ResolvedPaint::Radial { center, radius, stops } => {
            let t = if *radius > 0.0 {
                ((p - *center).length() / *radius).clamp(0.0, 1.0)
            } else {
                0.0
            };
            sample_stops(stops, t)
        }
    }
}

/// Interpolate sorted gradient stops at `t ∈ [0,1]`. Stops blend in
/// **premultiplied** linear light — matching the compositing space — so a fade
/// to a transparent stop doesn't pick up a dark fringe (the "transparent-black"
/// artifact you get from straight-alpha interpolation toward `(0,0,0,0)`).
fn sample_stops(stops: &[(f32, Color)], t: f32) -> Color {
    if stops.is_empty() {
        return Color::WHITE;
    }
    if t <= stops[0].0 {
        return stops[0].1;
    }
    let last = stops[stops.len() - 1];
    if t >= last.0 {
        return last.1;
    }
    for w in stops.windows(2) {
        let (o0, c0) = w[0];
        let (o1, c1) = w[1];
        if t >= o0 && t <= o1 {
            let span = (o1 - o0).max(1e-6);
            let u = (t - o0) / span;
            return lerp_premul(c0, c1, u);
        }
    }
    last.1
}

/// Blend two colors in premultiplied linear space, returning a straight-alpha
/// color (so the caller's `premul_coverage` re-premultiplies consistently).
fn lerp_premul(c0: Color, c1: Color, u: f32) -> Color {
    let p0 = c0.premultiplied();
    let p1 = c1.premultiplied();
    let lerp = |a: f32, b: f32| a + (b - a) * u;
    let a = lerp(p0[3], p1[3]);
    if a > 0.0 {
        Color::linear(lerp(p0[0], p1[0]) / a, lerp(p0[1], p1[1]) / a, lerp(p0[2], p1[2]) / a, a)
    } else {
        Color::TRANSPARENT
    }
}

/// Local-space axis-aligned bounds of a geometry, expanded by `stroke_half`.
fn local_bounds(geom: &ResolvedGeometry, stroke_half: f32) -> (Vec2, Vec2) {
    match geom {
        ResolvedGeometry::Rect { size, .. } | ResolvedGeometry::Ellipse { size } => {
            let h = *size * 0.5 + Vec2::splat(stroke_half);
            (-h, h)
        }
        ResolvedGeometry::Path { polylines } => {
            let mut min = Vec2::splat(f32::INFINITY);
            let mut max = Vec2::splat(f32::NEG_INFINITY);
            for poly in polylines {
                for p in &poly.points {
                    min = min.min(*p);
                    max = max.max(*p);
                }
            }
            if !min.is_finite() {
                return (Vec2::ZERO, Vec2::ZERO);
            }
            let e = Vec2::splat(stroke_half);
            (min - e, max + e)
        }
    }
}

/// AABB of `bounds` after applying `world` (transform all four corners).
fn transformed_aabb(world: Affine2, bounds: (Vec2, Vec2)) -> (Vec2, Vec2) {
    let (lo, hi) = bounds;
    let corners = [
        Vec2::new(lo.x, lo.y),
        Vec2::new(hi.x, lo.y),
        Vec2::new(lo.x, hi.y),
        Vec2::new(hi.x, hi.y),
    ];
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for c in corners {
        let p = world.transform_point2(c);
        min = min.min(p);
        max = max.max(p);
    }
    (min, max)
}

fn inside_fill(geom: &ResolvedGeometry, p: Vec2) -> bool {
    match geom {
        ResolvedGeometry::Rect { size, corner_radius } => {
            inside_round_rect(p, *size, *corner_radius)
        }
        ResolvedGeometry::Ellipse { size } => inside_ellipse(p, *size),
        ResolvedGeometry::Path { polylines } => point_in_path(polylines, p),
    }
}

fn inside_stroke(geom: &ResolvedGeometry, p: Vec2, width: f32) -> bool {
    let half = width * 0.5;
    match geom {
        ResolvedGeometry::Rect { size, corner_radius } => {
            let outer = *size + Vec2::splat(width);
            let inner = (*size - Vec2::splat(width)).max(Vec2::ZERO);
            inside_round_rect(p, outer, corner_radius + half)
                && !inside_round_rect(p, inner, (corner_radius - half).max(0.0))
        }
        ResolvedGeometry::Ellipse { size } => {
            let outer = *size + Vec2::splat(width);
            let inner = (*size - Vec2::splat(width)).max(Vec2::ZERO);
            inside_ellipse(p, outer) && !inside_ellipse(p, inner)
        }
        ResolvedGeometry::Path { polylines } => dist_to_path(polylines, p) <= half,
    }
}

/// Inside a rounded rectangle centered at the origin with the given full `size`.
fn inside_round_rect(p: Vec2, size: Vec2, radius: f32) -> bool {
    let h = size * 0.5;
    if p.x.abs() > h.x || p.y.abs() > h.y {
        return false;
    }
    let r = radius.min(h.x).min(h.y).max(0.0);
    if r <= 0.0 {
        return true;
    }
    // Inside the cross region always; in the corner squares, test the arc.
    let inner = h - Vec2::splat(r);
    let dx = p.x.abs() - inner.x;
    let dy = p.y.abs() - inner.y;
    if dx <= 0.0 || dy <= 0.0 {
        return true;
    }
    dx * dx + dy * dy <= r * r
}

/// Inside an ellipse inscribed in `size`, centered at the origin.
fn inside_ellipse(p: Vec2, size: Vec2) -> bool {
    let r = size * 0.5;
    if r.x <= 0.0 || r.y <= 0.0 {
        return false;
    }
    let nx = p.x / r.x;
    let ny = p.y / r.y;
    nx * nx + ny * ny <= 1.0
}

/// Even-odd point-in-path test over all (flattened) subpaths, closed implicitly.
fn point_in_path(polylines: &[Polyline], p: Vec2) -> bool {
    let mut inside = false;
    for poly in polylines {
        let n = poly.points.len();
        if n < 3 {
            continue;
        }
        let mut j = n - 1;
        for i in 0..n {
            let a = poly.points[i];
            let b = poly.points[j];
            if (a.y > p.y) != (b.y > p.y) {
                let t = (p.y - a.y) / (b.y - a.y);
                let x_cross = a.x + t * (b.x - a.x);
                if p.x < x_cross {
                    inside = !inside;
                }
            }
            j = i;
        }
    }
    inside
}

/// Minimum distance from `p` to any path segment.
fn dist_to_path(polylines: &[Polyline], p: Vec2) -> f32 {
    let mut best = f32::INFINITY;
    for poly in polylines {
        let n = poly.points.len();
        if n < 2 {
            continue;
        }
        let last = if poly.closed { n } else { n - 1 };
        for i in 0..last {
            let a = poly.points[i];
            let b = poly.points[(i + 1) % n];
            best = best.min(dist_point_segment(p, a, b));
        }
    }
    best
}

fn dist_point_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len2 = ab.length_squared();
    if len2 <= 1e-12 {
        return (p - a).length();
    }
    let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
    (p - (a + ab * t)).length()
}

// --- compositing ------------------------------------------------------------

/// Composite a full premultiplied `src` layer onto `dst` with `opacity`/`blend`.
fn composite(dst: &mut [[f32; 4]], src: &[[f32; 4]], opacity: f32, blend: BlendMode) {
    dst.par_iter_mut().zip(src.par_iter()).for_each(|(d, s)| {
        // Scaling a premultiplied pixel by opacity scales all four channels.
        let s = [s[0] * opacity, s[1] * opacity, s[2] * opacity, s[3] * opacity];
        *d = blend_pixel(*d, s, blend);
    });
}

/// Blend premultiplied `src` over premultiplied `dst` using `mode`.
/// Implements the W3C compositing-and-blending model in premultiplied space.
fn blend_pixel(dst: [f32; 4], src: [f32; 4], mode: BlendMode) -> [f32; 4] {
    let (cb, ab) = ([dst[0], dst[1], dst[2]], dst[3]);
    let (cs, a_s) = ([src[0], src[1], src[2]], src[3]);

    match mode {
        BlendMode::Normal => [
            cs[0] + cb[0] * (1.0 - a_s),
            cs[1] + cb[1] * (1.0 - a_s),
            cs[2] + cb[2] * (1.0 - a_s),
            a_s + ab * (1.0 - a_s),
        ],
        BlendMode::Add => [
            cs[0] + cb[0],
            cs[1] + cb[1],
            cs[2] + cb[2],
            (a_s + ab * (1.0 - a_s)).min(1.0),
        ],
        _ => {
            // Separable blend: B(Cb,Cs) on straight colors, recombined with the
            // general formula co = (1−ab)·cs + (1−as)·cb + as·ab·B.
            let cb_s = unpremul(cb, ab);
            let cs_s = unpremul(cs, a_s);
            let ao = a_s + ab * (1.0 - a_s);
            let mut out = [0.0; 4];
            for i in 0..3 {
                let b = separable(mode, cb_s[i], cs_s[i]);
                out[i] = (1.0 - ab) * cs[i] + (1.0 - a_s) * cb[i] + a_s * ab * b;
            }
            out[3] = ao;
            out
        }
    }
}

fn unpremul(c: [f32; 3], a: f32) -> [f32; 3] {
    if a > 0.0 {
        [c[0] / a, c[1] / a, c[2] / a]
    } else {
        [0.0; 3]
    }
}

/// Separable blend functions on straight (un-premultiplied) channel values.
fn separable(mode: BlendMode, cb: f32, cs: f32) -> f32 {
    match mode {
        BlendMode::Multiply => cb * cs,
        BlendMode::Screen => cb + cs - cb * cs,
        BlendMode::Darken => cb.min(cs),
        BlendMode::Lighten => cb.max(cs),
        BlendMode::Overlay => {
            if cb <= 0.5 {
                2.0 * cb * cs
            } else {
                1.0 - 2.0 * (1.0 - cb) * (1.0 - cs)
            }
        }
        // Normal/Add handled before reaching here.
        BlendMode::Normal | BlendMode::Add => cs,
    }
}

// --- track mattes -----------------------------------------------------------

/// Scale `layer`'s premultiplied pixels by the coverage defined by `matte`.
///
/// Alpha modes read the matte's alpha; luma modes read the **premultiplied**
/// linear Rec.709 luminance, which equals `luma(straight) × alpha` — i.e. a
/// dark or transparent matte pixel both reduce coverage, matching AE.
fn apply_matte(layer: &mut CpuTarget, matte: &CpuTarget, mode: MatteMode) {
    layer.pixels_mut().par_iter_mut().zip(matte.pixels().par_iter()).for_each(|(px, m)| {
        let coverage = match mode {
            MatteMode::Alpha => m[3],
            MatteMode::AlphaInverted => 1.0 - m[3],
            MatteMode::Luma => luma_premul(m),
            MatteMode::LumaInverted => 1.0 - luma_premul(m),
        }
        .clamp(0.0, 1.0);
        px[0] *= coverage;
        px[1] *= coverage;
        px[2] *= coverage;
        px[3] *= coverage;
    });
}

/// Linear-light Rec.709 luminance of a premultiplied pixel.
#[inline]
fn luma_premul(p: &[f32; 4]) -> f32 {
    0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2]
}

// --- effects ----------------------------------------------------------------

fn apply_effect(target: &mut CpuTarget, fx: ResolvedEffect) {
    match fx {
        ResolvedEffect::GaussianBlur { radius } => {
            if radius > 0.0 {
                box_blur(target, radius);
            }
        }
        ResolvedEffect::Tint { color, amount } => tint(target, color, amount),
    }
}

/// Three box-blur passes approximate a Gaussian. `radius` is the box half-width
/// in pixels. Operates on premultiplied pixels (so the alpha channel blurs
/// correctly). Edges clamp.
fn box_blur(target: &mut CpuTarget, radius: f32) {
    // No `.max(1.0)` floor: a sub-pixel radius must approach identity, not snap
    // to a full 1px blur (which made small/animated radii over-blur and jump).
    let r = radius.round() as i32;
    if r < 1 {
        return;
    }
    for _ in 0..3 {
        box_blur_axis(target, r, true);
        box_blur_axis(target, r, false);
    }
}

fn box_blur_axis(target: &mut CpuTarget, r: i32, horizontal: bool) {
    let w = target.width() as i32;
    let h = target.height() as i32;
    let src = target.pixels().to_vec();
    let dst = target.pixels_mut();
    let norm = 1.0 / (2 * r + 1) as f32;
    // Output rows are independent (they read only the `src` snapshot), so they
    // parallelize. The per-pixel window sum runs in a fixed order, keeping the
    // result bit-identical to the serial pass.
    dst.par_chunks_mut(w as usize).enumerate().for_each(|(row_idx, row)| {
        let y = row_idx as i32;
        for (xi, out) in row.iter_mut().enumerate() {
            let x = xi as i32;
            let mut acc = [0.0f32; 4];
            for k in -r..=r {
                let (sx, sy) = if horizontal {
                    ((x + k).clamp(0, w - 1), y)
                } else {
                    (x, (y + k).clamp(0, h - 1))
                };
                let s = src[(sy as usize) * (w as usize) + sx as usize];
                acc[0] += s[0];
                acc[1] += s[1];
                acc[2] += s[2];
                acc[3] += s[3];
            }
            *out = [acc[0] * norm, acc[1] * norm, acc[2] * norm, acc[3] * norm];
        }
    });
}

/// Blend every pixel's straight color toward `color` by `amount`, preserving
/// alpha.
fn tint(target: &mut CpuTarget, color: Color, amount: f32) {
    let a = amount.clamp(0.0, 1.0);
    target.pixels_mut().par_iter_mut().for_each(|px| {
        let alpha = px[3];
        if alpha <= 0.0 {
            return;
        }
        let straight = [px[0] / alpha, px[1] / alpha, px[2] / alpha];
        let r = straight[0] + (color.r - straight[0]) * a;
        let g = straight[1] + (color.g - straight[1]) * a;
        let b = straight[2] + (color.b - straight[2]) * a;
        *px = [r * alpha, g * alpha, b * alpha, alpha];
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NodeContent, RenderNode, ResolvedGeometry, ResolvedShape};

    fn solid_rect_tree(bg: Color, fill: Color) -> RenderTree {
        let mut tree = RenderTree::new(10, 10, bg);
        let shape = ResolvedShape {
            geometry: ResolvedGeometry::Rect { size: Vec2::new(6.0, 6.0), corner_radius: 0.0 },
            fill: Some(ResolvedPaint::Solid(fill)),
            stroke: None,
        };
        let node = RenderNode::leaf(NodeContent::Shape(shape))
            .with_transform(Affine2::from_translation(Vec2::new(5.0, 5.0)));
        tree.root = RenderNode::group(vec![node]);
        tree
    }

    #[test]
    fn centered_rect_fills_center_not_corner() {
        let tree = solid_rect_tree(Color::TRANSPARENT, Color::WHITE);
        let mut t = CpuTarget::new(10, 10);
        render(&tree, &mut t);
        // Center pixel fully covered.
        let c = t.pixel(5, 5);
        assert!(c[3] > 0.99, "center alpha {}", c[3]);
        // Far corner outside the 6x6 rect centered at (5,5).
        let corner = t.pixel(0, 0);
        assert!(corner[3] < 0.01, "corner alpha {}", corner[3]);
    }

    #[test]
    fn opacity_halves_alpha() {
        let mut tree = solid_rect_tree(Color::TRANSPARENT, Color::WHITE);
        if let NodeContent::Group(children) = &mut tree.root.content {
            children[0].opacity = 0.5;
        }
        let mut t = CpuTarget::new(10, 10);
        render(&tree, &mut t);
        let c = t.pixel(5, 5);
        assert!((c[3] - 0.5).abs() < 0.02, "alpha {}", c[3]);
    }

    #[test]
    fn antialiased_edge_has_partial_coverage() {
        // Rotate so an edge crosses pixels diagonally -> fractional coverage.
        let mut tree = solid_rect_tree(Color::TRANSPARENT, Color::WHITE);
        if let NodeContent::Group(children) = &mut tree.root.content {
            children[0].transform =
                Affine2::from_translation(Vec2::new(5.0, 5.0)) * Affine2::from_angle(0.5);
        }
        let mut t = CpuTarget::new(10, 10);
        render(&tree, &mut t);
        let any_partial = (0..100).any(|i| {
            let a = t.pixel((i % 10) as u32, (i / 10) as u32)[3];
            a > 0.05 && a < 0.95
        });
        assert!(any_partial, "expected an anti-aliased edge pixel");
    }

    #[test]
    fn multiply_blend_darkens() {
        // Red over green with Multiply => near black (premultiplied, opaque).
        let mut tree = RenderTree::new(4, 4, Color::linear(0.0, 1.0, 0.0, 1.0));
        let shape = ResolvedShape {
            geometry: ResolvedGeometry::Rect { size: Vec2::new(8.0, 8.0), corner_radius: 0.0 },
            fill: Some(ResolvedPaint::Solid(Color::linear(1.0, 0.0, 0.0, 1.0))),
            stroke: None,
        };
        let node = RenderNode::leaf(NodeContent::Shape(shape))
            .with_transform(Affine2::from_translation(Vec2::new(2.0, 2.0)))
            .with_blend(BlendMode::Multiply);
        tree.root = RenderNode::group(vec![node]);
        let mut t = CpuTarget::new(4, 4);
        render(&tree, &mut t);
        let c = t.pixel(2, 2);
        assert!(c[0] < 0.01 && c[1] < 0.01 && c[2] < 0.01, "got {c:?}");
        assert!((c[3] - 1.0).abs() < 0.01);
    }

    #[test]
    fn fill_stroke_leaf_with_blend_isolates() {
        use crate::ResolvedStroke;
        let with_both = ResolvedShape {
            geometry: ResolvedGeometry::Rect { size: Vec2::new(6.0, 6.0), corner_radius: 0.0 },
            fill: Some(ResolvedPaint::Solid(Color::WHITE)),
            stroke: Some(ResolvedStroke { color: Color::BLACK, width: 2.0 }),
        };
        let node = RenderNode::leaf(NodeContent::Shape(with_both)).with_blend(BlendMode::Multiply);
        assert!(node.needs_isolation(), "fill+stroke + non-Normal blend must isolate");

        // Single paint (fill only) is identical inline vs isolated -> no isolation.
        let single = ResolvedShape {
            geometry: ResolvedGeometry::Rect { size: Vec2::new(6.0, 6.0), corner_radius: 0.0 },
            fill: Some(ResolvedPaint::Solid(Color::WHITE)),
            stroke: None,
        };
        let node2 = RenderNode::leaf(NodeContent::Shape(single)).with_blend(BlendMode::Multiply);
        assert!(!node2.needs_isolation());
    }

    #[test]
    fn subpixel_blur_is_identity_but_real_blur_changes() {
        let tree = solid_rect_tree(Color::TRANSPARENT, Color::WHITE);
        let mut t = CpuTarget::new(10, 10);
        render(&tree, &mut t);
        let before = t.to_srgba8();
        super::box_blur(&mut t, 0.3); // r rounds to 0 -> no-op
        assert_eq!(before, t.to_srgba8(), "sub-pixel blur must be identity");

        let mut t2 = CpuTarget::new(10, 10);
        render(&tree, &mut t2);
        let b2 = t2.to_srgba8();
        super::box_blur(&mut t2, 3.0);
        assert_ne!(b2, t2.to_srgba8(), "a real radius must change the image");
    }

    #[test]
    fn linear_gradient_varies_across_shape() {
        let mut tree = RenderTree::new(20, 4, Color::TRANSPARENT);
        let shape = ResolvedShape {
            geometry: ResolvedGeometry::Rect { size: Vec2::new(20.0, 4.0), corner_radius: 0.0 },
            fill: Some(ResolvedPaint::Linear {
                start: Vec2::new(-10.0, 0.0),
                end: Vec2::new(10.0, 0.0),
                stops: vec![
                    (0.0, Color::linear(1.0, 0.0, 0.0, 1.0)),
                    (1.0, Color::linear(0.0, 0.0, 1.0, 1.0)),
                ],
            }),
            stroke: None,
        };
        let node = RenderNode::leaf(NodeContent::Shape(shape))
            .with_transform(Affine2::from_translation(Vec2::new(10.0, 2.0)));
        tree.root = RenderNode::group(vec![node]);
        let mut t = CpuTarget::new(20, 4);
        render(&tree, &mut t);
        let left = t.pixel(1, 2);
        let right = t.pixel(18, 2);
        assert!(left[0] > left[2], "left end should be reddish: {left:?}");
        assert!(right[2] > right[0], "right end should be bluish: {right:?}");
    }

    #[test]
    fn gradient_fade_to_transparent_stays_white() {
        // White → transparent gradient must not darken near the transparent end
        // (premultiplied interpolation; straight interp would gray it out).
        let mut tree = RenderTree::new(40, 4, Color::TRANSPARENT);
        let shape = ResolvedShape {
            geometry: ResolvedGeometry::Rect { size: Vec2::new(40.0, 4.0), corner_radius: 0.0 },
            fill: Some(ResolvedPaint::Linear {
                start: Vec2::new(-20.0, 0.0),
                end: Vec2::new(20.0, 0.0),
                stops: vec![(0.0, Color::WHITE), (1.0, Color::TRANSPARENT)],
            }),
            stroke: None,
        };
        let node = RenderNode::leaf(NodeContent::Shape(shape))
            .with_transform(Affine2::from_translation(Vec2::new(20.0, 2.0)));
        tree.root = RenderNode::group(vec![node]);
        let mut t = CpuTarget::new(40, 4);
        render(&tree, &mut t);
        let p = t.pixel(20, 2);
        assert!(p[3] > 0.1 && p[3] < 0.9, "expected partial alpha midpoint, got {}", p[3]);
        // straight color stays white: premultiplied r/a ≈ 1 (≈0.5 with the bug).
        assert!((p[0] / p[3] - 1.0).abs() < 0.05, "fade-to-transparent darkened: r/a={}", p[0] / p[3]);
    }

    #[test]
    fn radial_gradient_center_brighter_than_edge() {
        let mut tree = RenderTree::new(20, 20, Color::TRANSPARENT);
        let shape = ResolvedShape {
            geometry: ResolvedGeometry::Ellipse { size: Vec2::new(20.0, 20.0) },
            fill: Some(ResolvedPaint::Radial {
                center: Vec2::ZERO,
                radius: 10.0,
                stops: vec![(0.0, Color::WHITE), (1.0, Color::linear(0.0, 0.0, 0.0, 1.0))],
            }),
            stroke: None,
        };
        let node = RenderNode::leaf(NodeContent::Shape(shape))
            .with_transform(Affine2::from_translation(Vec2::new(10.0, 10.0)));
        tree.root = RenderNode::group(vec![node]);
        let mut t = CpuTarget::new(20, 20);
        render(&tree, &mut t);
        assert!(t.pixel(10, 10)[0] > t.pixel(10, 18)[0], "center brighter than edge");
    }

    #[test]
    fn path_polygon_fills_interior() {
        // A flattened triangle path fills inside, not outside.
        let polylines = creator_model::PathData::polygon(
            [Vec2::new(-10.0, -10.0), Vec2::new(10.0, -10.0), Vec2::new(0.0, 10.0)],
            true,
        )
        .flatten();
        let shape = ResolvedShape {
            geometry: ResolvedGeometry::Path { polylines },
            fill: Some(ResolvedPaint::Solid(Color::WHITE)),
            stroke: None,
        };
        let node = RenderNode::leaf(NodeContent::Shape(shape))
            .with_transform(Affine2::from_translation(Vec2::new(15.0, 15.0)));
        let mut tree = RenderTree::new(30, 30, Color::TRANSPARENT);
        tree.root = RenderNode::group(vec![node]);
        let mut t = CpuTarget::new(30, 30);
        render(&tree, &mut t);
        assert!(t.pixel(15, 12)[3] > 0.9, "interior should be filled");
        assert!(t.pixel(2, 2)[3] < 0.05, "exterior should be empty");
    }

    #[test]
    fn background_shows_through_transparency() {
        let tree = solid_rect_tree(Color::linear(0.2, 0.2, 0.2, 1.0), Color::TRANSPARENT);
        let mut t = CpuTarget::new(10, 10);
        render(&tree, &mut t);
        // Nothing drawn (transparent fill) -> background everywhere.
        let c = t.pixel(0, 0);
        assert!((c[0] - 0.2).abs() < 1e-4 && (c[3] - 1.0).abs() < 1e-4);
    }
}
