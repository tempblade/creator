import { invoke } from "@tauri-apps/api";
import InitCanvasKit, { CanvasKit } from "canvaskit-wasm";
import { AnimatedEntities } from "primitives/AnimatedEntities";
import { Entities } from "primitives/Entities";
import { useRenderStateStore } from "stores/render-state.store";
import { useTimelineStore } from "stores/timeline.store";
import { z } from "zod";
import { StaggeredTextCache } from "./staggered-text";

/**
 *
 * TODO Add dependency logic for e.g. dynamically loading fonts, images etc.
 */

export class Drawer {
  private readonly didLoad: boolean;
  private entities: z.output<typeof Entities> | undefined;
  private ckDidLoad: boolean;
  private dependenciesDidLoad: boolean;
  private CanvasKit: CanvasKit | undefined;
  cache: { staggeredText: Map<string, StaggeredTextCache> };

  constructor() {
    this.entities = undefined;
    this.CanvasKit = undefined;
    this.ckDidLoad = false;
    this.dependenciesDidLoad = false;
    this.cache = {
      staggeredText: new Map(),
    };
    this.didLoad = this.ckDidLoad && this.dependenciesDidLoad;
  }

  init() {
    InitCanvasKit({
      locateFile: (file) =>
        "https://unpkg.com/canvaskit-wasm@latest/bin/" + file,
    }).then((CanvasKit) => {
      this.CanvasKit = CanvasKit;
    });
  }

  /**
   * Updates the entities based on the input
   */
  update(animatedEntities: z.input<typeof AnimatedEntities>) {
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
        const parsedEntities = Entities.safeParse(data);
        if (parsedEntities.success) {
          this.entities = parsedEntities.data;
        } else {
          console.error(parsedEntities.error);
        }
      });
    }
  }

  draw() {
    if (this.didLoad) {
    }
  }
}
