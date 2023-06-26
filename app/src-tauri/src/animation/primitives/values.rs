use super::{
    entities::common::AnimationData,
    keyframe::{Keyframe, Keyframes},
};
use rayon::prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait AnimatedValue<T> {
    fn sort_keyframes(&mut self);
    fn get_value_at_frame(&self, curr_frame: i32, animation_data: &AnimationData, fps: i16) -> T;
    fn get_values_at_frame_range(
        &self,
        start_frame: i32,
        end_frame: i32,
        animation_data: &AnimationData,
        fps: i16,
    ) -> Vec<T>;
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimatedFloat {
    pub keyframes: Keyframes,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimatedFloatVec2 {
    pub keyframes: (AnimatedFloat, AnimatedFloat),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimatedFloatVec3 {
    pub keyframes: (AnimatedFloat, AnimatedFloat, AnimatedFloat),
}

#[tauri::command]
pub fn get_values_at_frame_range_from_animated_float(
    animated_value: AnimatedFloat,
    start_frame: i32,
    end_frame: i32,
    animation_data: AnimationData,
    fps: i16,
) -> Vec<f32> {
    animated_value.get_values_at_frame_range(start_frame, end_frame, &animation_data, fps)
}

#[tauri::command]
pub fn get_values_at_frame_range_from_animated_float_vec2(
    animated_value: AnimatedFloatVec2,
    start_frame: i32,
    end_frame: i32,
    animation_data: AnimationData,
    fps: i16,
) -> Vec<(f32, f32)> {
    animated_value.get_values_at_frame_range(start_frame, end_frame, &animation_data, fps)
}

#[tauri::command]
pub fn get_values_at_frame_range_from_animated_float_vec3(
    animated_value: AnimatedFloatVec3,
    start_frame: i32,
    end_frame: i32,
    animation_data: AnimationData,
    fps: i16,
) -> Vec<(f32, f32, f32)> {
    animated_value.get_values_at_frame_range(start_frame, end_frame, &animation_data, fps)
}

impl AnimatedFloat {
    pub fn new(val: f32) -> AnimatedFloat {
        AnimatedFloat {
            keyframes: Keyframes {
                values: vec![Keyframe {
                    id: Uuid::new_v4().to_string().into(),
                    value: val,
                    offset: 0.0,
                    interpolation: None,
                }],
            },
        }
    }
}

impl AnimatedFloatVec2 {
    pub fn new(x: f32, y: f32) -> AnimatedFloatVec2 {
        AnimatedFloatVec2 {
            keyframes: (AnimatedFloat::new(x), AnimatedFloat::new(y)),
        }
    }
}

impl AnimatedFloatVec3 {
    pub fn new(x: f32, y: f32, z: f32) -> AnimatedFloatVec3 {
        AnimatedFloatVec3 {
            keyframes: (
                AnimatedFloat::new(x),
                AnimatedFloat::new(y),
                AnimatedFloat::new(z),
            ),
        }
    }
}

impl AnimatedValue<f32> for AnimatedFloat {
    fn sort_keyframes(&mut self) {
        self.keyframes.sort();
    }

    fn get_value_at_frame(&self, curr_frame: i32, animation_data: &AnimationData, fps: i16) -> f32 {
        self.keyframes
            .get_value_at_frame(curr_frame, &animation_data, fps)
    }

    fn get_values_at_frame_range(
        &self,
        start_frame: i32,
        end_frame: i32,
        animation_data: &AnimationData,
        fps: i16,
    ) -> Vec<f32> {
        let values = (start_frame..end_frame)
            .into_par_iter()
            .map(|i| self.get_value_at_frame(i, animation_data, fps))
            .collect();

        values
    }
}

impl AnimatedValue<(f32, f32, f32)> for AnimatedFloatVec3 {
    fn sort_keyframes(&mut self) {
        self.keyframes.0.sort_keyframes();
        self.keyframes.1.sort_keyframes();
        self.keyframes.2.sort_keyframes();
    }

    fn get_value_at_frame(
        &self,
        curr_frame: i32,
        animation_data: &AnimationData,
        fps: i16,
    ) -> (f32, f32, f32) {
        let x = self
            .keyframes
            .0
            .get_value_at_frame(curr_frame, animation_data, fps);

        let y = self
            .keyframes
            .1
            .get_value_at_frame(curr_frame, animation_data, fps);

        let z = self
            .keyframes
            .2
            .get_value_at_frame(curr_frame, animation_data, fps);

        return (x, y, z);
    }

    fn get_values_at_frame_range(
        &self,
        start_frame: i32,
        end_frame: i32,
        animation_data: &AnimationData,
        fps: i16,
    ) -> Vec<(f32, f32, f32)> {
        let x =
            self.keyframes
                .0
                .get_values_at_frame_range(start_frame, end_frame, animation_data, fps);
        let y =
            self.keyframes
                .1
                .get_values_at_frame_range(start_frame, end_frame, animation_data, fps);
        let z =
            self.keyframes
                .2
                .get_values_at_frame_range(start_frame, end_frame, animation_data, fps);

        let vectors: Vec<(f32, f32, f32)> = x
            .into_par_iter()
            .enumerate()
            .map(|(index, val_x)| {
                let val_y: f32 = *y.get(index).unwrap();
                let val_z: f32 = *z.get(index).unwrap();

                (val_x, val_y, val_z)
            })
            .collect();

        vectors
    }
}

impl AnimatedValue<(f32, f32)> for AnimatedFloatVec2 {
    fn sort_keyframes(&mut self) {
        self.keyframes.0.sort_keyframes();
        self.keyframes.1.sort_keyframes();
    }

    fn get_value_at_frame(
        &self,
        curr_frame: i32,
        animation_data: &AnimationData,
        fps: i16,
    ) -> (f32, f32) {
        let x = self
            .keyframes
            .0
            .get_value_at_frame(curr_frame, animation_data, fps);

        let y = self
            .keyframes
            .1
            .get_value_at_frame(curr_frame, animation_data, fps);

        return (x, y);
    }

    fn get_values_at_frame_range(
        &self,
        start_frame: i32,
        end_frame: i32,
        animation_data: &AnimationData,
        fps: i16,
    ) -> Vec<(f32, f32)> {
        let x =
            self.keyframes
                .0
                .get_values_at_frame_range(start_frame, end_frame, animation_data, fps);
        let y =
            self.keyframes
                .1
                .get_values_at_frame_range(start_frame, end_frame, animation_data, fps);

        let vectors: Vec<(f32, f32)> = x
            .into_par_iter()
            .enumerate()
            .map(|(index, val_x)| {
                let val_y: f32 = *y.get(index).unwrap();

                (val_x, val_y)
            })
            .collect();

        vectors
    }
}
