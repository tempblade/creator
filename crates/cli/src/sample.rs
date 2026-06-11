//! Builds an example project that exercises the engine: animated position,
//! rotation, color, a closed-form spring bounce, opacity + a blur effect, a
//! blend mode, and a text placeholder. Used by `creator sample`.

use creator_anim::{Easing, Interp, Spring};
use creator_model::{
    BlendMode, Color, Composition, CompId, Effect, EffectKind, Fill, Geometry, GradientStop,
    Keyframe, Layer, LayerKind, MatteMode, MotionBlur, Paint, PathData, PathPoint, Project,
    Property, Shape, Stroke, SubPath, Text, Vec2,
};

/// Construct the demo project (one composition, several animated layers).
pub fn build_sample_project() -> Project {
    let mut project = Project::new("Creator Demo");
    let mut comp = Composition::new("Main", 480, 270, 30.0, 3.0);
    comp.background = Color::from_srgb8(18, 20, 28, 255);
    // Motion blur showcases the deterministic, time-pure eval: the renderer
    // samples several instants across the shutter and averages them.
    comp.motion_blur = Some(MotionBlur { samples: 12, shutter_angle: 180.0 });

    comp.add_layer(blob());
    comp.add_layer(spinner());
    comp.add_layer(bouncer());
    comp.add_layer(progress_bar());
    comp.add_layer(title());
    // Track matte directly above the title: a rect sliding left→right wipes the
    // title in (the matte layer itself is consumed, not drawn).
    comp.add_layer(title_reveal_matte());

    let _: CompId = project.add_composition(comp);
    project
}

/// A rotating square whose fill animates red→blue with an ease-in-out curve.
fn spinner() -> Layer {
    let mut shape = Shape::rect(Vec2::new(90.0, 90.0), Color::from_srgb8(229, 57, 53, 255));
    if let Some(fill) = shape.fill.as_mut() {
        fill.paint = Paint::Solid(Property::animated(vec![
            Keyframe::new(0.0, Color::from_srgb8(229, 57, 53, 255))
                .with_out(Interp::Easing(Easing::ease_in_out())),
            Keyframe::new(3.0, Color::from_srgb8(33, 150, 243, 255)),
        ]));
    }
    shape.stroke = Some(Stroke {
        color: Property::constant(Color::WHITE),
        width: Property::constant(3.0),
    });
    let mut layer = Layer::new("Spinner", LayerKind::Shape(shape));
    layer.transform.position = Property::constant(Vec2::new(130.0, 150.0));
    layer.transform.rotation = Property::animated(vec![
        Keyframe::new(0.0, 0.0).with_out(Interp::Easing(Easing::ease_in_out())),
        Keyframe::new(3.0, 540.0),
    ]);
    layer
}

/// A ball that drops and settles with a closed-form spring (overshoot/bounce).
/// Filled with a radial gradient (offset highlight) for a shaded look.
fn bouncer() -> Layer {
    let mut shape = Shape::ellipse(Vec2::new(56.0, 56.0), Color::from_srgb8(255, 193, 7, 255));
    shape.fill = Some(Fill::radial(
        Vec2::new(-7.0, -7.0),
        38.0,
        vec![
            GradientStop::new(0.0, Color::from_srgb8(255, 241, 191, 255)),
            GradientStop::new(0.55, Color::from_srgb8(255, 193, 7, 255)),
            GradientStop::new(1.0, Color::from_srgb8(214, 128, 0, 255)),
        ],
    ));
    let mut layer = Layer::new("Bouncer", LayerKind::Shape(shape));
    layer.transform.position = Property::animated(vec![
        Keyframe::new(0.2, Vec2::new(340.0, 40.0))
            .with_out(Interp::Spring(Spring::perceptual(0.6, 0.35))),
        Keyframe::new(3.0, Vec2::new(340.0, 210.0)),
    ]);
    layer
}

