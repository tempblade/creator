//! `eval(scene, t) -> render tree`.
//!
//! Pure and deterministic: the output depends only on `(project, comp, time)`.
//! This is the contract that makes the timeline randomly scrubbable and the
//! frame cache sound (PLAN.md §5, §11). Parenting is flattened into each layer's
//! world transform; precomps recurse into nested compositions (with cycle
//! guards).

use creator_model::{
    Color, Composition, CompId, EffectKind, Geometry, Layer, LayerId, LayerKind, Paint, Project,
};
use creator_render::{
    NodeContent, RenderNode, RenderTree, ResolvedEffect, ResolvedGeometry, ResolvedPaint,
    ResolvedShape, ResolvedStroke, ResolvedText,
};
use glam::Affine2;

/// Flatten `comp` at `time` into a fully-resolved [`RenderTree`].
pub fn eval(project: &Project, comp_id: CompId, time: f64) -> RenderTree {
    let comp = match project.composition(comp_id) {
        Some(c) => c,
        None => return RenderTree::new(1, 1, creator_render::Color::TRANSPARENT),
    };
    let mut visited = Vec::new();
    let root = eval_comp_node(project, comp_id, time, &mut visited);
    RenderTree { width: comp.width, height: comp.height, background: comp.background, root }
}

/// A composition flattened to a group node of its visible layers (no
/// background fill — that belongs to the top-level [`RenderTree`], and precomps
/// composite transparently).
fn eval_comp_node(
    project: &Project,
    comp_id: CompId,
    time: f64,
    visited: &mut Vec<CompId>,
) -> RenderNode {
    if visited.contains(&comp_id) {
        // Cyclic precomp reference: stop here.
        return RenderNode::group(Vec::new());
    }
    let comp = match project.composition(comp_id) {
        Some(c) => c,
        None => return RenderNode::group(Vec::new()),
    };
    visited.push(comp_id);
    // A layer with a track matte consumes the layer ABOVE it in the stack (the
    // next entry in back-to-front order, AE convention) as its matte source;
    // the source is not drawn normally.
    let consumed_as_matte: Vec<bool> = comp
        .order
        .iter()
        .enumerate()
        .map(|(i, _)| {
            i > 0
                && comp
                    .order
                    .get(i - 1)
                    .and_then(|&below| comp.layer(below))
                    .is_some_and(|l| l.matte.is_some())
        })
        .collect();

    let mut children = Vec::new();
    for (i, &consumed) in consumed_as_matte.iter().enumerate() {
        if consumed {
            continue;
        }
        if let Some(node) = build_node_with_matte(project, comp, i, time, visited) {
            children.push(node);
        }
    }
    visited.pop();
    RenderNode::group(children)
}

/// Build the node for `comp.order[i]` and, if it has a matte, attach the layer
/// above as its source — recursively, so a matte source that is itself matted
/// is shaped by *its* source in turn (AE matte chains; `consumed_as_matte`
/// already marks the whole chain). The index strictly increases, so recursion
/// terminates at the top of the stack. If a source can't be built (missing,
/// Null, hidden at `time`), the matte buffer stays empty: Alpha/Luma hide the
/// layer, inverted modes show it fully.
fn build_node_with_matte(
    project: &Project,
    comp: &Composition,
    i: usize,
    time: f64,
    visited: &mut Vec<CompId>,
) -> Option<RenderNode> {
    let &id = comp.order.get(i)?;
    let layer = comp.layer(id)?;
    if !layer.visible_at(time) {
        return None;
    }
    let mut node = build_layer_node(project, comp, id, layer, time, visited)?;
    if let Some(mode) = layer.matte {
        let matte_node = build_node_with_matte(project, comp, i + 1, time, visited)
            .unwrap_or_else(|| RenderNode::group(Vec::new()));
        node.matte = Some((mode, Box::new(matte_node)));
    }
    Some(node)
}

fn build_layer_node(
    project: &Project,
    comp: &Composition,
    id: LayerId,
    layer: &Layer,
    time: f64,
    visited: &mut Vec<CompId>,
) -> Option<RenderNode> {
    let content = match &layer.kind {
        LayerKind::Null => return None, // transform-only; contributes via parenting
        LayerKind::Shape(shape) => NodeContent::Shape(resolve_shape(shape, time)),
        LayerKind::Text(text) => NodeContent::Text(ResolvedText {
            content: text.content.clone(),
            font_size: text.font_size.eval(time) as f32,
            color: text.color.eval(time),
            tracking: text.tracking.eval(time) as f32,
            line_height: text.line_height.eval(time) as f32,
        }),
        LayerKind::Precomp(nested) => {
            let group = eval_comp_node(project, *nested, time, visited);
            group.content // a Group(Vec<RenderNode>)
        }
    };

    let world = layer_world(comp, id, time, &mut Vec::new());
    let effects = layer
        .effects
        .iter()
        .filter(|e| e.enabled)
        .map(|e| resolve_effect(&e.kind, time))
        .collect();

    Some(RenderNode {
        transform: world,
        opacity: layer.opacity.eval(time).clamp(0.0, 1.0) as f32,
        blend: layer.blend,
        effects,
        matte: None, // attached by the caller (it pairs stack neighbors)
        content,
    })
}

