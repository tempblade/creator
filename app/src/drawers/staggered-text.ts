import {
  Canvas,
  CanvasKit,
  Font,
  FontMetrics,
  MallocObj,
  TypedArray,
  Typeface,
} from "canvaskit-wasm";
import { StaggeredTextEntity } from "primitives/Entities";
import { z } from "zod";
import { buildPaintStyle } from "./paint";
import { EntityCache } from "./cache";
import { Dependencies } from "services/dependencies.service";

export type StaggeredTextCache = {
  letterMeasures: Array<LetterMeasures>;
  metrics: FontMetrics;
  typeface: Typeface;
  font: Font;
  glyphs: MallocObj;
};

export type StaggeredTextEntityCache = EntityCache<StaggeredTextCache>;

function getUniqueCharacters(str: string): string {
  const uniqueCharacters: string[] = [];

  for (let i = 0; i < str.length; i++) {
    const character = str[i];

    if (!uniqueCharacters.includes(character)) {
      uniqueCharacters.push(character);
    }
  }

  return uniqueCharacters.join("");
}

function measureLetters(
  glyphArr: TypedArray,
  boundsById: Record<number, LetterBounds>,
  maxWidth: number
): Array<LetterMeasures> {
  const measuredLetters: Array<LetterMeasures> = [];

  let currentWidth = 0;
  let currentLine = 0;

  for (let i = 0; i < glyphArr.length; i++) {
    const nextGlyph = boundsById[glyphArr[i]];

    const nextGlyphWidth = nextGlyph.x_advance;

    currentWidth += nextGlyphWidth;

    if (currentWidth > maxWidth) {
      currentLine += 1;
      currentWidth = 0;
    }

    measuredLetters.push({
      bounds: nextGlyph,
      line: currentLine,
      offset: {
        x: currentWidth - nextGlyphWidth,
      },
    });
  }

  return measuredLetters;
}

type LetterBounds = {
  x: {
    max: number;
    min: number;
  };
  y: {
    max: number;
    min: number;
  };
  width: number;
  height: number;
  x_advance: number;
};

type LetterMeasures = {
  offset: {
    x: number;
  };
  line: number;
  bounds: LetterBounds;
};

export function calculateLetters(
  CanvasKit: CanvasKit,
  entity: z.output<typeof StaggeredTextEntity>,
  dependencies: Dependencies
): StaggeredTextCache {
  const fontData = dependencies.fonts.get(
    entity.letter.paint.font_name
  ) as ArrayBuffer;

  const typeface = CanvasKit.Typeface.MakeFreeTypeFaceFromData(
    fontData
  ) as Typeface;

  const font = new CanvasKit.Font(typeface, entity.letter.paint.size);

  const glyphIDs = font.getGlyphIDs(entity.text);

  // font.setLinearMetrics(true);
  font.setSubpixel(true);
  font.setHinting(CanvasKit.FontHinting.None);

  const alphabet = getUniqueCharacters(entity.text);
  const ids = font.getGlyphIDs(alphabet);
  const unknownCharacterGlyphID = ids[0];

  const charsToGlyphIDs: Record<string, any> = {};

  let glyphIdx = 0;
  for (let i = 0; i < alphabet.length; i++) {
    charsToGlyphIDs[alphabet[i]] = ids[glyphIdx];
    if ((alphabet.codePointAt(i) as number) > 65535) {
      i++; // skip the next index because that will be the second half of the code point.
    }
    glyphIdx++;
  }

  const metrics = font.getMetrics();
  const bounds = font.getGlyphBounds(glyphIDs);
  const widths = font.getGlyphWidths(glyphIDs);

  const glyphMetricsByGlyphID: Record<number, LetterBounds> = {};
  for (let i = 0; i < glyphIDs.length; i++) {
    const id = glyphIDs[i];

    const x_min = bounds[i * 4];
    const x_max = bounds[i * 4 + 2];
    const y_min = bounds[i * 4 + 3];
    const y_max = bounds[i * 4 + 1];
    const width = x_max - x_min;
    const height = Math.abs(y_max - y_min);

    glyphMetricsByGlyphID[id] = {
      x: {
        min: x_min,
        max: x_max,
      },
      y: {
        min: y_min,
        max: y_max,
      },
      width,
      height,
      x_advance: widths[i],
    };
  }

  const glyphs = CanvasKit.MallocGlyphIDs(entity.text.length);
  let glyphArr = glyphs.toTypedArray();

  const MAX_WIDTH = 900;

  // Turn the code points into glyphs, accounting for up to 2 ligatures.
  let shapedGlyphIdx = -1;
  for (let i = 0; i < entity.text.length; i++) {
    const char = entity.text[i];
    shapedGlyphIdx++;
    glyphArr[shapedGlyphIdx] = charsToGlyphIDs[char] || unknownCharacterGlyphID;
    if ((entity.text.codePointAt(i) as number) > 65535) {
      i++; // skip the next index because that will be the second half of the code point.
    }
  }
  // Trim down our array of glyphs to only the amount we have after ligatures and code points
  // that are > 16 bits.
  glyphArr = glyphs.subarray(0, shapedGlyphIdx + 1);

  // Break our glyphs into runs based on the maxWidth and the xAdvance.

  const letterMeasures = measureLetters(
    glyphArr,
    glyphMetricsByGlyphID,
    MAX_WIDTH
  );

  return { letterMeasures, metrics, font, typeface, glyphs };
}

