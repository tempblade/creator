import { defineConfig } from "unocss";
import presetMini from "@unocss/preset-mini";

export default defineConfig({
  presets: [presetMini()],
  theme: {
    breakpoints: {
      s: "576px",
      m: "768px",
      l: "992px",
      xl: "1200px",
      "2xl": "1400px",
      "3xl": "1600px",
    },
  },
});
