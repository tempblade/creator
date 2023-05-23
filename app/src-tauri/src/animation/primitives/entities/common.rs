use serde::{Deserialize, Serialize};

use crate::animation::{primitives::utils::timestamp_to_frame, timeline::Timeline};

use super::{
    ellipse::{AnimatedEllipseEntity, EllipseEntity},
    rect::{AnimatedRectEntity, RectEntity},
    staggered_text::{AnimatedStaggeredTextEntity, StaggeredTextEntity},
    text::{AnimatedTextEntity, TextEntity},
};

pub trait Drawable {
    fn should_draw(&self, animation_data: &AnimationData, timeline: &Timeline) -> bool {
        let start_frame = timestamp_to_frame(animation_data.offset, timeline.fps);
        let end_frame = timestamp_to_frame(
            animation_data.offset + animation_data.duration,
            timeline.fps,
        );

        // println!("start {0} end {1}", start_frame, end_frame);

        let is_before = timeline.render_state.curr_frame < start_frame;
        let is_after = timeline.render_state.curr_frame > end_frame;
        let is_between = !is_after && !is_before;

        if is_between {
            return true;
        } else {
            return false;
        }
    }
}

pub trait Animateable {
    fn sort_keyframes(&mut self);

    fn calculate(&mut self, timeline: &Timeline) -> Option<Entity>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnimatedEntity {
    Text(AnimatedTextEntity),
    StaggeredText(AnimatedStaggeredTextEntity),
    Ellipse(AnimatedEllipseEntity),
    Rect(AnimatedRectEntity),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Entity {
    Text(TextEntity),
    StaggeredText(StaggeredTextEntity),
    Ellipse(EllipseEntity),
    Rect(RectEntity),
}

impl AnimatedEntity {
    pub fn calculate(&mut self, timeline: &Timeline) -> Option<Entity> {
        match self {
            Self::Text(text_entity) => text_entity.calculate(timeline),
            Self::Rect(box_entity) => box_entity.calculate(timeline),
            Self::StaggeredText(staggered_text_entity) => staggered_text_entity.calculate(timeline),
            Self::Ellipse(ellipse_entity) => ellipse_entity.calculate(timeline),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationData {
    pub offset: f32,
    pub duration: f32,
    pub visible: bool,
}
