import { FC, useMemo } from "react";
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
import { Drawer } from "drawers/draw";

type CanvasProps = {};

function typedArrayToBuffer(array: Uint8Array): ArrayBuffer {
  return array.buffer.slice(
    array.byteOffset,
    array.byteLength + array.byteOffset
  );
}

const CanvasComponent: FC<CanvasProps> = () => {
  const canvas = useRef<HTMLCanvasElement>(null);
  const [didInit, setDidInit] = useState(false);
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

  const drawer = useMemo(() => new Drawer(), []);

  useEffect(() => {
    if (canvas.current && !didInit) {
      drawer
        .init(canvas.current)
        .then(() => {
          setDidInit(true);
        })
        .catch((e) => console.error(e));
    }
  }, []);

  useEffect(() => {
    if (didInit) {
      drawer.update(entities);
    }
  }, [entities, renderState.curr_frame, didInit]);

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
