import { z } from "zod";
import { BoxEntity, EllipseEntity, TextEntity } from "./Entities";
import { AnimatedVec2 } from "./Values";

export const AnimationData = z.object({
  offset: z.number(),
  duration: z.number(),
  visible: z.boolean().optional().default(true),
});

export const AnimatedBoxEntity = BoxEntity.extend({
  position: AnimatedVec2,
  size: AnimatedVec2,
  origin: AnimatedVec2,

  animation_data: AnimationData,
});

export const AnimatedTextEntity = TextEntity.extend({
  origin: AnimatedVec2,
  animation_data: AnimationData,
});

export const AnimatedEllipseEntity = EllipseEntity.extend({
  radius: AnimatedVec2,
  position: AnimatedVec2,
  origin: AnimatedVec2,
  animation_data: AnimationData,
});

export const AnimatedEntity = z.discriminatedUnion("type", [
  AnimatedBoxEntity,
  AnimatedTextEntity,
  AnimatedEllipseEntity,
]);

export const AnimatedEntities = z.array(AnimatedEntity);
