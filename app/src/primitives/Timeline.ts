import { z } from "zod";
import { AnimatedEntities } from "./AnimatedEntities";

export const RenderState = z.object({
  curr_frame: z.number(),
});

export const Timeline = z.object({
  entities: AnimatedEntities,
  render_state: RenderState,
  duration: z.number(),
  fps: z.number().int(),
  size: z.array(z.number().int()).length(2),
});
