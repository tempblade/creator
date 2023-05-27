use std::str::FromStr;

use crate::animation::primitives::{
    interpolations::{EasingFunction, InterpolationType, SpringProperties},
    keyframe::{Keyframe, Keyframes},
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::primitives::{
    entities::{
        common::{AnimatedEntity, AnimationData, Cache, Entity},
        rect::AnimatedRectEntity,
        text::AnimatedTextEntity,
    },
    paint::{Color, FillStyle, Paint, PaintStyle, StrokeStyle, TextAlign, TextPaint},
    values::{AnimatedFloat, AnimatedFloatVec2},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub title: String,
    pub sub_title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    entities: Vec<AnimatedEntity>,
    pub render_state: RenderState,
    pub duration: f32,
    pub fps: i16,
    pub size: (i32, i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderState {
    pub curr_frame: i32,
}

impl Timeline {
    fn calculate(&self) -> Vec<Entity> {
        let mut entities = self.entities.clone();

        let entities = entities
            .par_iter_mut()
            .map(|entity| entity.calculate(self))
            .filter(|entity| entity.is_some())
            .map(|entity| entity.unwrap())
            .collect();

        return entities;
    }
}

fn build_bg(offset: f32, paint: Paint, size: (i32, i32)) -> AnimatedRectEntity {
    let bg_box = AnimatedRectEntity {
        id: String::from_str("1").unwrap(),
        paint,
        animation_data: AnimationData {
            offset: 0.0 + offset,
            duration: 5.0,
            visible: true,
        },
        cache: Cache { valid: false },
        transform: None,
        origin: AnimatedFloatVec2::new(1280.0 / 2.0, 720.0 / 2.0),
        position: AnimatedFloatVec2 {
            keyframes: (
                AnimatedFloat {
                    keyframes: Keyframes {
                        values: vec![
                            Keyframe {
                                value: (size.0 * -1) as f32,
                                offset: 0.0,
                                interpolation: Some(InterpolationType::EasingFunction(
                                    EasingFunction::QuintOut,
                                )),
                            },
                            Keyframe {
                                value: 0.0,
                                offset: 5.0,
                                interpolation: None,
                            },
                        ],
                    },
                },
                AnimatedFloat {
                    keyframes: Keyframes {
                        values: vec![Keyframe {
                            value: 0.0,
                            offset: 0.0,
                            interpolation: None,
                        }],
                    },
                },
            ),
        },
        size: AnimatedFloatVec2 {
            keyframes: (
                AnimatedFloat {
                    keyframes: Keyframes {
                        values: vec![Keyframe {
                            interpolation: None,
                            value: size.0 as f32,
                            offset: 0.0,
                        }],
                    },
                },
                AnimatedFloat {
                    keyframes: Keyframes {
                        values: vec![Keyframe {
                            value: size.1 as f32,
                            offset: 0.0,
                            interpolation: None,
                        }],
                    },
                },
            ),
        },
    };
    return bg_box;
}

#[tauri::command]
pub fn calculate_timeline_entities_at_frame(timeline: Timeline) -> Vec<Entity> {
    timeline.calculate()
}

pub fn test_timeline_entities_at_frame(
    render_state: RenderState,
    size: (i32, i32),
    input: Input,
) -> Vec<Entity> {
    let rect1_paint = Paint {
        style: PaintStyle::Fill(FillStyle {
            color: Color::new(34, 189, 58, 1.0),
        }),
    };

    let rect2_paint = Paint {
        style: PaintStyle::Fill(FillStyle {
            color: Color::new(23, 178, 28, 1.0),
        }),
    };

    let rect3_paint = Paint {
        style: PaintStyle::Fill(FillStyle {
            color: Color::new(43, 128, 98, 1.0),
        }),
    };

    let title_paint = TextPaint {
        style: PaintStyle::Stroke(StrokeStyle {
            color: Color::new(0, 0, 0, 1.0),
            width: 10.0,
        }),
        fontName: "Arial".to_string(),
        align: TextAlign::Center,
        size: 20.0,
    };

    let sub_title_paint = TextPaint {
        style: PaintStyle::Fill(FillStyle {
            color: Color::new(0, 0, 0, 1.0),
        }),
        fontName: "Arial".to_string(),
        align: TextAlign::Center,
        size: 10.0,
    };

    let timeline = Timeline {
        fps: 60,
        duration: 5.0,
        size,
        entities: vec![
            AnimatedEntity::Rect(build_bg(0.0, rect1_paint, size)),
            AnimatedEntity::Rect(build_bg(0.5, rect2_paint, size)),
            AnimatedEntity::Rect(build_bg(1.0, rect3_paint, size)),
            AnimatedEntity::Text(AnimatedTextEntity {
                id: String::from_str("2").unwrap(),
                paint: title_paint,
                cache: Cache { valid: false },
                text: input.title,
                animation_data: AnimationData {
                    offset: 0.0,
                    duration: 6.0,
                    visible: true,
                },
                transform: None,
                origin: AnimatedFloatVec2 {
                    keyframes: (
                        AnimatedFloat {
                            keyframes: Keyframes {
                                values: vec![
                                    Keyframe {
                                        value: 0.0,
                                        offset: 0.0,
                                        interpolation: Some(InterpolationType::Spring(
                                            SpringProperties {
                                                mass: 1.0,
                                                damping: 20.0,
                                                stiffness: 200.0,
                                            },
                                        )),
                                    },
                                    Keyframe {
                                        value: (size.0 / 2) as f32,
                                        offset: 2.0,
                                        interpolation: None,
                                    },
                                ],
                            },
                        },
                        AnimatedFloat {
                            keyframes: Keyframes {
                                values: vec![Keyframe {
                                    value: (size.1 / 2) as f32,
                                    offset: 0.0,
                                    interpolation: None,
                                }],
                            },
                        },
                    ),
                },
            }),
            AnimatedEntity::Text(AnimatedTextEntity {
                id: String::from_str("3").unwrap(),
                paint: sub_title_paint,
                text: input.sub_title,
                cache: Cache { valid: false },
                animation_data: AnimationData {
                    offset: 0.5,
                    duration: 6.0,
                    visible: true,
                },
                transform: None,
                origin: AnimatedFloatVec2 {
                    keyframes: (
                        AnimatedFloat {
                            keyframes: Keyframes {
                                values: vec![
                                    Keyframe {
                                        value: 0.0,
                                        offset: 0.0,
                                        interpolation: Some(InterpolationType::Spring(
                                            SpringProperties {
                                                mass: 1.0,
                                                damping: 20.0,
                                                stiffness: 200.0,
                                            },
                                        )),
                                    },
                                    Keyframe {
                                        value: (size.0 / 2) as f32,
                                        offset: 2.0,
                                        interpolation: None,
                                    },
                                ],
                            },
                        },
                        AnimatedFloat {
                            keyframes: Keyframes {
                                values: vec![Keyframe {
                                    value: ((size.1 / 2) as f32) + 80.0,
                                    offset: 0.0,
                                    interpolation: None,
                                }],
                            },
                        },
                    ),
                },
            }),
        ],
        render_state: render_state,
    };

    timeline.calculate()
}
