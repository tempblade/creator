import { AnimatedEntity } from "primitives/AnimatedEntities";
import { Color } from "primitives/Paint";
import { Timeline } from "primitives/Timeline";
import { staticAnimatedNumber, staticAnimatedVec2 } from "primitives/Values";
import { z } from "zod";

function buildBox1(
  offset: number,
  color: z.infer<typeof Color>
): z.input<typeof AnimatedEntity> {
  return {
    type: "Box",
    paint: {
      style: {
        type: "Stroke",
        width: 50,
        color,
      },
    },
    size: {
      keyframes: [
        {
          keyframes: {
            values: [
              {
                interpolation: {
                  type: "EasingFunction",
                  easing_function: "CircOut",
                },
                value: 0.0,
                offset: 0.0,
              },
              {
                interpolation: {
                  type: "Linear",
                },
                value: 1280.0,
                offset: 4.0,
              },
            ],
          },
        },
        staticAnimatedNumber(720),
      ],
    },
    origin: staticAnimatedVec2(1280 / 2, 720 / 2),
    position: staticAnimatedVec2(0, 0),
    animation_data: {
      offset,
      duration: 10.0,
    },
  };
}

function buildBox(
  offset: number,
  color: z.infer<typeof Color>
): z.input<typeof AnimatedEntity> {
  return {
    type: "Box",
    paint: {
      style: {
        type: "Fill",
        color,
      },
    },
    size: {
      keyframes: [
        {
          keyframes: {
            values: [
              {
                interpolation: {
                  type: "Linear",
                },
                value: 1280.0,
                offset: 0.0,
              },
            ],
          },
        },
        {
          keyframes: {
            values: [
              {
                interpolation: {
                  type: "EasingFunction",
                  easing_function: "CircOut",
                },
                value: 0.0,
                offset: 0.0,
              },
              {
                interpolation: {
                  type: "Linear",
                },
                value: 720.0,
                offset: 4.0,
              },
            ],
          },
        },
      ],
    },
    origin: staticAnimatedVec2(0, -720),
    position: staticAnimatedVec2(1280 / 2, 720 / 2),
    animation_data: {
      offset,
      duration: 10.0,
    },
  };
}

function buildText(
  text: string,
  offset: number,
  size: number,
  y_offset: number,
  color: z.infer<typeof Color>
): z.input<typeof AnimatedEntity> {
  return {
    type: "Text",
    paint: {
      style: {
        type: "Fill",
        color,
      },
      size,
      align: "Center",
    },
    text,
    animation_data: {
      offset,
      duration: 5.0,
    },
    origin: {
      keyframes: [
        {
          keyframes: {
            values: [
              {
                interpolation: {
                  type: "Spring",
                  mass: 1,
                  stiffness: 100,
                  damping: 15,
                },
                value: (1280 / 2) * -1 - 300,
                offset: 0.0,
              },
              {
                interpolation: {
                  type: "EasingFunction",
                  easing_function: "QuartOut",
                },
                value: 1280 / 2,
                offset: 5.0,
              },
            ],
          },
        },
        staticAnimatedNumber(720 / 2 + y_offset),
      ],
    },
  };
}

export const EXAMPLE_ANIMATED_ENTITIES: Array<z.input<typeof AnimatedEntity>> =
  [
    buildText("Kleine Dumpfkopf!", 1.0, 80, -30, {
      value: [255, 255, 255, 1.0],
    }),
    buildText("Wie gehts?", 1.5, 40, 30, { value: [255, 255, 255, 1.0] }),
    buildBox(0.6, { value: [30, 30, 30, 1.0] }),
    buildBox(0.4, { value: [20, 20, 20, 1.0] }),
    buildBox(0.2, { value: [10, 10, 10, 1.0] }),
    buildBox(0, { value: [0, 0, 0, 1.0] }),
  ];

const ExampleTimeline: z.input<typeof Timeline> = {
  size: [1920, 1080],
  duration: 10.0,
  render_state: {
    curr_frame: 20,
  },
  fps: 60,
  entities: EXAMPLE_ANIMATED_ENTITIES,
};

export { ExampleTimeline };
