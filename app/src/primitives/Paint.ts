import { z } from "zod";

export const Color = z.object({
  value: z.array(z.number().min(0).max(255)).max(4),
});

const PaintStyleTypeOptions = ["Fill", "Stroke"] as const;

export const PaintStyleType = z.enum(PaintStyleTypeOptions);

const ColorWithDefault = Color.optional().default({ value: [0, 0, 0, 1] });

export const StrokeStyle = z.object({
  width: z.number().min(0).optional().default(10),
  color: ColorWithDefault,
  type: z.literal(PaintStyleType.Enum.Stroke),
});

export const FillStyle = z.object({
  color: ColorWithDefault,
  type: z.literal(PaintStyleType.Enum.Fill),
});

export const TextAlign = z.enum(["Left", "Center", "Right"]);

export const PaintStyle = z.discriminatedUnion("type", [
  StrokeStyle,
  FillStyle,
]);

export const Paint = z.object({
  style: PaintStyle,
});

export const TextPaint = z.object({
  style: PaintStyle,
  align: TextAlign,
  fontName: z.string().default("Helvetica-Bold"),
  size: z.number().min(0),
});

/* const NestedFillStyle = FillStyle.omit({ type: true }).default({});
const NestedStrokeStyle = StrokeStyle.omit({ type: true }).default({});

export const StrokeAndFillStyle = z.object({
  color: ColorWithDefault,
  type: z.literal(PaintStyleType.Enum.StrokeAndFill),
  fill: NestedFillStyle,
  stroke: NestedStrokeStyle,
}); */
