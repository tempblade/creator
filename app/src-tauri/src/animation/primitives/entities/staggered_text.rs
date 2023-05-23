use super::common::{Animateable, AnimationData, Drawable, Entity};
use crate::animation::{
    primitives::{
        paint::TextPaint,
        transform::{AnimatedTransform, Transform},
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
    pub transform: Option<Transform>,
    pub paint: TextPaint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimatedStaggeredTextEntity {
    pub text: String,
    pub stagger: f32,
    pub animation_data: AnimationData,
    pub transform: Option<AnimatedTransform>,
    pub letter: AnimatedStaggeredTextLetter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaggeredTextEntity {
    pub text: String,
    pub stagger: f32,
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

            let letter_transform: Option<Transform> = match self.letter.transform.clone() {
                Some(mut val) => Some(val.calculate(timeline, &self.animation_data)),
                None => None,
            };

            Some(Entity::StaggeredText(StaggeredTextEntity {
                transform,
                stagger: self.stagger,
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
