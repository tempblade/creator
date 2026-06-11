//! `creator-engine` — ties `model` + `anim` + `render` together: evaluate a
//! scene at a time and render it to a target, with an edit/undo command bus and
//! a frame cache.
//!
//! This crate (and everything below it) has **zero** dependency on
//! Tauri/windowing — the hard rule from PLAN.md §3 that makes headless rendering
//! and preview/final parity possible. The desktop app and the CLI both drive
//! this same crate.

mod cache;
mod command;
mod eval;

pub use cache::{FrameCache, FrameKey};
pub use command::{
    AddLayer, Command, Document, RemoveLayer, ReorderLayer, SetBackground, SetLayerBlend,
    SetLayerEnabled, SetLayerName, SetOpacity, SetPosition,
};
pub use eval::eval;

use creator_model::{CompId, MotionBlur, Project};
use creator_render::{render, CpuTarget, RenderTarget};
use rayon::prelude::*;

/// Evaluate `comp` at the given `frame` index and rasterize it. If the comp has
/// motion blur enabled, this averages several temporal sub-samples across the
/// shutter interval (see [`render_motion_blur`]); otherwise it renders the
/// single instant. This is the headless/export path.
pub fn render_frame(project: &Project, comp: CompId, frame: u64) -> CpuTarget {
    let c = match project.composition(comp) {
        Some(c) => c,
        None => return CpuTarget::new(1, 1),
    };
    let time = c.frame_to_time(frame);
    match c.motion_blur {
        Some(mb) if mb.samples >= 2 && mb.shutter_angle > 0.0 && c.frame_rate > 0.0 => {
            render_motion_blur(project, comp, time, c.width, c.height, c.frame_rate, mb)
        }
        _ => render_at(project, comp, time),
    }
}

/// Evaluate + render a single instant at `time` (seconds), **without** motion
/// blur — the instantaneous preview/scrub path. (Motion blur is a frame-timed
/// effect; the desktop scrub uses this for speed, `render_frame` for final.)
pub fn render_at(project: &Project, comp: CompId, time: f64) -> CpuTarget {
    let (w, h) = match project.composition(comp) {
        Some(c) => (c.width, c.height),
        None => (1, 1),
    };
    let tree = eval(project, comp, time);
    let mut target = CpuTarget::new(w, h);
    render(&tree, &mut target);
    target
}

/// The instants a motion-blurred frame samples: the **midpoint** of each of `n`
/// equal time slices across the shutter interval centered on `time`. Midpoint
/// positions with equal weights are an unbiased estimator of the shutter
/// integral (closed-interval endpoints + equal weights would over-count the
/// shutter edges). Public so alternative backends (GPU) average the exact same
/// instants as the CPU path.
pub fn shutter_sample_times(time: f64, frame_rate: f64, mb: MotionBlur) -> Vec<f64> {
    if frame_rate <= 0.0 || frame_rate.is_nan() {
        return vec![time];
    }
    let dt = 1.0 / frame_rate;
    let shutter = (mb.shutter_angle / 360.0).clamp(0.0, 1.0) * dt; // seconds open
    let n = mb.samples.max(2);
    (0..n)
        .map(|i| {
            let frac = (i as f64 + 0.5) / n as f64;
            time - shutter * 0.5 + frac * shutter
        })
        .collect()
}

