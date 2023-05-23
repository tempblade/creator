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

  const mappedPosition = entity.position.map(
    (val, index) => val - entity.radius[index] * 0.5
  );

  const rect = CanvasKit.XYWHRect(
    mappedPosition[0],
    mappedPosition[1],
    entity.radius[0],
    entity.radius[1]
  );

  canvas.drawOval(rect, paint);
}
