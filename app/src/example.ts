import { AnimatedEntity } from "primitives/AnimatedEntities";
import { Color } from "primitives/Paint";
import { Timeline } from "primitives/Timeline";
import {
  staticAnimatedNumber,
  staticAnimatedVec2,
  staticAnimatedVec3,
} from "primitives/Values";
import { z } from "zod";
import { v4 as uuid } from "uuid";

function buildRect1(
  offset: number,
  color: z.infer<typeof Color>
): z.input<typeof AnimatedEntity> {
  return {
    id: uuid(),
    cache: {},
    type: "Rect",
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

function buildRect(
  offset: number,
  color: z.infer<typeof Color>
): z.input<typeof AnimatedEntity> {
  return {
    type: "Rect",
    id: uuid(),
    cache: {},
    paint: {
      style: {
        type: "Fill",
        color,
      },
    },
    size: staticAnimatedVec2(1280, 720),
    origin: staticAnimatedVec2(0, -720),
    position: staticAnimatedVec2(1280 / 2, 720 / 2),
    transform: {
      translate: staticAnimatedVec2(0, 0),
      rotate: staticAnimatedVec2(0, 0),
      skew: staticAnimatedVec2(0, 0),
      scale: {
        keyframes: [
          {
            keyframes: {
              values: [
                {
                  interpolation: {
                    type: "Linear",
                  },
                  value: 1.0,
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
                  value: 1.0,
                  offset: 4.0,
                },
              ],
            },
          },
        ],
      },
    },
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
    id: uuid(),
    cache: {},
    paint: {
      style: {
        type: "Fill",
        color,
      },
      font_name: "Arial",
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

function buildStaggeredText(
  text: string,
  offset: number,
  color: z.input<typeof Color>
): z.input<typeof AnimatedEntity> {
  return {
    type: "StaggeredText",
    text,
    cache: { valid: false },
    id: uuid(),
    origin: staticAnimatedVec2(1280 / 2, 720 / 2),
    transform: {
      translate: staticAnimatedVec2(0, 0),
      rotate: staticAnimatedVec2(0, 0),
      skew: staticAnimatedVec2(0, 0),
      scale: staticAnimatedVec2(1, 1),
    },
    animation_data: {
      offset,
      duration: 5.0,
    },
    stagger: 0.05,
    letter: {
      paint: {
        font_name: "Arial",
        style: {
          type: "Fill",
          color,
        },
        size: 90,
        align: "Center",
      },
      transform: {
        translate: staticAnimatedVec2(0, 0),
        rotate: staticAnimatedVec3(0, 0, 45),
        skew: staticAnimatedVec2(0, 0),
        scale: {
          keyframes: [
            {
              keyframes: {
                values: [
                  {
                    interpolation: {
                      type: "Spring",
                      stiffness: 200,
                      mass: 1,
                      damping: 15,
                    },
                    value: 5.0,
                    offset: 0.0,
                  },
                  {
                    interpolation: {
                      type: "Linear",
                    },
                    value: 1.0,
                    offset: 4.0,
                  },
                ],
              },
            },
            {
              keyframes: {
                values: [
                  {
                    interpolation: {
                      type: "Spring",
                      stiffness: 300,
                      mass: 1,
                      damping: 15,
                    },
                    value: -10.0,
                    offset: 0.0,
                  },
                  {
                    interpolation: {
                      type: "Linear",
                    },
                    value: 1.0,
                    offset: 4.0,
                  },
                ],
              },
            },
          ],
        },
      },
    },
  };
}

export const EXAMPLE_ANIMATED_ENTITIES: Array<z.input<typeof AnimatedEntity>> =
  [
    buildStaggeredText("Ehrenmann?", 2.0, {
      value: [255, 255, 255, 1.0],
    }),
    // buildText("Wie gehts?", 2.5, 40, 40, { value: [200, 200, 200, 1.0] }),
    buildRect(0.6, { value: [30, 30, 30, 1.0] }),
    buildRect(0.4, { value: [20, 20, 20, 1.0] }),
    buildRect(0.2, { value: [10, 10, 10, 1.0] }),
    buildRect(0, { value: [0, 0, 0, 1.0] }),
  ];

export const EXAMPLE_ANIMATED_ENTITIES_2: Array<
  z.input<typeof AnimatedEntity>
> = [
  buildText("Kleine Dumpfkopf!", 1.0, 80, -30, {
    value: [255, 255, 255, 1.0],
  }),
  // buildText("Wie gehts?", 1.5, 40, 30, { value: [255, 255, 255, 1.0] }),
  buildRect(0.8, { value: [40, 40, 40, 1.0] }),
  buildRect(0.6, { value: [30, 30, 30, 1.0] }),
  buildRect(0.4, { value: [20, 20, 20, 1.0] }),
  buildRect(0.2, { value: [10, 10, 10, 1.0] }),
  buildRect(0, { value: [0, 0, 0, 1.0] }),
];

const ExampleTimeline: z.input<typeof Timeline> = {
  size: [1920, 1080],
  duration: 10.0,
  render_state: {
    curr_frame: 20,
  },
  fps: 120,
  entities: EXAMPLE_ANIMATED_ENTITIES,
};

export { ExampleTimeline };
