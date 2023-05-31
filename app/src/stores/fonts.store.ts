import { create } from "zustand";

interface FontsStore {
  fonts: Array<string>;
  didInit: boolean;
  setDidInit: (didInit: boolean) => void;
  setFonts: (fonts: Array<string>) => void;
}

const useFontsStore = create<FontsStore>((set) => ({
  fonts: [],
  didInit: false,
  setDidInit: (didInit) => set({ didInit }),
  setFonts: (fonts) => set({ fonts }),
}));

export { useFontsStore };
