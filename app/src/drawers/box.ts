import { Canvas, CanvasKit } from "canvaskit-wasm";
import { z } from "zod";
import { BoxEntity } from "primitives/Entities";
import { buildPaintStyle } from "./paint";

export default function drawBox(
  CanvasKit: CanvasKit,
  canvas: Canvas,
  entity: z.infer<typeof BoxEntity>
) {
  const paint = new CanvasKit.Paint();

  const debugPaint = new CanvasKit.Paint();
  debugPaint.setColor(CanvasKit.RED);
  buildPaintStyle(CanvasKit, paint, entity.paint);

  let targetPosition = entity.position;

  canvas.drawCircle(targetPosition[0], targetPosition[1], 10, debugPaint);

  targetPosition = targetPosition.map((val, index) => {
    let temp = val - entity.size[index] * 0.5;

    return temp;
  });

  debugPaint.setColor(CanvasKit.BLUE);

  canvas.drawCircle(targetPosition[0], targetPosition[1], 10, debugPaint);

  debugPaint.setColor(CanvasKit.GREEN);

  canvas.drawCircle(targetPosition[0], targetPosition[1], 10, debugPaint);

  console.log(targetPosition[0], targetPosition[1]);

  const rect = CanvasKit.XYWHRect(
    targetPosition[0],
    targetPosition[1],
    entity.size[0],
    entity.size[1]
  );

  canvas.drawRect(rect, paint);

  canvas.drawCircle(targetPosition[0], targetPosition[1], 10, debugPaint);
}
