import { invoke } from "@tauri-apps/api";
import InitCanvasKit, { Canvas, CanvasKit, Surface } from "canvaskit-wasm";
import { AnimatedEntities } from "primitives/AnimatedEntities";
import { Entities, EntityType, StaggeredText } from "primitives/Entities";
import { useRenderStateStore } from "stores/render-state.store";
import { useTimelineStore } from "stores/timeline.store";
import { z } from "zod";
import drawStaggeredText, {
  StaggeredTextCache,
  StaggeredTextEntityCache,
  calculateLetters,
} from "./staggered-text";
import drawText from "./text";
import drawEllipse from "./ellipse";
import drawRect from "./rect";
import { useEntitiesStore } from "stores/entities.store";
import { handleEntityCache } from "./cache";

function typedArrayToBuffer(array: Uint8Array): ArrayBuffer {
  return array.buffer.slice(
    array.byteOffset,
    array.byteLength + array.byteOffset
  );
}

/**
 *
 * TODO Add more sophisticated dependency logic for e.g. dynamically loading fonts, images etc.
 */

export class Drawer {
  private didLoad: boolean;
  private entities: z.output<typeof Entities> | undefined;
  private ckDidLoad: boolean;
  private dependenciesDidLoad: boolean;
  drawCount: number;
  private CanvasKit: CanvasKit | undefined;
  cache: { staggeredText: Map<string, StaggeredTextCache> };
  surface: Surface | undefined;
  fontData: ArrayBuffer | undefined;
  raf: number | undefined;
  isLocked: boolean;

  constructor() {
    this.entities = undefined;
    this.CanvasKit = undefined;
    this.ckDidLoad = false;
    this.dependenciesDidLoad = false;
    this.drawCount = 0;
    this.surface = undefined;
    this.fontData = undefined;
    this.cache = {
      staggeredText: new Map(),
    };
    this.isLocked = false;
    this.raf = undefined;
    this.didLoad = this.ckDidLoad && this.dependenciesDidLoad;
  }

  async init(canvas: HTMLCanvasElement) {
    await this.loadCanvasKit(canvas);
    await this.loadDependencies(false);

    this.didLoad = this.ckDidLoad && this.dependenciesDidLoad;
  }

  async loadCanvasKit(canvas: HTMLCanvasElement) {
    await InitCanvasKit({
      locateFile: (file) =>
        "https://unpkg.com/canvaskit-wasm@latest/bin/" + file,
    }).then((CanvasKit) => {
      if (canvas) {
        const CSurface = CanvasKit.MakeWebGLCanvasSurface(canvas);
        if (CSurface) {
          this.CanvasKit = CanvasKit;
          this.surface = CSurface;
          this.ckDidLoad = true;
        }
      }
    });
  }

  async loadDependencies(remote: boolean) {
    if (remote) {
      await fetch(
        "https://storage.googleapis.com/skia-cdn/misc/Roboto-Regular.ttf"
      )
        .then((response) => response.arrayBuffer())
        .then((arrayBuffer) => {
          this.fontData = arrayBuffer;
          this.dependenciesDidLoad = true;
        });
    } else {
      await invoke("get_system_font", { fontName: "Helvetica-Bold" }).then(
        (data) => {
          if (Array.isArray(data)) {
            const u8 = new Uint8Array(data as any);
            const buffer = typedArrayToBuffer(u8);
            this.fontData = buffer;
            this.dependenciesDidLoad = true;
          }
        }
      );
    }
  }

  /**
   * Updates the entities based on the input
   */
  update(animatedEntities: z.input<typeof AnimatedEntities>) {
    console.time("calculate");

    const parsedAnimatedEntities = AnimatedEntities.parse(animatedEntities);

    if (this.didLoad) {
      const render_state = useRenderStateStore.getState().renderState;
      const { fps, size, duration } = useTimelineStore.getState();

      invoke("calculate_timeline_entities_at_frame", {
        timeline: {
          entities: parsedAnimatedEntities,
          render_state,
          fps,
          size,
          duration,
        },
      }).then((data) => {
        console.timeEnd("calculate");
        const parsedEntities = Entities.safeParse(data);
        if (parsedEntities.success) {
          this.entities = parsedEntities.data;

          const isCached = this.entities.reduce(
            (prev, curr) => prev && curr.cache.valid,
            true
          );

          this.requestRedraw(!isCached);
        } else {
          console.error(parsedEntities.error);
        }
      });
    }
  }

  requestRedraw(rebuild: boolean) {
    if (this.didLoad && this.surface) {
      if (rebuild && this.raf !== undefined) {
        cancelAnimationFrame(this.raf);
        this.surface.flush();
        this.raf = this.surface.requestAnimationFrame((canvas) =>
          this.draw(canvas)
        );
      } else {
        this.surface.flush();
        this.raf = this.surface.requestAnimationFrame((canvas) =>
          this.draw(canvas)
        );
      }
    }
  }

  draw(canvas: Canvas) {
    if (this.CanvasKit && this.entities && this.fontData && !this.isLocked) {
      this.isLocked = true;
      console.time("draw");
      const CanvasKit = this.CanvasKit;
      const fontData = this.fontData;

      canvas.clear(CanvasKit.WHITE);

      this.drawCount++;

      [...this.entities].reverse().forEach((entity) => {
        switch (entity.type) {
          case EntityType.Enum.Rect:
            drawRect(CanvasKit, canvas, entity);
            break;
          case EntityType.Enum.Ellipse:
            drawEllipse(CanvasKit, canvas, entity);
            break;
          case EntityType.Enum.Text:
            drawText(CanvasKit, canvas, entity, fontData);
            break;
          case EntityType.Enum.StaggeredText:
            {
              const cache = handleEntityCache<
                z.output<typeof StaggeredText>,
                StaggeredTextCache,
                StaggeredTextEntityCache
              >(entity, {
                build: () => {
                  const cache = calculateLetters(CanvasKit, entity, fontData);
                  useEntitiesStore
                    .getState()
                    .updateEntityById(entity.id, { cache: { valid: true } });

                  return cache;
                },
                get: () => this.cache.staggeredText.get(entity.id),
                set: (id, cache) => this.cache.staggeredText.set(id, cache),
                cleanup: (cache) => {
                  cache.font.delete();
                  cache.typeface.delete();
                  CanvasKit.Free(cache.glyphs);
                },
              });

              drawStaggeredText(CanvasKit, canvas, entity, cache);
            }

            break;
          default:
            break;
        }
      });
      this.isLocked = false;
      console.timeEnd("draw");
    }
  }
}