export default function drawStaggeredText(
  CanvasKit: CanvasKit,
  canvas: Canvas,
  entity: z.output<typeof StaggeredTextEntity>,
  cache: StaggeredTextCache
) {
  const paint = new CanvasKit.Paint();

  const { letterMeasures: measuredLetters, font, glyphs, metrics } = cache;

  buildPaintStyle(CanvasKit, paint, entity.letter.paint);

  if (glyphs) {
    // Draw all those runs.
    for (let i = 0; i < measuredLetters.length; i++) {
      const measuredLetter = measuredLetters[i];

      const glyph = glyphs.subarray(i, i + 1);

      const blob = CanvasKit.TextBlob.MakeFromGlyphs(
        glyph as unknown as Array<number>,
        font
      );
      if (blob) {
        canvas.save();

        const width = measuredLetters
          .filter((letter) => letter.line === 0)
          .reduce((prev, curr) => curr.bounds.x_advance + prev, 0);

        const lineOffset = (entity.letter.paint.size / 2) * measuredLetter.line;

        const entityOrigin = [
          entity.origin[0] - width / 2,
          entity.origin[1] + lineOffset,
        ];

        const lineCount = measuredLetters
          .map((e) => e.line)
          .sort((a, b) => a - b)[measuredLetters.length - 1];

        if (entity.letter.transform && entity.letter.transform[i]) {
          const letterTransform = entity.letter.transform[i];
          const letterOrigin = [0, 0];

          let origin = letterOrigin.map(
            (val, index) => val + entityOrigin[index]
          );

          // Calculate the spacing

          const spacing =
            measuredLetter.bounds.x_advance - measuredLetter.bounds.width;

          //console.log(spacing);

          // Center the origin

          origin[0] =
            origin[0] +
            measuredLetter.bounds.width / 2 +
            measuredLetter.offset.x +
            letterTransform.translate[0];
          origin[1] =
            origin[1] -
            metrics.descent +
            lineOffset +
            letterTransform.translate[1];

          //console.log(measuredLetter.bounds);

          canvas.translate(origin[0], origin[1]);

          canvas.rotate(
            letterTransform.rotate[2],
            letterTransform.rotate[0],
            letterTransform.rotate[1]
          );

          canvas.scale(letterTransform.scale[0], letterTransform.scale[1]);

          canvas.translate(
            letterTransform.translate[0],
            letterTransform.translate[1]
          );

          canvas.translate(
            -origin[0] + measuredLetter.offset.x,
            -origin[1] + lineOffset
          );

          /*  canvas.translate(
          measuredLetter.offset.x + measuredLetter.bounds.width / 2,
          0
        ); */
        }

        /*     canvas.translate(
        width * -0.5,
        lineCount * (-entity.letter.paint.size / 2)
      ); */

        canvas.drawTextBlob(blob, entityOrigin[0], entityOrigin[1], paint);

        canvas.restore();

        blob.delete();
      }
    }
  }
}
