import { z } from "zod";

const InterpolationTypeOptions = [
  "Linear",
  "Spring",
  "EasingFunction",
] as const;

const EasingFunctionOptions = [
  "QuintOut",
  "QuintIn",
  "QuintInOut",
  "CircOut",
  "CircIn",
  "CircInOut",
  "CubicOut",
  "CubicIn",
  "CubicInOut",
  "ExpoOut",
  "ExpoIn",
  "ExpoInOut",
  "QuadOut",
  "QuadIn",
  "QuadInOut",
  "QuartOut",
  "QuartIn",
  "QuartInOut",
] as const;

export const EasingFunction = z.enum(EasingFunctionOptions);
export const InterpolationType = z.enum(InterpolationTypeOptions);

export const LinearInterpolation = z.object({
  type: z.literal(InterpolationType.Enum.Linear),
});

export const EasingFunctionInterpolation = z.object({
  type: z.literal(InterpolationType.Enum.EasingFunction),
  easing_function: EasingFunction.default("CircOut"),
});

export const SpringInterpolation = z.object({
  mass: z.number().default(1),
  damping: z.number().default(15),
  stiffness: z.number().default(200),
  type: z.literal(InterpolationType.Enum.Spring),
});

export const Interpolation = z.discriminatedUnion("type", [
  SpringInterpolation,
  EasingFunctionInterpolation,
  LinearInterpolation,
]);
