use serde::{Deserialize, Serialize};

use crate::animation::timeline::Timeline;

use super::{
    paint::{Paint, TextPaint},
    utils::timestamp_to_frame,
    values::{AnimatedFloatVec2, AnimatedValue},
};

//#region Animateable Objects

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnimatedEntity {
    Text(AnimatedTextEntity),
    Ellipse(AnimatedEllipseEntity),
    Box(AnimatedBoxEntity),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Entity {
    Text(TextEntity),
    Ellipse(EllipseEntity),
    Box(BoxEntity),
}

impl AnimatedEntity {
    pub fn calculate(&mut self, timeline: &Timeline) -> Option<Entity> {
        match self {
            Self::Text(text_entity) => text_entity.calculate(timeline),
            Self::Box(box_entity) => box_entity.calculate(timeline),
            Self::Ellipse(ellipse_entity) => ellipse_entity.calculate(timeline),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimatedTextEntity {
    pub text: String,
    pub origin: AnimatedFloatVec2,
    pub paint: TextPaint,
    pub animation_data: AnimationData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextEntity {
    pub text: String,
    pub origin: (f32, f32),
    pub paint: TextPaint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimatedBoxEntity {
    pub position: AnimatedFloatVec2,
    pub size: AnimatedFloatVec2,
    pub origin: AnimatedFloatVec2,
    pub paint: Paint,
    pub animation_data: AnimationData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoxEntity {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub origin: (f32, f32),
    pub paint: Paint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimatedEllipseEntity {
    pub paint: Paint,
    pub radius: AnimatedFloatVec2,
    pub origin: AnimatedFloatVec2,
    pub position: AnimatedFloatVec2,
    pub animation_data: AnimationData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EllipseEntity {
    pub radius: (f32, f32),
    pub position: (f32, f32),
    pub origin: (f32, f32),
    pub paint: Paint,
}

pub trait Animateable {
    fn sort_keyframes(&mut self);

    fn calculate(&mut self, timeline: &Timeline) -> Option<Entity>;

    // Checks if the Box is visible and should be drawn
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

impl AnimatedTextEntity {
    fn into_static(&mut self, timeline: &Timeline) -> TextEntity {
        self.sort_keyframes();

        let origin = self.origin.get_value_at_frame(
            timeline.render_state.curr_frame,
            &self.animation_data,
            timeline.fps,
        );

        TextEntity {
            text: self.text.clone(),
            origin,
            paint: self.paint.clone(),
        }
    }
}

impl Animateable for AnimatedTextEntity {
    fn calculate(&mut self, timeline: &Timeline) -> Option<Entity> {
        let should_draw = self.should_draw(&self.animation_data, timeline);

        if should_draw {
            self.sort_keyframes();

            Some(Entity::Text(self.into_static(timeline)))
        } else {
            None
        }
    }

    fn sort_keyframes(&mut self) {
        self.origin.sort_keyframes();
    }
}

impl Animateable for AnimatedBoxEntity {
    fn sort_keyframes(&mut self) {
        self.position.sort_keyframes();
        self.size.sort_keyframes();
    }

    fn calculate(&mut self, timeline: &Timeline) -> Option<Entity> {
        let should_draw = self.should_draw(&self.animation_data, timeline);

        if should_draw {
            self.sort_keyframes();

            let position = self.position.get_value_at_frame(
                timeline.render_state.curr_frame,
                &self.animation_data,
                timeline.fps,
            );

            let size = self.size.get_value_at_frame(
                timeline.render_state.curr_frame,
                &self.animation_data,
                timeline.fps,
            );

            let origin = self.origin.get_value_at_frame(
                timeline.render_state.curr_frame,
                &self.animation_data,
                timeline.fps,
            );

            Some(Entity::Box(BoxEntity {
                position,
                size,
                origin,
                paint: self.paint.clone(),
            }))
        } else {
            None
        }
    }
}

impl Animateable for AnimatedEllipseEntity {
    fn calculate(&mut self, timeline: &Timeline) -> Option<Entity> {
        let should_draw = self.should_draw(&self.animation_data, timeline);

        if should_draw {
            self.sort_keyframes();

            let radius = self.radius.get_value_at_frame(
                timeline.render_state.curr_frame,
                &self.animation_data,
                timeline.fps,
            );

            let position = self.position.get_value_at_frame(
                timeline.render_state.curr_frame,
                &self.animation_data,
                timeline.fps,
            );

            let origin = self.origin.get_value_at_frame(
                timeline.render_state.curr_frame,
                &self.animation_data,
                timeline.fps,
            );

            Some(Entity::Ellipse(EllipseEntity {
                radius,
                position,
                origin,
                paint: self.paint.clone(),
            }))
        } else {
            None
        }
    }

    fn sort_keyframes(&mut self) {
        self.position.sort_keyframes();
        self.radius.sort_keyframes();
    }
}

//#endregion

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationData {
    pub offset: f32,
    pub duration: f32,
    pub visible: bool,
}
