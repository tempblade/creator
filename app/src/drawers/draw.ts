import { invoke } from "@tauri-apps/api";
import InitCanvasKit, { Canvas, CanvasKit, Surface } from "canvaskit-wasm";
import { AnimatedEntities } from "primitives/AnimatedEntities";
import {
  Entities,
  EntityType,
  StaggeredTextEntity,
  TextEntity,
} from "primitives/Entities";
import { useRenderStateStore } from "stores/render-state.store";
import { useTimelineStore } from "stores/timeline.store";
import { z } from "zod";
import drawStaggeredText, {
  StaggeredTextCache,
  StaggeredTextEntityCache,
  calculateLetters,
} from "./staggered-text";
import drawText, { TextCache, TextEntityCache, buildTextCache } from "./text";
import drawEllipse from "./ellipse";
import drawRect from "./rect";
import { useEntitiesStore } from "stores/entities.store";
import { handleEntityCache } from "./cache";
import { DependenciesService } from "services/dependencies.service";
import { RenderState } from "primitives/Timeline";

/**
 *
 * TODO Add more sophisticated dependency logic for e.g. dynamically loading fonts, images etc.
 */

export class Drawer {
  private didLoad: boolean;
  private entities: z.output<typeof Entities> | undefined;
  private ckDidLoad: boolean;
  drawCount: number;
  private CanvasKit: CanvasKit | undefined;
  cache: {
    staggeredText: Map<string, StaggeredTextCache>;
    text: Map<string, TextCache>;
  };
  surface: Surface | undefined;
  fontData: ArrayBuffer | undefined;
  raf: number | undefined;
  isLocked: boolean;
  dependenciesService: DependenciesService;

  constructor() {
    this.entities = undefined;
    this.CanvasKit = undefined;
    this.ckDidLoad = false;
    this.drawCount = 0;
    this.surface = undefined;
    this.fontData = undefined;
    this.cache = {
      staggeredText: new Map(),
      text: new Map(),
    };
    this.dependenciesService = new DependenciesService();
    this.isLocked = false;
    this.raf = undefined;
    this.didLoad = this.ckDidLoad;
  }

  async init(canvas: HTMLCanvasElement) {
    await this.loadCanvasKit(canvas);

    this.didLoad = this.ckDidLoad;
  }

  async loadCanvasKit(canvas: HTMLCanvasElement) {
    await InitCanvasKit({
      locateFile: (file) => file,
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

  async calculateAnimatedEntities(
    animatedEntities: z.input<typeof AnimatedEntities>,
    renderState: z.output<typeof RenderState>
  ) {
    const { fps, size, duration } = useTimelineStore.getState();

    const parsedAnimatedEntities = AnimatedEntities.parse(animatedEntities);

    const data = await invoke("calculate_timeline_at_curr_frame", {
      timeline: {
        entities: parsedAnimatedEntities,
        render_state: renderState,
        fps,
        size,
        duration,
      },
    });

    const parsedEntities = Entities.parse(data);

    return parsedEntities;
  }

  get isCached(): boolean {
    if (this.entities) {
      return this.entities.reduce(
        (prev, curr) => prev && curr.cache.valid,
        true
      );
    } else {
      return false;
    }
  }

  /**
   * Updates the entities based on the input
   */
  update(
    animatedEntities: z.input<typeof AnimatedEntities>,
    prepareDependencies: boolean
  ) {
    //  console.time("calculate");

    if (this.didLoad) {
      const renderState = useRenderStateStore.getState().renderState;

      this.calculateAnimatedEntities(animatedEntities, renderState).then(
        (entities) => {
          this.entities = entities;

          if (prepareDependencies) {
            this.dependenciesService
              .prepareForEntities(this.entities)
              .then(() => {
                this.requestRedraw(!this.isCached);
              });
          } else {
            this.requestRedraw(!this.isCached);
          }
        }
      );
    } else {
      // console.timeEnd("calculate");
    }
  }

  requestRedraw(rebuild: boolean) {
    if (this.didLoad && this.surface && !this.isLocked) {
      if (rebuild && this.raf !== undefined) {
        cancelAnimationFrame(this.raf);
        // this.surface.flush();
        this.raf = this.surface.requestAnimationFrame((canvas) =>
          this.draw(canvas)
        );
      } else {
        // this.surface.flush();
        this.raf = this.surface.requestAnimationFrame((canvas) =>
          this.draw(canvas)
        );
      }
    }
  }

  draw(canvas: Canvas) {
    if (this.CanvasKit && this.entities && !this.isLocked) {
      this.isLocked = true;
      //console.time("draw");
      const CanvasKit = this.CanvasKit;

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
            {
              const cache = handleEntityCache<
                z.output<typeof TextEntity>,
                TextCache,
                TextEntityCache
              >(entity, {
                build: () => {
                  const cache = buildTextCache(
                    CanvasKit,
                    entity,
                    this.dependenciesService.dependencies
                  );

                  useEntitiesStore
                    .getState()
                    .updateEntityById(entity.id, { cache: { valid: true } });

                  return cache;
                },
                get: () => this.cache.text.get(entity.id),
                set: (id, cache) => this.cache.text.set(id, cache),
                cleanup: (cache) => {
                  cache.fontManager.delete();
                },
              });

              drawText(CanvasKit, canvas, entity, cache);
            }

            break;
          case EntityType.Enum.StaggeredText:
            {
              const cache = handleEntityCache<
                z.output<typeof StaggeredTextEntity>,
                StaggeredTextCache,
                StaggeredTextEntityCache
              >(entity, {
                build: () => {
                  const cache = calculateLetters(
                    CanvasKit,
                    entity,
                    this.dependenciesService.dependencies
                  );
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
      //console.timeEnd("draw");
    }
  }
}
