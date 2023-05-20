import { convertToFloat } from "@tempblade/common";
import { Paint as SkPaint, CanvasKit } from "canvaskit-wasm";
import { Paint } from "primitives/Paint";
import { z } from "zod";

export function buildPaintStyle(
  CanvasKit: CanvasKit,
  skPaint: SkPaint,
  paint: z.output<typeof Paint>
) {
  const color = convertToFloat(paint.style.color.value);

  skPaint.setAntiAlias(true);
  skPaint.setColor(color);

  switch (paint.style.type) {
    case "Fill":
      skPaint.setStyle(CanvasKit.PaintStyle.Fill);
      break;

    case "Stroke":
      skPaint.setStyle(CanvasKit.PaintStyle.Stroke);
      skPaint.setStrokeWidth(paint.style.width);
      break;

    default:
      console.error("Paint Style not supported!");
      break;
  }
}
