use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use super::{
    entities::common::AnimationData,
    interpolations::{interpolate_rendered_keyframes, InterpolationType},
    utils::render_keyframe,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Keyframe {
    pub value: f32,
    pub offset: f32,
    pub id: String,
    pub interpolation: Option<InterpolationType>,
}

#[derive(Debug, Clone)]
pub struct RenderedKeyframe {
    pub absolute_frame: i32,
    pub keyframe: Keyframe,
    pub index: usize,
    pub distance_from_curr: i32,
    pub abs_distance_from_curr: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Keyframes {
    pub values: Vec<Keyframe>,
}

impl Keyframes {
    pub fn get_value_at_frame(
        &self,
        curr_frame: i32,
        animation_data: &AnimationData,
        fps: i16,
    ) -> f32 {
        let keyframe_count = self.values.len();

        if keyframe_count > 0 {
            let mut rendered_keyframes: Vec<RenderedKeyframe> = self
                .values
                .to_vec()
                .into_iter()
                .enumerate()
                .map(|(index, keyframe)| {
                    render_keyframe(keyframe, animation_data, index, curr_frame, fps)
                })
                .collect();

            rendered_keyframes
                .sort_by(|a, b| a.abs_distance_from_curr.cmp(&b.abs_distance_from_curr));

            let closest_keyframe = rendered_keyframes.get(0).unwrap();

            let result = match (closest_keyframe.distance_from_curr).cmp(&0) {
                Ordering::Equal => closest_keyframe.keyframe.value,
                Ordering::Greater => {
                    if closest_keyframe.absolute_frame == curr_frame {
                        return closest_keyframe.keyframe.value;
                    } else {
                        let previous_keyframe =
                            rendered_keyframes.to_vec().into_iter().find(|keyframe| {
                                if closest_keyframe.index > 0 {
                                    keyframe.index == closest_keyframe.index - 1
                                } else {
                                    false
                                }
                            });

                        if let Some(previous_keyframe) = previous_keyframe {
                            let interpolation = match previous_keyframe.keyframe.interpolation {
                                Some(val) => val,
                                None => InterpolationType::Linear,
                            };

                            let interpolated_value = interpolate_rendered_keyframes(
                                &previous_keyframe,
                                closest_keyframe,
                                curr_frame,
                                interpolation,
                                fps,
                            );

                            return interpolated_value;
                        } else {
                            if closest_keyframe.absolute_frame > curr_frame {
                                return closest_keyframe.keyframe.value;
                            } else {
                                return 0.0;
                            }
                        }
                    }
                }
                Ordering::Less => {
                    if closest_keyframe.absolute_frame == curr_frame {
                        return closest_keyframe.keyframe.value;
                    } else {
                        let next_keyframe = rendered_keyframes
                            .to_vec()
                            .into_iter()
                            .find(|keyframe| keyframe.index == closest_keyframe.index + 1);

                        if let Some(next_keyframe) = next_keyframe {
                            let interpolation = match closest_keyframe.keyframe.interpolation {
                                Some(val) => val,
                                None => InterpolationType::Linear,
                            };

                            let interpolated_value = interpolate_rendered_keyframes(
                                closest_keyframe,
                                &next_keyframe,
                                curr_frame,
                                interpolation,
                                fps,
                            );

                            return interpolated_value;
                        } else {
                            if closest_keyframe.absolute_frame < curr_frame {
                                return closest_keyframe.keyframe.value;
                            } else {
                                return 0.0;
                            }
                        }
                    }
                }
            };

            result
        } else {
            0.0
        }
    }

    pub fn sort(&mut self) {
        self.values.sort_by(|a, b| a.offset.total_cmp(&b.offset));
    }
}
