import { Canvas, CanvasKit, Font, FontMgr, Typeface } from "canvaskit-wasm";
import { TextEntity } from "primitives/Entities";
import { convertToFloat } from "@tempblade/common";
import { z } from "zod";
import { EntityCache } from "./cache";
import { Dependencies } from "services/dependencies.service";
import { buildPaintStyle } from "./paint";

export type TextCache = {
  fontManager: FontMgr;
};

export type TextEntityCache = EntityCache<TextCache>;

export function buildTextCache(
  CanvasKit: CanvasKit,
  entity: z.output<typeof TextEntity>,
  dependencies: Dependencies
): TextCache {
  const fontData = dependencies.fonts.get(entity.paint.fontName) as ArrayBuffer;

  const fontManager = CanvasKit.FontMgr.FromData(fontData) as FontMgr;

  return {
    fontManager,
  };
}

export default function drawText(
  CanvasKit: CanvasKit,
  canvas: Canvas,
  entity: z.output<typeof TextEntity>,
  cache: TextCache
) {
  canvas.save();

  const paint = new CanvasKit.Paint();

  const color = convertToFloat(entity.paint.style.color.value);

  buildPaintStyle(CanvasKit, paint, entity.paint);

  const pStyle = new CanvasKit.ParagraphStyle({
    textStyle: {
      color: color,
      fontFamilies: [entity.paint.fontName],
      fontSize: entity.paint.size,
    },
    textDirection: CanvasKit.TextDirection.LTR,
    textAlign: CanvasKit.TextAlign[entity.paint.align],
  });

  const builder = CanvasKit.ParagraphBuilder.Make(pStyle, cache.fontManager);
  builder.addText(entity.text);
  const p = builder.build();
  p.layout(900);
  const height = p.getHeight() / 2;
  const width = p.getMaxWidth() / 2;

  canvas.drawParagraph(p, entity.origin[0] - width, entity.origin[1] - height);

  canvas.restore();

  builder.delete();
}
