import "./App.css";
import Timeline from "./components/Timeline";
import Canvas from "./components/Canvas";
import Properties, { PropertiesContainer } from "components/Properties";
import MenuBar from "components/MenuBar";
import ToolBar from "components/ToolBar";
import useKeyControls from "hooks/useKeyControls";
import { useFontsStore } from "stores/fonts.store";

export default function App() {
  const fontsStoreDidInit = useFontsStore((store) => store.didInit);

  useKeyControls();

  return (
    <div className="bg-gray-950 h-full w-full flex flex-col overflow-hidden">
      <MenuBar />
      <div className="flex flex-row flex-[1] overflow-hidden">
        <ToolBar />
        {fontsStoreDidInit && (
          <div className="flex flex-col pl-4 gap-4 pr-4 overflow-x-hidden overflow-y-auto">
            <div className="flex w-full gap-4 flex-col lg:flex-row justify-center items-center">
              <Canvas />
              <PropertiesContainer>
                <Properties />
              </PropertiesContainer>
            </div>
            <Timeline />
          </div>
        )}
      </div>
    </div>
  );
}

{
  /*  */
}
