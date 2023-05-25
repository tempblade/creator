import { RenderState } from "primitives/Timeline";
import { z } from "zod";
import { create } from "zustand";

interface RenderStateStore {
  renderState: z.infer<typeof RenderState>;
  playing: boolean;
  setPlaying: (playing: boolean) => void;
  setCurrentFrame: (target: number) => void;
}

const useRenderStateStore = create<RenderStateStore>((set) => ({
  renderState: {
    curr_frame: 20,
  },
  playing: false,
  setPlaying: (playing) => set({ playing }),
  setCurrentFrame: (target) =>
    set((store) => {
      store.renderState = {
        curr_frame: target,
      };

      return { renderState: store.renderState };
    }),
}));

export { useRenderStateStore };
