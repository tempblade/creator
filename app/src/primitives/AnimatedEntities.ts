import { z } from "zod";
import {
  BaseEntity,
  EllipseEntity,
  EntityType,
  RectEntity,
  TextEntity,
} from "./Entities";
import { AnimatedTransform, AnimatedVec2 } from "./Values";
import { TextPaint } from "./Paint";
import { AnimatedProperties } from "./AnimatedProperty";

export const AnimationData = z.object({
  offset: z.number(),
  duration: z.number(),
  visible: z.boolean().optional().default(true),
});

export const AnimatedStaggeredTextEntity = BaseEntity.extend({
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
  origin: AnimatedVec2,
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
  AnimatedStaggeredTextEntity,
  AnimatedEllipseEntity,
]);

export const AnimatedEntities = z.array(AnimatedEntity);

export function animatedTransformToAnimatedProperties(
  animatedTransform: z.input<typeof AnimatedTransform>,
  basePath?: string
): z.input<typeof AnimatedProperties> {
  return [
    {
      animatedValue: animatedTransform.translate,
      label: "Translation",
      propertyPath: basePath
        ? basePath + ".transform.translate"
        : "transform.translate",
    },
    {
      animatedValue: animatedTransform.rotate,
      label: "Rotation",
      propertyPath: basePath
        ? basePath + ".transform.rotate"
        : "transform.rotate",
    },
    {
      animatedValue: animatedTransform.scale,
      label: "Scale",
      propertyPath: basePath
        ? basePath + ".transform.scale"
        : "transform.scale",
    },
    {
      animatedValue: animatedTransform.skew,
      label: "Skew",
      propertyPath: basePath ? basePath + ".transform.skew" : "transform.skew",
    },
  ];
}

export function getAnimatedPropertiesByAnimatedEntity(
  animatedEntity: z.input<typeof AnimatedEntity>
) {
  const animatedProperties: z.input<typeof AnimatedProperties> = [];

  switch (animatedEntity.type) {
    case "Ellipse":
      animatedProperties.push({
        propertyPath: "origin",
        animatedValue: animatedEntity.origin,
        label: "Origin",
      });
      animatedProperties.push({
        propertyPath: "radius",
        animatedValue: animatedEntity.radius,
        label: "Radius",
      });

      if (animatedEntity.transform) {
        animatedProperties.push(
          ...animatedTransformToAnimatedProperties(animatedEntity.transform)
        );
      }

      break;

    case "Rect":
      animatedProperties.push({
        propertyPath: "origin",
        animatedValue: animatedEntity.origin,
        label: "Origin",
      });
      animatedProperties.push({
        propertyPath: "radius",
        animatedValue: animatedEntity.size,
        label: "Radius",
      });

      if (animatedEntity.transform) {
        animatedProperties.push(
          ...animatedTransformToAnimatedProperties(animatedEntity.transform)
        );
      }
      break;

    case "StaggeredText":
      animatedProperties.push({
        propertyPath: "origin",
        animatedValue: animatedEntity.origin,
        label: "Origin",
      });

      if (animatedEntity.transform) {
        animatedProperties.push(
          ...animatedTransformToAnimatedProperties(animatedEntity.transform)
        );
      }

      if (animatedEntity.letter.transform) {
        animatedProperties.push(
          ...animatedTransformToAnimatedProperties(
            animatedEntity.letter.transform,
            "letter"
          )
        );
      }
      break;

    case "Text":
      animatedProperties.push({
        propertyPath: "origin",
        animatedValue: animatedEntity.origin,
        label: "Origin",
      });

      if (animatedEntity.transform) {
        animatedProperties.push(
          ...animatedTransformToAnimatedProperties(animatedEntity.transform)
        );
      }
      break;
  }

  return animatedProperties;
}

export function getAnimatedPropertiesByAnimatedEnties(
  animatedEntities: z.input<typeof AnimatedEntities>
) {
  const animatedProperties: z.input<typeof AnimatedProperties> = [];

  animatedEntities.forEach((aEnt) => {
    animatedProperties.push(...getAnimatedPropertiesByAnimatedEntity(aEnt));
  });
}
