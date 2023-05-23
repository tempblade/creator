import { FC } from "react";
import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api";
import { useTimelineStore } from "stores/timeline.store";
import InitCanvasKit, { CanvasKit } from "canvaskit-wasm";
import { Surface } from "canvaskit-wasm";
import drawText from "drawers/text";
import drawRect from "drawers/rect";
import { Entities, EntityType } from "primitives/Entities";
import drawEllipse from "drawers/ellipse";
import { useRenderStateStore } from "stores/render-state.store";
import { useEntitiesStore } from "stores/entities.store";
import { AnimatedEntities } from "primitives/AnimatedEntities";
import drawStaggeredText, {
  StaggeredTextCache,
  calculateLetters,
} from "drawers/staggered-text";
import useMap from "hooks/useMap";

type CanvasProps = {};

function typedArrayToBuffer(array: Uint8Array): ArrayBuffer {
  return array.buffer.slice(
    array.byteOffset,
    array.byteLength + array.byteOffset
  );
}

const CanvasComponent: FC<CanvasProps> = () => {
  const canvas = useRef<HTMLCanvasElement>(null);

  const [loading, setLoading] = useState(true);
  const [canvasKit, setCanvasKit] = useState<CanvasKit>();
  const [fontData, setFontData] = useState<ArrayBuffer>();
  const surface = useRef<Surface>();
  const staggeredTextCache = useMap<string, StaggeredTextCache>();
  const isLocked = useRef<boolean>(false);
  const renderState = useRenderStateStore((store) => store.renderState);
  const { fps, size, duration } = useTimelineStore((store) => ({
    fps: store.fps,
    size: store.size,
    duration: store.duration,
  }));
  const { entities, updateEntityById } = useEntitiesStore((store) => ({
    entities: store.entities,
    updateEntityById: store.updateEntityById,
  }));

  useEffect(() => {
    InitCanvasKit({
      locateFile: (file) =>
        "https://unpkg.com/canvaskit-wasm@latest/bin/" + file,
    }).then((CanvasKit) => {
      setCanvasKit(CanvasKit);

      /*   fetch("https://storage.googleapis.com/skia-cdn/misc/Roboto-Regular.ttf")
        .then((response) => response.arrayBuffer())
        .then((arrayBuffer) => {
          setLoading(false);
          setFontData(arrayBuffer);
        }); */

      invoke("get_system_font", { fontName: "Helvetica-Bold" }).then((data) => {
        console.log(data);

        if (Array.isArray(data)) {
          const u8 = new Uint8Array(data as any);
          const buffer = typedArrayToBuffer(u8);
          setFontData(buffer);
          setLoading(false);

          if (canvas.current) {
            const CSurface = CanvasKit.MakeWebGLCanvasSurface(canvas.current);
            if (CSurface) {
              surface.current = CSurface;
            }
          }
        }
      });
    });
  }, []);

  useEffect(() => {
    //console.time("calculation");
    const parsedEntities = AnimatedEntities.parse(entities);

    if (!loading && !isLocked.current) {
      isLocked.current = true;
      invoke("calculate_timeline_entities_at_frame", {
        timeline: {
          entities: parsedEntities,
          render_state: renderState,
          fps,
          size,
          duration,
        },
      }).then((data) => {
        const entitiesResult = Entities.safeParse(data);

        if (canvasKit && canvas.current && surface.current && fontData) {
          surface.current.flush();

          surface.current.requestAnimationFrame((skCanvas) => {
            skCanvas.clear(canvasKit.WHITE);
            if (entitiesResult.success) {
              const entities = entitiesResult.data;

              entities.reverse().forEach((entity) => {
                switch (entity.type) {
                  case EntityType.Enum.Rect:
                    drawRect(canvasKit, skCanvas, entity);
                    break;
                  case EntityType.Enum.Ellipse:
                    drawEllipse(canvasKit, skCanvas, entity);
                    break;
                  case EntityType.Enum.Text:
                    drawText(canvasKit, skCanvas, entity, fontData);
                    break;
                  case EntityType.Enum.StaggeredText:
                    {
                      let cache: StaggeredTextCache;
                      if (!entity.cache.valid) {
                        const _cache = staggeredTextCache[0].get(entity.id);

                        if (_cache !== undefined) {
                          canvasKit.Free(_cache.glyphs);
                        }

                        cache = calculateLetters(canvasKit, entity, fontData);

                        staggeredTextCache[1].set(entity.id, cache);
                        updateEntityById(entity.id, { cache: { valid: true } });
                      } else {
                        const _cache = staggeredTextCache[0].get(entity.id);
                        if (_cache) {
                          cache = _cache;
                        } else {
                          cache = calculateLetters(canvasKit, entity, fontData);
                        }
                      }

                      drawStaggeredText(
                        canvasKit,
                        skCanvas,
                        entity,
                        cache.font,
                        cache.letterMeasures,
                        cache.metrics
                      );
                    }

                    break;
                  default:
                    break;
                }

                isLocked.current = false;
              });
            } else {
              isLocked.current = false;
              console.log(entitiesResult.error);
            }
          });
        }
        //console.timeEnd("draw");
      });
    }
  }, [entities, loading, renderState.curr_frame]);

  return (
    <div>
      <div
        className="flex items-center justify-center"
        style={{ width: "100%", height: "500px" }}
      >
        <canvas
          className="aspect-video h-full"
          height={720}
          width={1280}
          ref={canvas}
        ></canvas>
      </div>
    </div>
  );
};

export default CanvasComponent;