/// World (comp-space) transform of a layer, composing its parent chain:
/// `world(child) = world(parent) · local(child)`.
///
/// `ancestors` is the chain currently being resolved; if a parent pointer would
/// revisit a layer already in the chain (a cycle of any length, including a
/// self-parent), the chain is broken at that point — so a cyclic `parent` graph
/// yields a bounded, well-defined transform instead of garbage, and never
/// recurses without end.
fn layer_world(comp: &Composition, id: LayerId, time: f64, ancestors: &mut Vec<LayerId>) -> Affine2 {
    let layer = match comp.layer(id) {
        Some(l) => l,
        None => return Affine2::IDENTITY,
    };
    let local = layer.transform.matrix(time);
    if let Some(p) = layer.parent {
        if p != id && !ancestors.contains(&p) {
            ancestors.push(id);
            let parent_world = layer_world(comp, p, time, ancestors);
            ancestors.pop();
            return parent_world * local;
        }
    }
    local
}

fn resolve_shape(shape: &creator_model::Shape, time: f64) -> ResolvedShape {
    let geometry = match &shape.geometry {
        Geometry::Rect { size, corner_radius } => ResolvedGeometry::Rect {
            size: size.eval(time),
            corner_radius: corner_radius.eval(time) as f32,
        },
        Geometry::Ellipse { size } => ResolvedGeometry::Ellipse { size: size.eval(time) },
        Geometry::Path { data } => ResolvedGeometry::Path { polylines: data.eval(time).flatten() },
    };
    ResolvedShape {
        geometry,
        fill: shape.fill.as_ref().map(|f| resolve_paint(&f.paint, time)),
        stroke: shape.stroke.as_ref().map(|s| ResolvedStroke {
            color: s.color.eval(time),
            width: s.width.eval(time) as f32,
        }),
    }
}

fn resolve_paint(paint: &Paint, time: f64) -> ResolvedPaint {
    match paint {
        Paint::Solid(c) => ResolvedPaint::Solid(c.eval(time)),
        Paint::LinearGradient { start, end, stops } => ResolvedPaint::Linear {
            start: start.eval(time),
            end: end.eval(time),
            stops: resolve_stops(stops, time),
        },
        Paint::RadialGradient { center, radius, stops } => ResolvedPaint::Radial {
            center: center.eval(time),
            radius: radius.eval(time) as f32,
            stops: resolve_stops(stops, time),
        },
    }
}

/// Resolve and sort gradient stops by offset (the rasterizer assumes ascending).
fn resolve_stops(stops: &[creator_model::GradientStop], time: f64) -> Vec<(f32, Color)> {
    let mut v: Vec<(f32, Color)> = stops.iter().map(|s| (s.offset, s.color.eval(time))).collect();
    v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    v
}

