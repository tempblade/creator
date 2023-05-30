import { useCallback, useEffect } from "react";
import { useEntitiesStore } from "stores/entities.store";
import { useRenderStateStore } from "stores/render-state.store";

export default function useKeyControls() {
  const handleKeyPress = useCallback((e: KeyboardEvent) => {
    if (e.code === "Space") {
      useRenderStateStore.getState().togglePlaying();
    }
    if (e.code === "Backspace") {
      const selectedEntity = useEntitiesStore.getState().selectedEntity;
      if (selectedEntity !== undefined) {
        useEntitiesStore.getState().deleteEntity(selectedEntity);
      }
    }
  }, []);

  useEffect(() => {
    // attach the event listener
    document.addEventListener("keydown", handleKeyPress);

    // remove the event listener
    return () => {
      document.removeEventListener("keydown", handleKeyPress);
    };
  }, [handleKeyPress]);
}