/// Render a frame with motion blur: sample the scene at `mb.samples` instants
/// spread across the shutter interval (centered on `time`) and average them in
/// **premultiplied linear light** (a physically correct temporal integration of
/// radiance — only possible because `eval` is a pure function of time).
pub fn render_motion_blur(
    project: &Project,
    comp: CompId,
    time: f64,
    width: u32,
    height: u32,
    frame_rate: f64,
    mb: MotionBlur,
) -> CpuTarget {
    // Self-guard: a non-positive (or NaN) frame rate has no shutter interval;
    // render the instant rather than dividing into inf/NaN sample times.
    if frame_rate <= 0.0 || frame_rate.is_nan() {
        return render_at(project, comp, time);
    }
    let times = shutter_sample_times(time, frame_rate, mb);
    let count = (width as usize) * (height as usize);
    let n = times.len() as u32;

    // Sub-samples are independent renders of a pure function of time, so they
    // run in parallel. `collect` preserves index order; the accumulation below
    // is then sequential in that fixed order — f32 addition isn't associative,
    // so this is what keeps the result bit-deterministic across runs and thread
    // counts. (Peak memory is `n` frame buffers; motion blur is inherently an
    // n-renders cost.)
    let samples: Vec<CpuTarget> = times
        .par_iter()
        .map(|&st| {
            let tree = eval(project, comp, st);
            let mut sample = CpuTarget::new(width, height);
            render(&tree, &mut sample);
            sample
        })
        .collect();

    let mut acc = vec![[0.0f32; 4]; count];
    for sample in &samples {
        for (a, p) in acc.iter_mut().zip(sample.pixels()) {
            a[0] += p[0];
            a[1] += p[1];
            a[2] += p[2];
            a[3] += p[3];
        }
    }
    let inv = 1.0 / n as f32;
    for a in &mut acc {
        a[0] *= inv;
        a[1] *= inv;
        a[2] *= inv;
        a[3] *= inv;
    }
    CpuTarget::from_premultiplied(width, height, acc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use creator_model::{
        Color, Composition, Keyframe, Layer, LayerKind, MotionBlur, Project, Property, Shape, Vec2,
    };

    #[test]
    fn motion_blur_on_static_scene_is_unchanged() {
        // Averaging identical sub-frames must equal a single render.
        let mut project = Project::new("t");
        let mut comp = Composition::new("m", 20, 20, 30.0, 1.0);
        comp.background = Color::BLACK;
        let mut layer = Layer::new("box", LayerKind::Shape(Shape::rect(Vec2::new(10.0, 10.0), Color::WHITE)));
        layer.transform.position = Property::constant(Vec2::new(10.0, 10.0));
        comp.add_layer(layer);
        let mut comp_mb = comp.clone();
        comp_mb.motion_blur = Some(MotionBlur { samples: 8, shutter_angle: 180.0 });
        let plain = project.add_composition(comp);
        let mb = project.add_composition(comp_mb);
        assert_eq!(render_frame(&project, plain, 5).to_srgba8(), render_frame(&project, mb, 5).to_srgba8());
    }

    #[test]
    fn motion_blur_smears_a_moving_edge() {
        let mut project = Project::new("t");
        let mut comp = Composition::new("m", 60, 20, 30.0, 1.0);
        // Transparent bg so the moving box's coverage shows in the alpha channel.
        comp.background = Color::TRANSPARENT;
        comp.motion_blur = Some(MotionBlur { samples: 16, shutter_angle: 360.0 });
        let mut layer = Layer::new("box", LayerKind::Shape(Shape::rect(Vec2::new(8.0, 20.0), Color::WHITE)));
        layer.transform.position = Property::animated(vec![
            Keyframe::new(0.0, Vec2::new(8.0, 10.0)),
            Keyframe::new(1.0, Vec2::new(52.0, 10.0)),
        ]);
        comp.add_layer(layer);
        let id = project.add_composition(comp);

        let blurred = render_frame(&project, id, 15); // motion-blurred
        let sharp = render_at(&project, id, 0.5); // instantaneous
        assert_ne!(blurred.to_srgba8(), sharp.to_srgba8(), "blur should differ from sharp");
        let has_partial = (0..60).any(|x| {
            let a = blurred.pixel(x, 10)[3];
            a > 0.05 && a < 0.95
        });
        assert!(has_partial, "motion blur should produce a partial-coverage smear");
    }

    #[test]
    fn motion_blur_is_bit_deterministic() {
        // Parallel sub-sample rendering must not change the result between
        // runs (ordered accumulation; f32 addition isn't associative).
        let mut project = Project::new("t");
        let mut comp = Composition::new("m", 60, 20, 30.0, 1.0);
        comp.motion_blur = Some(MotionBlur { samples: 12, shutter_angle: 270.0 });
        let mut layer = Layer::new("box", LayerKind::Shape(Shape::rect(Vec2::new(8.0, 20.0), Color::WHITE)));
        layer.transform.position = Property::animated(vec![
            Keyframe::new(0.0, Vec2::new(8.0, 10.0)),
            Keyframe::new(1.0, Vec2::new(52.0, 10.0)),
        ]);
        comp.add_layer(layer);
        let id = project.add_composition(comp);
        let a = render_frame(&project, id, 15);
        let b = render_frame(&project, id, 15);
        assert_eq!(a.pixels(), b.pixels(), "identical input must give bit-identical output");
    }

    #[test]
    fn motion_blur_with_zero_frame_rate_is_safe() {
        // Calling the public fn directly with frame_rate=0 must not divide into
        // inf/NaN sample times — it falls back to the instant.
        let mut project = Project::new("t");
        let mut comp = Composition::new("m", 8, 8, 30.0, 1.0);
        comp.add_layer(Layer::new("box", LayerKind::Shape(Shape::rect(Vec2::new(4.0, 4.0), Color::WHITE))));
        let id = project.add_composition(comp);
        let mb = MotionBlur { samples: 8, shutter_angle: 180.0 };
        let blurred = render_motion_blur(&project, id, 0.5, 8, 8, 0.0, mb);
        assert_eq!(blurred.to_srgba8(), render_at(&project, id, 0.5).to_srgba8());
    }

    #[test]
    fn render_frame_matches_render_at_for_same_instant() {
        let mut project = Project::new("t");
        let mut comp = Composition::new("m", 40, 40, 25.0, 2.0);
        let mut layer = Layer::new("box", LayerKind::Shape(Shape::rect(Vec2::new(10.0, 10.0), Color::WHITE)));
        layer.transform.position = Property::animated(vec![
            Keyframe::new(0.0, Vec2::new(5.0, 20.0)),
            Keyframe::new(2.0, Vec2::new(35.0, 20.0)),
        ]);
        comp.add_layer(layer);
        let id = project.add_composition(comp);

        // frame 25 == 1.0s at 25fps.
        let a = render_frame(&project, id, 25);
        let b = render_at(&project, id, 1.0);
        assert_eq!(a.to_srgba8(), b.to_srgba8());
    }
}