/// A growing bar that fades in while a blur resolves — opacity + effect chain,
/// filled with a left-to-right linear gradient.
fn progress_bar() -> Layer {
    let mut shape = Shape::rect(Vec2::new(240.0, 16.0), Color::from_srgb8(76, 175, 80, 255));
    shape.fill = Some(Fill::linear(
        Vec2::new(-120.0, 0.0),
        Vec2::new(120.0, 0.0),
        vec![
            GradientStop::new(0.0, Color::from_srgb8(76, 175, 80, 255)),
            GradientStop::new(1.0, Color::from_srgb8(0, 200, 180, 255)),
        ],
    ));
    let mut layer = Layer::new("Progress", LayerKind::Shape(shape));
    layer.transform.position = Property::constant(Vec2::new(240.0, 244.0));
    layer.transform.scale = Property::animated(vec![
        Keyframe::new(0.0, Vec2::new(0.0, 1.0)).with_out(Interp::Easing(Easing::ease_out())),
        Keyframe::new(2.0, Vec2::new(1.0, 1.0)),
    ]);
    layer.opacity = Property::animated(vec![
        Keyframe::new(0.0, 0.0_f64),
        Keyframe::new(0.6, 1.0),
    ]);
    layer.blend = BlendMode::Screen;
    layer.effects.push(Effect {
        enabled: true,
        kind: EffectKind::GaussianBlur {
            radius: Property::animated(vec![
                Keyframe::new(0.0, 8.0_f64),
                Keyframe::new(1.0, 0.0),
            ]),
        },
    });
    layer
}

/// A smooth organic blob built from cubic-Bézier path points, with a gradient
/// fill and a slow rotation — showcases curved path geometry.
fn blob() -> Layer {
    let points = vec![
        PathPoint::smooth(Vec2::new(0.0, -30.0), Vec2::new(24.0, 0.0)),
        PathPoint::smooth(Vec2::new(34.0, 6.0), Vec2::new(0.0, 22.0)),
        PathPoint::smooth(Vec2::new(0.0, 32.0), Vec2::new(-24.0, 4.0)),
        PathPoint::smooth(Vec2::new(-30.0, -4.0), Vec2::new(0.0, -22.0)),
    ];
    let shape = Shape {
        geometry: Geometry::Path {
            data: Property::constant(PathData { subpaths: vec![SubPath { closed: true, points }] }),
        },
        fill: Some(Fill::linear(
            Vec2::new(-30.0, -30.0),
            Vec2::new(30.0, 30.0),
            vec![
                GradientStop::new(0.0, Color::from_srgb8(124, 77, 255, 255)),
                GradientStop::new(1.0, Color::from_srgb8(236, 64, 162, 255)),
            ],
        )),
        stroke: Some(Stroke {
            color: Property::constant(Color::from_srgb8(255, 255, 255, 60)),
            width: Property::constant(2.0),
        }),
    };
    let mut layer = Layer::new("Blob", LayerKind::Shape(shape));
    layer.transform.position = Property::constant(Vec2::new(410.0, 80.0));
    layer.transform.rotation = Property::animated(vec![
        Keyframe::new(0.0, 0.0),
        Keyframe::new(3.0, -120.0),
    ]);
    layer
}

/// A text placeholder near the top (real glyphs land with the Skia backend).
fn title() -> Layer {
    let mut text = Text::new("creator");
    text.font_size = Property::constant(34.0);
    text.color = Property::constant(Color::from_srgb8(236, 239, 244, 255));
    let mut layer = Layer::new("Title", LayerKind::Text(text));
    layer.transform.position = Property::constant(Vec2::new(40.0, 30.0));
    layer.opacity = Property::animated(vec![
        Keyframe::new(0.0, 0.0_f64).with_out(Interp::Easing(Easing::ease_out())),
        Keyframe::new(0.8, 1.0),
    ]);
    layer.matte = Some(MatteMode::Alpha);
    layer
}

/// The title's wipe matte: a tall rect sliding in from the left over the first
/// second. Consumed as the title's alpha matte — never drawn itself.
fn title_reveal_matte() -> Layer {
    let shape = Shape::rect(Vec2::new(160.0, 60.0), Color::WHITE);
    let mut layer = Layer::new("Title reveal", LayerKind::Shape(shape));
    layer.transform.position = Property::animated(vec![
        // Slides from fully off the title's left edge to fully covering it
        // (title box spans x ≈ -26..106; the settled matte spans -40..120).
        Keyframe::new(0.0, Vec2::new(-120.0, 50.0)).with_out(Interp::Easing(Easing::ease_out())),
        Keyframe::new(1.0, Vec2::new(40.0, 50.0)),
    ]);
    layer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_renders_nonblank_frames() {
        let project = build_sample_project();
        let comp = project.root.unwrap();
        // Render midway; expect some lit pixels.
        let target = creator_engine::render_at(&project, comp, 1.5);
        let rgba = target.to_srgba8();
        let lit = rgba.chunks_exact(4).filter(|p| p[3] > 0).count();
        assert!(lit > 100, "expected a populated frame, got {lit} lit pixels");
    }

    #[test]
    fn sample_serializes() {
        let project = build_sample_project();
        let json = project.to_json().unwrap();
        assert!(Project::from_json(&json).is_ok());
    }
}
