import { useEffect, useRef, useState } from "react";
import "./App.css";
import { invoke } from "@tauri-apps/api";
import { useTimelineStore } from "./stores/timeline.store";
import Timeline, { AnimationEntity } from "./components/Timeline";
import InitCanvasKit from "canvaskit-wasm";
import CanvasKitWasm from "canvaskit-wasm/bin/canvaskit.wasm?url";

const WIDTH = 1280 / 2;
const HEIGHT = 720 / 2;

const ENTITIES: Array<AnimationEntity> = [
  {
    offset: 0.2,
    duration: 1.0,
  },
];

function App() {
  const canvas = useRef<HTMLCanvasElement>(null);

  const [title, setTitle] = useState("Kleine");
  const [subTitle, setSubTitle] = useState("Dumpfkopf");
  const [loading, setLoading] = useState(true);

  const { currentFrame } = useTimelineStore();

  useEffect(() => {
    console.time("render");
    invoke("render_timeline_frame_cpu", {
      currFrame: currentFrame,
      title,
      subTitle,
      width: WIDTH,
      height: HEIGHT,
    }).then((data) => {
      console.timeEnd("render");
      if (canvas.current) {
        const canvasElement = canvas.current;

        canvasElement.width = WIDTH;
        canvasElement.height = HEIGHT;
        //      console.time("draw");
        const img = document.createElement("img");

        const ctx = canvasElement.getContext("2d");

        const arr = new Uint8ClampedArray(data as any);

        const image = new Blob([arr], { type: "image/webp" });
        img.src = URL.createObjectURL(image);

        if (ctx) {
          //  ctx.fillStyle = "red";
          // ctx.fillRect(0, 0, 1920, 1080);
        }

        img.onload = () => {
          if (ctx) {
            ctx.drawImage(img, 0, 0);
            //       console.timeEnd("draw");
          }
        };
      }
    });
  }, [currentFrame, title, subTitle]);

  if (loading) return;

  return (
    <div className="container">
      <div style={{ width: "600px" }}>
        <canvas style={{ width: "100%", height: "100%" }} ref={canvas}></canvas>
      </div>
      <input value={title} onChange={(e) => setTitle(e.target.value)} />
      <input value={subTitle} onChange={(e) => setSubTitle(e.target.value)} />
      <Timeline entities={ENTITIES} />
    </div>
  );
}
