import { convertToFloat } from "@tempblade/common";
import { Canvas, CanvasKit } from "canvaskit-wasm";
import { StaggeredText } from "primitives/Entities";
import { z } from "zod";

export default function drawStaggeredText(
  CanvasKit: CanvasKit,
  canvs: Canvas,
  entity: z.output<typeof StaggeredText>,
  fontData: ArrayBuffer
) {
  const paint = new CanvasKit.Paint();

  const color = convertToFloat(entity.letter.paint.style.color.value);

  paint.setColor(color);

  const typeface = CanvasKit.Typeface.MakeFreeTypeFaceFromData(fontData);

  const font = new CanvasKit.Font(typeface, entity.letter.paint.size);

  const glyphIDs = font.getGlyphIDs(entity.text);

  font.setLinearMetrics(true);
  font.setSubpixel(true);
  font.setHinting(CanvasKit.FontHinting.Slight);

  const bounds = font.getGlyphBounds(glyphIDs, paint);
  const widths = font.getGlyphWidths(glyphIDs, paint);

  console.log(bounds);
  console.log(widths);
}
