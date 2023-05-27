import { create } from "zustand";

interface FontsStore {
  fonts: Array<string>;
  setFonts: (fonts: Array<string>) => void;
}

const useFontsStore = create<FontsStore>((set) => ({
  fonts: [],
  setFonts: (fonts) => set({ fonts }),
}));

export { useFontsStore };
