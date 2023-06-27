use super::{
    entities::common::AnimationData,
    values::animated_values::{AnimatedFloatVec2, AnimatedFloatVec3, AnimatedValue},
};
use crate::animation::timeline::Timeline;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimatedTransform {
    pub translate: AnimatedFloatVec2,
    pub scale: AnimatedFloatVec2,
    pub skew: AnimatedFloatVec2,
    pub rotate: AnimatedFloatVec3,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transform {
    pub translate: (f32, f32),
    pub scale: (f32, f32),
    pub skew: (f32, f32),
    pub rotate: (f32, f32, f32),
}

impl AnimatedTransform {
    pub fn sort_keyframes(&mut self) {
        self.rotate.sort_keyframes();
        self.skew.sort_keyframes();
        self.scale.sort_keyframes();
        self.translate.sort_keyframes();
    }

    pub fn calculate(&mut self, timeline: &Timeline, animation_data: &AnimationData) -> Transform {
        let skew = self.skew.get_value_at_frame(
            timeline.render_state.curr_frame,
            animation_data,
            timeline.fps,
        );

        let scale = self.scale.get_value_at_frame(
            timeline.render_state.curr_frame,
            animation_data,
            timeline.fps,
        );

        let translate = self.translate.get_value_at_frame(
            timeline.render_state.curr_frame,
            animation_data,
            timeline.fps,
        );

        let rotate = self.rotate.get_value_at_frame(
            timeline.render_state.curr_frame,
            animation_data,
            timeline.fps,
        );

        Transform {
            skew,
            scale,
            translate,
            rotate,
        }
    }
}
