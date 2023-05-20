import { z } from "zod";
import { Interpolation } from "./Interpolation";

export const Keyframe = z.object({
  value: z.number(),
  offset: z.number(),
  interpolation: z.optional(Interpolation),
});

export const Keyframes = z.object({
  values: z.array(Keyframe),
});
