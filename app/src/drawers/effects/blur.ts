import { Canvas, CanvasKit, Surface } from "canvaskit-wasm";
import { BlurEffectLayer } from "primitives/Effects";
import { z } from "zod";

export default function applyBlur(
  CanvasKit: CanvasKit,
  canvas: Canvas,
  surface: Surface,
  options: z.input<typeof BlurEffectLayer>
) {
  const image = surface.makeImageSnapshot();

  if (image) {
    const blurFilter = CanvasKit.ImageFilter.MakeBlur(
      options.amountX,
      options.amountY,
      CanvasKit.TileMode[options.tileMode],
      null
    );

    const paint = new CanvasKit.Paint();

    paint.setImageFilter(blurFilter);

    canvas.drawImage(image, 0, 0, paint);
  }
}
