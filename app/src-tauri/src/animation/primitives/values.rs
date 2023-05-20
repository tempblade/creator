use serde::{Deserialize, Serialize};

use super::{
    entities::AnimationData,
    keyframe::{Keyframe, Keyframes},
};

pub trait AnimatedValue<T> {
    fn sort_keyframes(&mut self);
    fn get_value_at_frame(&self, curr_frame: i32, animation_data: &AnimationData, fps: i16) -> T;
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimatedFloat {
    pub keyframes: Keyframes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimatedFloatVec2 {
    pub keyframes: (AnimatedFloat, AnimatedFloat),
}

impl AnimatedFloat {
    pub fn new(val: f32) -> AnimatedFloat {
        AnimatedFloat {
            keyframes: Keyframes {
                values: vec![Keyframe {
                    value: val,
                    offset: 0.0,
                    interpolation: None,
                }],
            },
        }
    }
}

impl AnimatedFloatVec2 {
    pub fn new(x: f32, y: f32) -> AnimatedFloatVec2 {
        AnimatedFloatVec2 {
            keyframes: (AnimatedFloat::new(x), AnimatedFloat::new(y)),
        }
    }
}

impl AnimatedValue<f32> for AnimatedFloat {
    fn sort_keyframes(&mut self) {
        self.keyframes.sort();
    }

    fn get_value_at_frame(&self, curr_frame: i32, animation_data: &AnimationData, fps: i16) -> f32 {
        self.keyframes
            .get_value_at_frame(curr_frame, &animation_data, fps)
    }
}

impl AnimatedValue<(f32, f32)> for AnimatedFloatVec2 {
    fn sort_keyframes(&mut self) {
        self.keyframes.0.sort_keyframes();
        self.keyframes.1.sort_keyframes();
    }

    fn get_value_at_frame(
        &self,
        curr_frame: i32,
        animation_data: &AnimationData,
        fps: i16,
    ) -> (f32, f32) {
        let x = self
            .keyframes
            .0
            .get_value_at_frame(curr_frame, animation_data, fps);

        let y = self
            .keyframes
            .1
            .get_value_at_frame(curr_frame, animation_data, fps);

        return (x, y);
    }
}
