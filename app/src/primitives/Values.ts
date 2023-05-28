import { z } from "zod";
import { Keyframes } from "./Keyframe";
import { Interpolation } from "./Interpolation";

export const Vec2 = z.array(z.number()).length(2);
export const Vec3 = z.array(z.number()).length(3);

export const AnimatedNumber = z.object({
  keyframes: Keyframes,
});

export const AnimatedVec2 = z.object({
  keyframes: z.array(AnimatedNumber).length(2),
});

export const AnimatedVec3 = z.object({
  keyframes: z.array(AnimatedNumber).length(3),
});

export function staticAnimatedNumber(
  number: number
): z.infer<typeof AnimatedNumber> {
  return {
    keyframes: {
      values: [
        {
          interpolation: {
            type: "Linear",
          },
          value: number,
          offset: 0,
        },
      ],
    },
  };
}

export function staticAnimatedVec2(
  x: number,
  y: number
): z.infer<typeof AnimatedVec2> {
  return {
    keyframes: [staticAnimatedNumber(x), staticAnimatedNumber(y)],
  };
}

export function staticAnimatedVec3(
  x: number,
  y: number,
  z: number
): z.infer<typeof AnimatedVec2> {
  return {
    keyframes: [
      staticAnimatedNumber(x),
      staticAnimatedNumber(y),
      staticAnimatedNumber(z),
    ],
  };
}