fn resolve_effect(kind: &EffectKind, time: f64) -> ResolvedEffect {
    match kind {
        EffectKind::GaussianBlur { radius } => {
            ResolvedEffect::GaussianBlur { radius: radius.eval(time) as f32 }
        }
        EffectKind::Tint { color, amount } => ResolvedEffect::Tint {
            color: color.eval(time),
            amount: amount.eval(time) as f32,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use creator_model::{
        BlendMode, Color, Composition, Keyframe, Layer, LayerKind, Project, Property, Shape, Vec2,
    };
    use creator_render::{render, CpuTarget};

    fn moving_box() -> (Project, CompId) {
        let mut project = Project::new("t");
        let mut comp = Composition::new("main", 100, 100, 30.0, 2.0);
        comp.background = Color::BLACK;
        let mut layer = Layer::new(
            "box",
            LayerKind::Shape(Shape::rect(Vec2::new(20.0, 20.0), Color::WHITE)),
        );
        layer.transform.position = Property::animated(vec![
            Keyframe::new(0.0, Vec2::new(10.0, 50.0)),
            Keyframe::new(2.0, Vec2::new(90.0, 50.0)),
        ]);
        comp.add_layer(layer);
        let id = project.add_composition(comp);
        (project, id)
    }

    #[test]
    fn eval_is_deterministic() {
        let (project, id) = moving_box();
        let a = eval(&project, id, 1.0);
        let b = eval(&project, id, 1.0);
        let mut ta = CpuTarget::new(100, 100);
        let mut tb = CpuTarget::new(100, 100);
        render(&a, &mut ta);
        render(&b, &mut tb);
        assert_eq!(ta.to_srgba8(), tb.to_srgba8());
    }

    #[test]
    fn animation_moves_the_box() {
        let (project, id) = moving_box();
        // at t=0 the box is at x=10; at t=2 it's at x=90.
        let mut t0 = CpuTarget::new(100, 100);
        render(&eval(&project, id, 0.0), &mut t0);
        let mut t2 = CpuTarget::new(100, 100);
        render(&eval(&project, id, 2.0), &mut t2);
        // left side lit at t0, dark at t2 (and vice versa).
        assert!(t0.pixel(10, 50)[0] > 0.5);
        assert!(t2.pixel(10, 50)[0] < 0.5);
        assert!(t2.pixel(90, 50)[0] > 0.5);
    }

    #[test]
    fn parenting_inherits_transform() {
        let mut project = Project::new("t");
        let mut comp = Composition::new("main", 100, 100, 30.0, 1.0);
        // parent null translated by (50,50)
        let mut parent = Layer::new("null", LayerKind::Null);
        parent.transform.position = Property::constant(Vec2::new(50.0, 50.0));
        let pid = comp.add_layer(parent);
        // child box at local (0,0), parented -> ends up at (50,50)
        let mut child = Layer::new("box", LayerKind::Shape(Shape::rect(Vec2::new(10.0, 10.0), Color::WHITE)));
        child.parent = Some(pid);
        comp.add_layer(child);
        let id = project.add_composition(comp);
        let mut t = CpuTarget::new(100, 100);
        render(&eval(&project, id, 0.0), &mut t);
        assert!(t.pixel(50, 50)[3] > 0.9, "child should render at parent origin");
        assert!(t.pixel(5, 5)[3] < 0.1, "nothing at local origin");
    }

    /// A 40x20 comp: full-canvas white box matted by a 20x20 rect covering only
    /// the LEFT half, with the matte mode under test.
    fn matte_comp(mode: creator_model::MatteMode, matte_fill: Color) -> (Project, CompId) {
        let mut project = Project::new("t");
        let mut comp = Composition::new("m", 40, 20, 30.0, 1.0);
        // matted layer: white, covers everything
        let mut content = Layer::new(
            "content",
            LayerKind::Shape(Shape::rect(Vec2::new(40.0, 20.0), Color::WHITE)),
        );
        content.transform.position = Property::constant(Vec2::new(20.0, 10.0));
        content.matte = Some(mode);
        comp.add_layer(content);
        // matte source ABOVE it: covers the left half only
        let mut matte = Layer::new(
            "matte",
            LayerKind::Shape(Shape::rect(Vec2::new(20.0, 20.0), matte_fill)),
        );
        matte.transform.position = Property::constant(Vec2::new(10.0, 10.0));
        comp.add_layer(matte);
        let id = project.add_composition(comp);
        (project, id)
    }

    #[test]
    fn alpha_matte_clips_to_matte_coverage() {
        let (project, id) = matte_comp(creator_model::MatteMode::Alpha, Color::WHITE);
        let t = crate::render_at(&project, id, 0.0);
        assert!(t.pixel(5, 10)[3] > 0.9, "inside matte: visible");
        assert!(t.pixel(35, 10)[3] < 0.05, "outside matte: clipped");
    }

    #[test]
    fn alpha_inverted_matte_shows_complement() {
        let (project, id) = matte_comp(creator_model::MatteMode::AlphaInverted, Color::WHITE);
        let t = crate::render_at(&project, id, 0.0);
        assert!(t.pixel(5, 10)[3] < 0.05, "inside matte: hidden");
        assert!(t.pixel(35, 10)[3] > 0.9, "outside matte: visible");
    }

    #[test]
    fn luma_matte_weights_by_brightness() {
        // A mid-gray (linear 0.5) matte passes ~50% coverage; alpha matte would
        // pass 100% since the gray is opaque.
        let gray = Color::linear(0.5, 0.5, 0.5, 1.0);
        let (project, id) = matte_comp(creator_model::MatteMode::Luma, gray);
        let t = crate::render_at(&project, id, 0.0);
        let a = t.pixel(5, 10)[3];
        assert!((a - 0.5).abs() < 0.05, "luma 0.5 -> ~50% coverage, got {a}");
        assert!(t.pixel(35, 10)[3] < 0.05, "outside matte: clipped");
    }

    #[test]
    fn matte_chain_intersects_coverage() {
        // L0 (white, full canvas) matted by L1 (left half), which is itself
        // matted by L2 (top half). Chains must compose: L0 shows only in the
        // intersection (top-left quadrant), and L2 must not vanish silently.
        let mut project = Project::new("t");
        let mut comp = Composition::new("m", 40, 20, 30.0, 1.0);
        let mut l0 = Layer::new("content", LayerKind::Shape(Shape::rect(Vec2::new(40.0, 20.0), Color::WHITE)));
        l0.transform.position = Property::constant(Vec2::new(20.0, 10.0));
        l0.matte = Some(creator_model::MatteMode::Alpha);
        comp.add_layer(l0);
        let mut l1 = Layer::new("matte", LayerKind::Shape(Shape::rect(Vec2::new(20.0, 20.0), Color::WHITE)));
        l1.transform.position = Property::constant(Vec2::new(10.0, 10.0));
        l1.matte = Some(creator_model::MatteMode::Alpha);
        comp.add_layer(l1);
        let mut l2 = Layer::new("matte2", LayerKind::Shape(Shape::rect(Vec2::new(40.0, 10.0), Color::WHITE)));
        l2.transform.position = Property::constant(Vec2::new(20.0, 5.0));
        comp.add_layer(l2);
        let id = project.add_composition(comp);
        let t = crate::render_at(&project, id, 0.0);
        assert!(t.pixel(5, 5)[3] > 0.9, "top-left quadrant: visible");
        assert!(t.pixel(5, 15)[3] < 0.05, "bottom-left: cut by L2 via chain");
        assert!(t.pixel(35, 5)[3] < 0.05, "top-right: outside L1");
    }

    #[test]
    fn matte_source_is_not_drawn() {
        // The matte rect is red; with Alpha mode the visible pixels must be the
        // matted layer's white, not the matte's red.
        let (project, id) = matte_comp(
            creator_model::MatteMode::Alpha,
            Color::linear(1.0, 0.0, 0.0, 1.0),
        );
        let t = crate::render_at(&project, id, 0.0);
        let p = t.pixel(5, 10);
        assert!(p[1] > 0.9 && p[2] > 0.9, "should be white (content), not red (matte): {p:?}");
    }

    #[test]
    fn parent_cycle_terminates_and_is_bounded() {
        // Two layers parented to each other (A.parent=B, B.parent=A) must not
        // hang or produce a garbage 256-deep transform.
        let mut project = Project::new("t");
        let mut comp = Composition::new("main", 60, 60, 30.0, 1.0);
        let a = comp.add_layer(Layer::new("a", LayerKind::Shape(Shape::rect(Vec2::new(10.0, 10.0), Color::WHITE))));
        let b = comp.add_layer(Layer::new("b", LayerKind::Null));
        comp.layer_mut(a).unwrap().parent = Some(b);
        comp.layer_mut(b).unwrap().parent = Some(a);
        let id = project.add_composition(comp);
        // Should evaluate (and render) without hanging.
        let mut t = CpuTarget::new(60, 60);
        render(&eval(&project, id, 0.0), &mut t);
        // The transform is bounded (finite) — at least one pixel is finite-valued.
        assert!(t.pixel(30, 30)[3].is_finite());
    }

    #[test]
    fn precomp_cycle_terminates() {
        // A comp that references itself as a precomp must not infinite-loop.
        let mut project = Project::new("t");
        let comp = Composition::new("c", 10, 10, 30.0, 1.0);
        let id = project.add_composition(comp);
        let mut selfref = Layer::new("self", LayerKind::Precomp(id));
        project.composition_mut(id).unwrap().add_layer(selfref.clone());
        let _ = &mut selfref;
        let tree = eval(&project, id, 0.0);
        let mut t = CpuTarget::new(10, 10);
        render(&tree, &mut t); // should return, not hang
        assert_eq!((tree.width, tree.height), (10, 10));
    }

    #[test]
    fn invisible_layer_is_skipped() {
        let mut project = Project::new("t");
        let mut comp = Composition::new("main", 20, 20, 30.0, 2.0);
        let mut layer = Layer::new("box", LayerKind::Shape(Shape::rect(Vec2::new(20.0, 20.0), Color::WHITE)));
        layer.transform.position = Property::constant(Vec2::new(10.0, 10.0));
        layer.time_range = Some((1.0, 2.0));
        let _ = BlendMode::Normal;
        comp.add_layer(layer);
        let id = project.add_composition(comp);
        // before the in-point: nothing drawn.
        let mut t = CpuTarget::new(20, 20);
        render(&eval(&project, id, 0.0), &mut t);
        assert!(t.pixel(10, 10)[3] < 0.01);
        // during: drawn.
        let mut t2 = CpuTarget::new(20, 20);
        render(&eval(&project, id, 1.5), &mut t2);
        assert!(t2.pixel(10, 10)[3] > 0.9);
    }
}
