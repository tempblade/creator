import { RenderState } from "primitives/Timeline";
import { z } from "zod";
import { create } from "zustand";

interface RenderStateStore {
  renderState: z.infer<typeof RenderState>;
  setCurrentFrame: (target: number) => void;
}

const useRenderStateStore = create<RenderStateStore>((set) => ({
  renderState: {
    curr_frame: 20,
  },
  setCurrentFrame: (target) =>
    set((store) => {
      store.renderState = {
        curr_frame: target,
      };

      return { renderState: store.renderState };
    }),
}));

export { useRenderStateStore };
