import { z } from "zod";
import { Vec2 } from "./Values";
import { Paint, TextPaint } from "./Paint";

const EntityTypeOptions = ["Text", "Ellipse", "Box"] as const;

export const EntityType = z.enum(EntityTypeOptions);

export const GeometryEntity = z.object({
  paint: Paint,
});

export const BoxEntity = GeometryEntity.extend({
  type: z.literal(EntityType.Enum.Box),
  size: Vec2,
  position: Vec2,
  origin: Vec2,
});

export const EllipseEntity = GeometryEntity.extend({
  type: z.literal(EntityType.Enum.Ellipse),
  radius: Vec2,
  position: Vec2,
  origin: Vec2,
});

export const TextEntity = z.object({
  type: z.literal(EntityType.Enum.Text),
  paint: TextPaint,
  origin: Vec2,
  text: z.string(),
});

export const Entity = z.discriminatedUnion("type", [
  BoxEntity,
  EllipseEntity,
  TextEntity,
]);

export const Entities = z.array(Entity);
