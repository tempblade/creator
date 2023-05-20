import { Canvas, CanvasKit } from "canvaskit-wasm";
import { TextEntity } from "primitives/Entities";
import { convertToFloat } from "@tempblade/common";
import { z } from "zod";

export default function drawText(
  CanvasKit: CanvasKit,
  canvas: Canvas,
  entity: z.infer<typeof TextEntity>,
  fontData: ArrayBuffer
) {
  const fontMgr = CanvasKit.FontMgr.FromData(fontData);

  if (!fontMgr) {
    console.error("No FontMgr");
    return;
  }

  const paint = new CanvasKit.Paint();

  const color = convertToFloat(entity.paint.style.color.value);

  paint.setColor(color);

  const pStyle = new CanvasKit.ParagraphStyle({
    textStyle: {
      color: color,
      fontFamilies: ["Roboto"],
      fontSize: entity.paint.size,
    },
    textDirection: CanvasKit.TextDirection.LTR,
    textAlign: CanvasKit.TextAlign[entity.paint.align],
  });

  const builder = CanvasKit.ParagraphBuilder.Make(pStyle, fontMgr);
  builder.addText(entity.text);
  const p = builder.build();
  p.layout(900);
  const height = p.getHeight() / 2;
  const width = p.getMaxWidth() / 2;
  canvas.drawParagraph(p, entity.origin[0] - width, entity.origin[1] - height);
}
