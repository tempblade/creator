import { create } from "zustand";

interface KeyframeStore {
  selectedKeyframe: string | undefined;
  selectKeyframe: (id: string) => void;
  deselectKeyframe: () => void;
}

const useKeyframeStore = create<KeyframeStore>((set) => ({
  selectKeyframe: (id) => set({ selectedKeyframe: id }),
  deselectKeyframe: () => set({ selectedKeyframe: undefined }),
  selectedKeyframe: undefined,
}));

export { useKeyframeStore };
