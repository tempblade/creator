import { z } from "zod";
import { Vec2 } from "./Values";
import { Paint, TextPaint } from "./Paint";

const EntityTypeOptions = ["Text", "Ellipse", "Rect", "StaggeredText"] as const;

export const EntityType = z.enum(EntityTypeOptions);

export const GeometryEntity = z.object({
  paint: Paint,
});

export const Transform = z.object({
  skew: Vec2,
  rotate: Vec2,
  translate: Vec2,
  scale: Vec2,
});

export const StaggeredText = z.object({
  letter: z.object({
    position: Vec2,
    transform: Transform,
    paint: TextPaint,
  }),
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

export const TextEntity = z.object({
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
]);

export const Entities = z.array(Entity);
