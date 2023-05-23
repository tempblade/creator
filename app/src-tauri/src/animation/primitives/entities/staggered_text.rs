use super::common::{Animateable, AnimationData, Cache, Drawable, Entity};
use crate::animation::{
    primitives::{
        paint::TextPaint,
        transform::{AnimatedTransform, Transform},
        values::{AnimatedFloatVec2, AnimatedValue},
    },
    timeline::Timeline,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimatedStaggeredTextLetter {
    pub transform: Option<AnimatedTransform>,
    pub paint: TextPaint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaggeredTextLetter {
    pub transform: Option<Vec<Transform>>,
    pub paint: TextPaint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimatedStaggeredTextEntity {
    pub id: String,
    pub cache: Cache,
    pub text: String,
    pub stagger: f32,
    pub origin: AnimatedFloatVec2,
    pub animation_data: AnimationData,
    pub transform: Option<AnimatedTransform>,
    pub letter: AnimatedStaggeredTextLetter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaggeredTextEntity {
    pub id: String,
    pub cache: Cache,
    pub text: String,
    pub stagger: f32,
    pub origin: (f32, f32),
    pub transform: Option<Transform>,
    pub animation_data: AnimationData,
    pub letter: StaggeredTextLetter,
}

impl Drawable for AnimatedStaggeredTextEntity {}
impl Animateable for AnimatedStaggeredTextEntity {
    fn calculate(&mut self, timeline: &Timeline) -> Option<Entity> {
        let should_draw: bool = self.should_draw(&self.animation_data, timeline);

        if should_draw {
            self.sort_keyframes();

            let transform: Option<Transform> = match self.transform.clone() {
                Some(mut val) => Some(val.calculate(timeline, &self.animation_data)),
                None => None,
            };

            // Iterate over the chars of the string and calculate the animation with the staggered offset
            let letter_transform: Option<Vec<Transform>> = match self.letter.transform.clone() {
                Some(mut val) => {
                    let mut transforms: Vec<Transform> = Vec::new();

                    for c in self.text.chars().enumerate() {
                        let mut animation_data = self.animation_data.clone();
                        animation_data.offset += self.stagger * c.0 as f32;

                        let transform = val.calculate(timeline, &animation_data);
                        transforms.push(transform);
                    }

                    Some(transforms)
                }
                None => None,
            };

            let origin = self.origin.get_value_at_frame(
                timeline.render_state.curr_frame,
                &self.animation_data,
                timeline.fps,
            );

            Some(Entity::StaggeredText(StaggeredTextEntity {
                id: self.id.clone(),
                transform,
                cache: self.cache.clone(),
                stagger: self.stagger,
                origin,
                text: self.text.clone(),
                animation_data: self.animation_data.clone(),
                letter: StaggeredTextLetter {
                    transform: letter_transform,
                    paint: self.letter.paint.clone(),
                },
            }))
        } else {
            None
        }
    }

    fn sort_keyframes(&mut self) {
        if let Some(x) = &mut self.transform {
            x.sort_keyframes();
        }

        if let Some(x) = &mut self.letter.transform {
            x.sort_keyframes();
        }
    }
}
