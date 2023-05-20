import { AnimatedEntity } from "primitives/AnimatedEntities";
import { Keyframe } from "primitives/Keyframe";
import { AnimatedNumber, AnimatedVec2 } from "primitives/Values";
import { z } from "zod";

export function flattenAnimatedNumberKeyframes(
  aNumber: z.input<typeof AnimatedNumber>
): Array<z.input<typeof Keyframe>> {
  return aNumber.keyframes.values;
}

export function flattenAnimatedVec2Keyframes(
  aVec2: z.input<typeof AnimatedVec2>
): Array<z.input<typeof Keyframe>> {
  const keyframes: Array<z.input<typeof Keyframe>> = [
    ...flattenAnimatedNumberKeyframes(aVec2.keyframes[0]),
    ...flattenAnimatedNumberKeyframes(aVec2.keyframes[1]),
  ];

  return keyframes;
}

export function flattenedKeyframesByEntity(
  entity: z.input<typeof AnimatedEntity>
): Array<z.input<typeof Keyframe>> {
  const keyframes: Array<z.input<typeof Keyframe>> = [];

  switch (entity.type) {
    case "Text":
      keyframes.push(...flattenAnimatedVec2Keyframes(entity.origin));
      break;
    case "Box":
      keyframes.push(...flattenAnimatedVec2Keyframes(entity.position));
      keyframes.push(...flattenAnimatedVec2Keyframes(entity.size));
      break;
    case "Ellipse":
      keyframes.push(...flattenAnimatedVec2Keyframes(entity.position));
      keyframes.push(...flattenAnimatedVec2Keyframes(entity.radius));
      break;
    default:
      break;
  }

  return keyframes;
}
