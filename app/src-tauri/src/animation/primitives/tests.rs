#[cfg(test)]
use crate::animation::primitives::{
    entities::AnimationData,
    interpolations::{calculate_spring_value, SpringProperties},
    keyframe::{Keyframe, Keyframes},
    utils::timestamp_to_frame,
};

#[test]
fn interpolates_the_input() {
    use crate::animation::primitives::{
        interpolations::{interpolate_rendered_keyframes, InterpolationType},
        keyframe::{Keyframe, Keyframes, RenderedKeyframe},
        utils::render_keyframe,
    };

    let animation_data = AnimationData {
        offset: 0.0,
        duration: 3.0,
        visible: true,
    };

    let fps = 60;

    let keyframes1 = Keyframes {
        values: vec![
            Keyframe {
                value: 0.0,
                offset: 0.0,
                interpolation: None,
            },
            Keyframe {
                value: 100.0,
                offset: 1.0,
                interpolation: None,
            },
            Keyframe {
                value: 300.0,
                offset: 3.0,
                interpolation: None,
            },
        ],
    };

    let keyframes2 = Keyframes {
        values: vec![
            Keyframe {
                value: -100.0,
                offset: 0.0,
                interpolation: None,
            },
            Keyframe {
                value: 0.0,
                offset: 1.0,
                interpolation: None,
            },
        ],
    };

    let rendered_keyframes1: Vec<RenderedKeyframe> = keyframes1
        .values
        .into_iter()
        .enumerate()
        .map(|(index, keyframe)| {
            let rendered_keyframe = render_keyframe(keyframe, &animation_data, index, 120, 60);
            rendered_keyframe
        })
        .collect();

    let rendered_keyframes2: Vec<RenderedKeyframe> = keyframes2
        .values
        .into_iter()
        .enumerate()
        .map(|(index, keyframe)| {
            let rendered_keyframe = render_keyframe(keyframe, &animation_data, index, 120, 60);
            rendered_keyframe
        })
        .collect();

    let val1 = interpolate_rendered_keyframes(
        rendered_keyframes1.get(1).unwrap(),
        rendered_keyframes1.get(2).unwrap(),
        120,
        InterpolationType::Linear,
        fps,
    );

    let _val2 = interpolate_rendered_keyframes(
        rendered_keyframes2.get(0).unwrap(),
        rendered_keyframes2.get(1).unwrap(),
        30,
        InterpolationType::Linear,
        fps,
    );

    assert_eq!(val1, 200.0);
    //println!("{0}", val2);
}

#[test]
fn calculates_the_spring_value() {
    let _fps = 60;
    let previous_value = 0.0;
    let next_value = 500.0;

    let mut spring_props = SpringProperties {
        mass: 1.0,        // Mass of the object attached to the spring
        stiffness: 100.0, // Stiffness of the spring
        damping: 10.0,    // Damping factor of the spring
    };

    let value1 =
        calculate_spring_value(100, previous_value, next_value, 100, 300, &mut spring_props);
    let value2 =
        calculate_spring_value(150, previous_value, next_value, 100, 300, &mut spring_props);
    let value3 =
        calculate_spring_value(200, previous_value, next_value, 100, 300, &mut spring_props);

    println!("{value1}");
    println!("{value2}");
    println!("{value3}");
}

#[test]
fn converts_timestamp_to_frame() {
    let frame1 = timestamp_to_frame(0.0, 60);
    let frame2 = timestamp_to_frame(1.0, 60);
    let frame3 = timestamp_to_frame(1.5, 60);

    assert_eq!(frame1, 0);
    assert_eq!(frame2, 60);
    assert_eq!(frame3, 90);
}

#[test]
fn gets_value_at_frame() {
    let animation_data = AnimationData {
        offset: 0.0,
        duration: 5.0,
        visible: true,
    };

    let fps = 60;

    let keyframes = Keyframes {
        values: vec![
            Keyframe {
                value: 0.0,
                offset: 0.0,
                interpolation: None,
            },
            Keyframe {
                value: 100.0,
                offset: 1.0,
                interpolation: None,
            },
            Keyframe {
                value: 300.0,
                offset: 3.0,
                interpolation: None,
            },
        ],
    };

    let value1 = keyframes.get_value_at_frame(50, &animation_data, fps);
    let value2 = keyframes.get_value_at_frame(90, &animation_data, fps);
    let value3 = keyframes.get_value_at_frame(120, &animation_data, fps);
    let value4 = keyframes.get_value_at_frame(180, &animation_data, fps);
    let value5 = keyframes.get_value_at_frame(220, &animation_data, fps);
    println!("value1: {0}", value1);
    println!("value2: {0}", value2);
    println!("value3: {0}", value3);
    println!("value4: {0}", value4);
    println!("value5: {0}", value5);
}
