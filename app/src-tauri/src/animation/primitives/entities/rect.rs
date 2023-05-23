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
pub struct AnimatedRectEntity {
    pub position: AnimatedFloatVec2,
    pub size: AnimatedFloatVec2,
    pub origin: AnimatedFloatVec2,
    pub paint: Paint,
    pub animation_data: AnimationData,
    pub transform: Option<AnimatedTransform>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RectEntity {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub origin: (f32, f32),
    pub paint: Paint,
    pub transform: Option<Transform>,
}

impl Drawable for AnimatedRectEntity {}
impl Animateable for AnimatedRectEntity {
    fn sort_keyframes(&mut self) {
        if let Some(x) = &mut self.transform {
            x.sort_keyframes();
        }

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

            let transform: Option<Transform> = match self.transform.clone() {
                Some(mut val) => Some(val.calculate(timeline, &self.animation_data)),
                None => None,
            };

            Some(Entity::Rect(RectEntity {
                position,
                size,
                origin,
                paint: self.paint.clone(),
                transform,
            }))
        } else {
            None
        }
    }
}
