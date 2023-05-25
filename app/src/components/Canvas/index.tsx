import { FC, useMemo } from "react";
import { useEffect, useRef, useState } from "react";
import { useTimelineStore } from "stores/timeline.store";
import { useRenderStateStore } from "stores/render-state.store";
import { useEntitiesStore } from "stores/entities.store";
import { Drawer } from "drawers/draw";
import { PlaybackService } from "services/playback.service";

type CanvasProps = {};

const CanvasComponent: FC<CanvasProps> = () => {
  const canvas = useRef<HTMLCanvasElement>(null);
  const [didInit, setDidInit] = useState(false);

  const playbackService = useMemo(() => new PlaybackService(), []);

  useEffect(() => {
    if (canvas.current && !didInit) {
      playbackService
        .init(canvas.current)
        .then(() => {
          setDidInit(true);
        })
        .catch((e) => console.error(e));
    }
  }, []);

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
