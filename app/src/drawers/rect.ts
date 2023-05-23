import { Canvas, CanvasKit } from "canvaskit-wasm";
import { z } from "zod";
import { RectEntity } from "primitives/Entities";
import { buildPaintStyle } from "./paint";

export default function drawRect(
  CanvasKit: CanvasKit,
  canvas: Canvas,
  entity: z.infer<typeof RectEntity>
) {
  canvas.save();

  const paint = new CanvasKit.Paint();

  buildPaintStyle(CanvasKit, paint, entity.paint);

  const mappedPosition = entity.position.map(
    (val, index) => val - entity.size[index] * 0.5
  );

  const rect = CanvasKit.XYWHRect(
    mappedPosition[0],
    mappedPosition[1],
    entity.size[0],
    entity.size[1]
  );

  if (entity.transform) {
    const origin = [0, entity.size[1]];

    canvas.translate(origin[0], origin[1]);

    canvas.scale(entity.transform.scale[0], entity.transform.scale[1]);

    canvas.rotate;

    canvas.translate(-origin[0], -origin[1]);
  }

  canvas.drawRect(rect, paint);

  canvas.restore();
}
