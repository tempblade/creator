use crate::animation::{
    primitives::{
        paint::TextPaint,
        transform::{AnimatedTransform, Transform},
        values::animated_values::{AnimatedFloatVec2, AnimatedValue},
    },
    timeline::Timeline,
};
use serde::{Deserialize, Serialize};

use super::common::{Animateable, AnimationData, Cache, Drawable, Entity};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextEntity {
    pub id: String,
    pub cache: Cache,
    pub text: String,
    pub origin: (f32, f32),
    pub paint: TextPaint,
    pub transform: Option<Transform>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimatedTextEntity {
    pub id: String,
    pub cache: Cache,
    pub text: String,
    pub origin: AnimatedFloatVec2,
    pub paint: TextPaint,
    pub animation_data: AnimationData,
    pub transform: Option<AnimatedTransform>,
}

impl Drawable for AnimatedTextEntity {}

impl AnimatedTextEntity {
    fn into_static(&mut self, timeline: &Timeline) -> TextEntity {
        self.sort_keyframes();

        let origin = self.origin.get_value_at_frame(
            timeline.render_state.curr_frame,
            &self.animation_data,
            timeline.fps,
        );

        let transform: Option<Transform> = match self.transform.clone() {
            Some(mut val) => Some(val.calculate(timeline, &self.animation_data)),
            None => None,
        };

        TextEntity {
            id: self.id.clone(),
            cache: self.cache.clone(),
            transform,
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
        if let Some(x) = &mut self.transform {
            x.sort_keyframes();
        }

        self.origin.sort_keyframes();
    }
}
