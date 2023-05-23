import { FC } from "react";
import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api";
import { useTimelineStore } from "stores/timeline.store";
import InitCanvasKit, { CanvasKit } from "canvaskit-wasm";
import { Surface } from "canvaskit-wasm";
import drawText from "drawers/text";
import drawBox from "drawers/box";
import { Entities, EntityType } from "primitives/Entities";
import drawEllipse from "drawers/ellipse";
import { useRenderStateStore } from "stores/render-state.store";
import { useEntitiesStore } from "stores/entities.store";
import { AnimatedEntities } from "primitives/AnimatedEntities";

type CanvasProps = {};

const CanvasComponent: FC<CanvasProps> = () => {
  const canvas = useRef<HTMLCanvasElement>(null);

  const [loading, setLoading] = useState(true);
  const [canvasKit, setCanvasKit] = useState<CanvasKit>();
  const [fontData, setFontData] = useState<ArrayBuffer>();
  const surface = useRef<Surface>();

  const renderState = useRenderStateStore((store) => store.renderState);
  const { fps, size, duration } = useTimelineStore((store) => ({
    fps: store.fps,
    size: store.size,
    duration: store.duration,
  }));
  const { entities } = useEntitiesStore((store) => ({
    entities: store.entities,
  }));

  useEffect(() => {
    InitCanvasKit({
      locateFile: (file) =>
        "https://unpkg.com/canvaskit-wasm@latest/bin/" + file,
    }).then((CanvasKit) => {
      setLoading(false);
      setCanvasKit(CanvasKit);

      fetch("https://storage.googleapis.com/skia-cdn/misc/Roboto-Regular.ttf")
        .then((response) => response.arrayBuffer())
        .then((arrayBuffer) => {
          setFontData(arrayBuffer);
        });

      if (canvas.current) {
        const CSurface = CanvasKit.MakeWebGLCanvasSurface(canvas.current);
        if (CSurface) {
          surface.current = CSurface;
        }
      }
    });
  }, []);

  useEffect(() => {
    console.time("calculation");
    const parsedEntities = AnimatedEntities.parse(entities);

    invoke("calculate_timeline_entities_at_frame", {
      timeline: {
        entities: parsedEntities,
        render_state: renderState,
        fps,
        size,
        duration,
      },
    }).then((data) => {
      console.timeEnd("calculation");
      // console.log(data);

      const entitiesResult = Entities.safeParse(data);
      console.time("draw");

      if (canvasKit && canvas.current && surface.current && fontData) {
        surface.current.flush();
        surface.current.requestAnimationFrame((skCanvas) => {
          skCanvas.clear(canvasKit.WHITE);
          if (entitiesResult.success) {
            const entities = entitiesResult.data;

            entities.reverse().forEach((entity) => {
              switch (entity.type) {
                case EntityType.Enum.Box:
                  drawBox(canvasKit, skCanvas, entity);
                  break;
                case EntityType.Enum.Ellipse:
                  drawEllipse(canvasKit, skCanvas, entity);
                  break;
                case EntityType.Enum.Text:
                  drawText(canvasKit, skCanvas, entity, fontData);
                  break;
                default:
                  break;
              }
            });
          } else {
            console.log(entitiesResult.error);
          }
        });
      }
      console.timeEnd("draw");
    });
  });

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
