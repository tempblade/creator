import { z } from "zod";
import { Interpolation } from "./Interpolation";

export const Keyframe = z.object({
  id: z.string().uuid(),
  value: z.number(),
  offset: z.number(),
  interpolation: z.optional(Interpolation),
});

export const Keyframes = z.object({
  values: z.array(Keyframe),
});
