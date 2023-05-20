import { convertToFloat } from "@tempblade/common";
import { Canvas, CanvasKit } from "canvaskit-wasm";
import { EllipseEntity } from "primitives/Entities";
import { z } from "zod";
import { buildPaintStyle } from "./paint";

export default function drawEllipse(
  CanvasKit: CanvasKit,
  canvas: Canvas,
  entity: z.infer<typeof EllipseEntity>
) {
  const paint = new CanvasKit.Paint();

  buildPaintStyle(CanvasKit, paint, entity.paint);

  const rect = CanvasKit.XYWHRect(
    entity.position[0],
    entity.position[1],
    entity.radius[0],
    entity.radius[1]
  );

  canvas.drawOval(rect, paint);
}
