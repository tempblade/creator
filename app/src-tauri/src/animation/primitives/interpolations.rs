use std::cmp::Ordering;

use super::keyframe::RenderedKeyframe;
use serde::{Deserialize, Serialize};
use simple_easing::{
    circ_in, circ_in_out, circ_out, cubic_in, cubic_in_out, cubic_out, expo_in, expo_in_out,
    expo_out, quad_in, quad_in_out, quad_out, quart_in, quart_in_out, quart_out, quint_in,
    quint_in_out, quint_out,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SpringProperties {
    pub mass: f32,
    pub damping: f32,
    pub stiffness: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SpringState {
    pub velocity: f32,
    pub last_val: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "easing_function")]
pub enum EasingFunction {
    QuintOut,
    QuintIn,
    QuintInOut,
    CircOut,
    CircIn,
    CircInOut,
    CubicOut,
    CubicIn,
    CubicInOut,
    ExpoOut,
    ExpoIn,
    ExpoInOut,
    QuadOut,
    QuadIn,
    QuadInOut,
    QuartOut,
    QuartIn,
    QuartInOut,
}

impl EasingFunction {
    fn ease(self: &Self, t: f32) -> f32 {
        match self {
            EasingFunction::QuintOut => quint_out(t),
            EasingFunction::QuintIn => quint_in(t),
            EasingFunction::QuintInOut => quint_in_out(t),
            EasingFunction::CircOut => circ_out(t),
            EasingFunction::CircIn => circ_in(t),
            EasingFunction::CircInOut => circ_in_out(t),
            EasingFunction::CubicOut => cubic_out(t),
            EasingFunction::CubicIn => cubic_in(t),
            EasingFunction::CubicInOut => cubic_in_out(t),
            EasingFunction::ExpoOut => expo_out(t),
            EasingFunction::ExpoIn => expo_in(t),
            EasingFunction::ExpoInOut => expo_in_out(t),
            EasingFunction::QuadOut => quad_out(t),
            EasingFunction::QuadIn => quad_in(t),
            EasingFunction::QuadInOut => quad_in_out(t),
            EasingFunction::QuartOut => quart_out(t),
            EasingFunction::QuartIn => quart_in(t),
            EasingFunction::QuartInOut => quart_in_out(t),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InterpolationType {
    Linear,
    Spring(SpringProperties),
    EasingFunction(EasingFunction),
}

pub fn calculate_spring_value(
    curr_frame: i32,
    start_value: f32,
    target_value: f32,
    start_frame: i32,
    _end_frame: i32,
    spring_props: &SpringProperties,
) -> f32 {
    const PRECISION: f32 = 0.01;
    const STEP: f32 = 10.0;
    const REST_VELOCITY: f32 = PRECISION / 10.0;

    let _is_growing = match start_value.total_cmp(&target_value) {
        Ordering::Equal => false,
        Ordering::Less => false,
        Ordering::Greater => true,
    };

    let mut _is_moving = false;
    let mut spring_state = SpringState {
        last_val: start_value,
        velocity: 0.0,
    };

    let mut position = start_value;
    let relative_curr_frame = curr_frame - start_frame;

    //  println!("target_value {target_value} start_value {start_value}");
    //  println!("start_frame {start_frame} end_frame {end_frame}");

    for _ in 0..relative_curr_frame {
        let _is_moving = spring_state.velocity.abs() > REST_VELOCITY;

        let spring_force = -spring_props.stiffness * 0.000001 * (position - target_value);
        let damping_force = -spring_props.damping * 0.001 * spring_state.velocity;
        let acceleration = (spring_force + damping_force) / spring_props.mass; // pt/ms^2

        spring_state.velocity = spring_state.velocity + acceleration * STEP; // pt/ms
        position = position + spring_state.velocity * STEP;
        //       println!("{position}")
    }

    position
}

pub fn interpolate_rendered_keyframes(
    first_ren_keyframe: &RenderedKeyframe,
    second_ren_keyframe: &RenderedKeyframe,
    curr_frame: i32,
    interpolation_type: InterpolationType,
    _fps: i16,
) -> f32 {
    let frame_range = second_ren_keyframe.absolute_frame - first_ren_keyframe.absolute_frame;
    let position_in_range = curr_frame - first_ren_keyframe.absolute_frame;
    let progress: f32 = (1.0 / frame_range as f32) * position_in_range as f32;

    /*     println!(
        "Progress:{0} Frame_Range: {1} Position_In_Range: {2}",
        progress, frame_range, position_in_range
    ); */

    let value_diff = second_ren_keyframe.keyframe.value - first_ren_keyframe.keyframe.value;

    match interpolation_type {
        InterpolationType::Linear => {
            let interpolated_val =
                first_ren_keyframe.keyframe.value + (value_diff * progress as f32);

            return interpolated_val;
        }
        InterpolationType::EasingFunction(easing_function) => {
            let eased_progress = easing_function.ease(progress);

            let interpolated_val =
                first_ren_keyframe.keyframe.value + (value_diff * eased_progress as f32);

            return interpolated_val;
        }
        InterpolationType::Spring(spring_properties) => {
            let interpolated_value = calculate_spring_value(
                curr_frame,
                first_ren_keyframe.keyframe.value,
                second_ren_keyframe.keyframe.value,
                first_ren_keyframe.absolute_frame,
                second_ren_keyframe.absolute_frame,
                &spring_properties,
            );
            return interpolated_value;
        }
    };
}
