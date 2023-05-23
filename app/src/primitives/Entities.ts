import { z } from "zod";
import { Vec2 } from "./Values";
import { Paint, TextPaint } from "./Paint";

const EntityTypeOptions = ["Text", "Ellipse", "Rect", "StaggeredText"] as const;

export const EntityType = z.enum(EntityTypeOptions);

export const Transform = z.object({
  skew: Vec2,
  rotate: Vec2,
  translate: Vec2,
  scale: Vec2,
});

export const Cache = z.object({
  valid: z.boolean().optional().default(false),
});

export const BaseEntity = z.object({
  id: z.string(),
  cache: Cache,
});

export const GeometryEntity = BaseEntity.extend({
  paint: Paint,
});

export const StaggeredText = BaseEntity.extend({
  letter: z.object({
    transform: z.array(Transform).optional(),
    paint: TextPaint,
  }),
  origin: Vec2,
  text: z.string(),
  type: z.literal(EntityType.Enum.StaggeredText),
});

export const RectEntity = GeometryEntity.extend({
  type: z.literal(EntityType.Enum.Rect),
  size: Vec2,
  position: Vec2,
  origin: Vec2,
  transform: z.nullable(Transform),
});

export const EllipseEntity = GeometryEntity.extend({
  type: z.literal(EntityType.Enum.Ellipse),
  radius: Vec2,
  position: Vec2,
  origin: Vec2,
  transform: z.nullable(Transform),
});

export const TextEntity = BaseEntity.extend({
  type: z.literal(EntityType.Enum.Text),
  paint: TextPaint,
  origin: Vec2,
  text: z.string(),
  transform: z.nullable(Transform),
});

export const Entity = z.discriminatedUnion("type", [
  RectEntity,
  EllipseEntity,
  TextEntity,
  StaggeredText,
]);

export const Entities = z.array(Entity);
