import { z } from "zod";
import { Keyframes } from "./Keyframe";
import { v4 as uuid } from "uuid";

export const Vec2 = z.array(z.number()).length(2);
export const Vec3 = z.array(z.number()).length(3);

const ValueTypeOptions = ["Vec2", "Vec3", "Number"] as const;

export const ValueType = z.enum(ValueTypeOptions);

export const AnimatedNumber = z.object({
  keyframes: Keyframes,
  type: z.literal(ValueType.Enum.Number),
});

export const AnimatedVec2 = z.object({
  keyframes: z.array(AnimatedNumber).length(2),
  type: z.literal(ValueType.Enum.Vec2),
});

export const AnimatedVec3 = z.object({
  keyframes: z.array(AnimatedNumber).length(3),
  type: z.literal(ValueType.Enum.Vec3),
});

export const AnimatedTransform = z.object({
  type: z.literal("Transform"),
  /** Translates by the given animated vec2 */
  translate: AnimatedVec2,
  /** Skews by the given animated vec2 */
  skew: AnimatedVec2,
  /** Rotates by the given animated vec3 */
  rotate: AnimatedVec3,
  /** Scales on the x and y axis by the given animated vec2 */
  scale: AnimatedVec2,
});

export const AnimatedValue = z.discriminatedUnion("type", [
  AnimatedNumber,
  AnimatedVec2,
  AnimatedVec3,
  AnimatedTransform,
]);

export function staticAnimatedNumber(
  number: number
): z.infer<typeof AnimatedNumber> {
  return {
    type: ValueType.Enum.Number,
    keyframes: {
      values: [
        {
          id: uuid(),
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
    type: ValueType.Enum.Vec2,
    keyframes: [staticAnimatedNumber(x), staticAnimatedNumber(y)],
  };
}

export function staticAnimatedVec3(
  x: number,
  y: number,
  z: number
): z.infer<typeof AnimatedVec3> {
  return {
    type: ValueType.Enum.Vec3,
    keyframes: [
      staticAnimatedNumber(x),
      staticAnimatedNumber(y),
      staticAnimatedNumber(z),
    ],
  };
}

export function staticAnimatedTransform(
  translate: [number, number],
  scale: [number, number],
  rotate: [number, number, number],
  skew: [number, number]
): z.input<typeof AnimatedTransform> {
  return {
    type: "Transform",
    translate: staticAnimatedVec2(...translate),
    scale: staticAnimatedVec2(...scale),
    rotate: staticAnimatedVec3(...rotate),
    skew: staticAnimatedVec2(...skew),
  };
}
