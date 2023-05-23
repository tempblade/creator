import { z } from "zod";
import { EllipseEntity, EntityType, RectEntity, TextEntity } from "./Entities";
import { AnimatedVec2 } from "./Values";
import { TextPaint } from "./Paint";

export const AnimationData = z.object({
  offset: z.number(),
  duration: z.number(),
  visible: z.boolean().optional().default(true),
});

export const AnimatedTransform = z.object({
  /** Translates by the given animated vec2 */
  translate: AnimatedVec2,
  /** Skews by the given animated vec2 */
  skew: AnimatedVec2,
  /** Rotates by the given animated vec2 */
  rotate: AnimatedVec2,
  /** Scales on the x and y axis by the given animated vec2 */
  scale: AnimatedVec2,
});

export const AnimatedStaggeredText = z.object({
  /** Transform applied to the whole layer. */
  transform: AnimatedTransform,
  /** The staggered delay that is applied for each letter. Gets multiplied by the index of the letter. */
  stagger: z.number().min(0),
  /** These properties get applied to each letter */
  letter: z.object({
    transform: AnimatedTransform,
    paint: TextPaint,
  }),
  text: z.string(),
  animation_data: AnimationData,
  type: z.literal(EntityType.Enum.StaggeredText),
});

export const AnimatedRectEntity = RectEntity.extend({
  position: AnimatedVec2,
  size: AnimatedVec2,
  origin: AnimatedVec2,
  transform: AnimatedTransform.optional(),
  animation_data: AnimationData,
});

export const AnimatedTextEntity = TextEntity.extend({
  origin: AnimatedVec2,
  transform: AnimatedTransform.optional(),
  animation_data: AnimationData,
});

export const AnimatedEllipseEntity = EllipseEntity.extend({
  radius: AnimatedVec2,
  position: AnimatedVec2,
  origin: AnimatedVec2,
  transform: AnimatedTransform.optional(),
  animation_data: AnimationData,
});

export const AnimatedEntity = z.discriminatedUnion("type", [
  AnimatedRectEntity,
  AnimatedTextEntity,
  AnimatedStaggeredText,
  AnimatedEllipseEntity,
]);

export const AnimatedEntities = z.array(AnimatedEntity);
