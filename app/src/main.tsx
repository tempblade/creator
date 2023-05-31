import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";
import { useFontsStore } from "stores/fonts.store";
import { invoke } from "@tauri-apps/api";

invoke("get_system_families").then((data) => {
  if (data && Array.isArray(data)) {
    const fontsStore = useFontsStore.getState();
    fontsStore.setFonts(data);
    fontsStore.setDidInit(true);
  }
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
