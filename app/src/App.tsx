import "./App.css";
import Timeline from "./components/Timeline";
import Canvas from "./components/Canvas";
import Properties, { PropertiesContainer } from "components/Properties";
import MenuBar from "components/MenuBar";
import ToolBar from "components/ToolBar";
import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { useFontsStore } from "stores/fonts.store";

export default function App() {
  const { setFonts } = useFontsStore();

  useEffect(() => {
    invoke("get_system_fonts").then((data) => {
      if (data && Array.isArray(data)) {
        setFonts(data);
      }
    });
  }, []);

  return (
    <div className="bg-gray-950 overflow-y-hidden h-full w-full flex flex-col">
      <MenuBar />
      <div className="flex flex-row w-full h-full">
        <ToolBar />
        <div className="flex flex-col ml-4 mr-4 mt-4 w-full h-full overflow-x-hidden">
          <div className="flex gap-4 flex-col lg:flex-row mb-4 justify-center items-center">
            <Canvas />
            <PropertiesContainer>
              <Properties />
            </PropertiesContainer>
          </div>
          <Timeline />
        </div>
      </div>
    </div>
  );
}
