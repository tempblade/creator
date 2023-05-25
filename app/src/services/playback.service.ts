import { Drawer } from "drawers/draw";
import { AnimatedEntities } from "primitives/AnimatedEntities";
import { useEntitiesStore } from "stores/entities.store";
import { useRenderStateStore } from "stores/render-state.store";
import { useTimelineStore } from "stores/timeline.store";

export class PlaybackService {
  drawer: Drawer;
  lastDrawTime: number | undefined;
  raf: number | undefined;
  playing: boolean;

  constructor() {
    this.drawer = new Drawer();
    this.lastDrawTime = undefined;
    this.raf = undefined;
    this.playing = false;
  }

  async init(canvas: HTMLCanvasElement) {
    await this.drawer.init(canvas);

    useRenderStateStore.subscribe((state) => {
      if (!this.playing && state.playing) {
        this.playing = true;
        this.play();
      }

      if (this.playing && !state.playing) {
        this.playing = false;
        this.stop();
      }

      if (!this.playing && !state.playing) {
        this.seek();
      }
    });

    this.seek();
  }

  play() {
    this.drawer.dependenciesService.prepareForAnimatedEntities(
      this.animatedEntities
    );

    const currentTime = window.performance.now();
    this.lastDrawTime = currentTime;
    this.playLoop(currentTime);
  }

  stop() {
    if (this.raf !== undefined) {
      cancelAnimationFrame(this.raf);
    }
  }

  seek() {
    this.drawer.update(this.animatedEntities, true);
  }

  get animatedEntities() {
    return AnimatedEntities.parse(useEntitiesStore.getState().entities);
  }

  get timelineStore() {
    return useTimelineStore.getState();
  }

  get fpsInterval() {
    return 1000 / this.timelineStore.fps;
  }

  get currFrame() {
    return useRenderStateStore.getState().renderState.curr_frame;
  }

  get totalFrameCount() {
    return this.timelineStore.fps * this.timelineStore.duration;
  }

  playLoop(currentTime: number) {
    this.raf = requestAnimationFrame(this.playLoop.bind(this));

    if (this.lastDrawTime !== undefined) {
      const elapsed = currentTime - this.lastDrawTime;

      if (elapsed > this.fpsInterval) {
        this.lastDrawTime = currentTime - (elapsed % this.fpsInterval);

        const nextFrame =
          this.currFrame + 1 < this.totalFrameCount ? this.currFrame + 1 : 0;

        useRenderStateStore.getState().setCurrentFrame(nextFrame);

        this.drawer.update(this.animatedEntities, false);
      }
    }
  }
}
