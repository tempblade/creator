use super::{
    entities::common::AnimationData,
    keyframe::{Keyframe, RenderedKeyframe},
};

pub fn timestamp_to_frame(timestamp: f32, fps: i16) -> i32 {
    return (timestamp * fps as f32).round() as i32;
}

pub fn render_keyframe(
    keyframe: Keyframe,
    animation_data: &AnimationData,
    index: usize,
    curr_frame: i32,
    fps: i16,
) -> RenderedKeyframe {
    let animation_start_frame = timestamp_to_frame(animation_data.offset, fps);
    let frame_offset = timestamp_to_frame(keyframe.offset, fps);
    let absolute_frame = animation_start_frame + frame_offset;
    let distance_from_curr = absolute_frame - curr_frame;

    RenderedKeyframe {
        absolute_frame,
        keyframe,
        index,
        distance_from_curr,
        abs_distance_from_curr: distance_from_curr.abs(),
    }
}
