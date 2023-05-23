use serde::{Deserialize, Serialize};

use crate::animation::{
    primitives::{
        paint::Paint,
        transform::{AnimatedTransform, Transform},
        values::{AnimatedFloatVec2, AnimatedValue},
    },
    timeline::Timeline,
};

use super::common::{Animateable, AnimationData, Drawable, Entity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimatedEllipseEntity {
    pub paint: Paint,
    pub radius: AnimatedFloatVec2,
    pub origin: AnimatedFloatVec2,
    pub position: AnimatedFloatVec2,
    pub animation_data: AnimationData,
    pub transform: Option<AnimatedTransform>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EllipseEntity {
    pub radius: (f32, f32),
    pub position: (f32, f32),
    pub origin: (f32, f32),
    pub paint: Paint,
    pub transform: Option<Transform>,
}

impl Drawable for AnimatedEllipseEntity {}
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

            let transform: Option<Transform> = match self.transform.clone() {
                Some(mut val) => Some(val.calculate(timeline, &self.animation_data)),
                None => None,
            };

            Some(Entity::Ellipse(EllipseEntity {
                radius,
                position,
                origin,
                paint: self.paint.clone(),
                transform,
            }))
        } else {
            None
        }
    }

    fn sort_keyframes(&mut self) {
        let transform = self.transform.clone();

        if let Some(mut transform) = transform {
            transform.sort_keyframes();
        }

        self.position.sort_keyframes();
        self.radius.sort_keyframes();
    }
}
