import { FC, useMemo } from "react";
import { useEffect, useRef, useState } from "react";
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
    <div
      className="flex items-center justify-center"
      style={{ width: "100%", height: "100%" }}
    >
      <canvas
        style={{ width: "100%", height: "100%" }}
        className="h-full object-contain"
        height={720}
        width={1280}
        ref={canvas}
      ></canvas>
    </div>
  );
};

export default CanvasComponent;
