import { FC } from "react";
import { Reorder } from "framer-motion";
import TimePicker from "./Timepicker";
import { useEntitiesStore } from "stores/entities.store";
import Timestamp from "./Timestamp";
import { PauseIcon, PlayIcon } from "@radix-ui/react-icons";
import { useRenderStateStore } from "stores/render-state.store";
import Track from "./Track";
import * as ScrollArea from "@radix-ui/react-scroll-area";
import ScrollBar from "components/ScrollArea";

export type AnimationEntity = {
  offset: number;
  duration: number;
};

type TimelineProps = {};

const Timeline: FC<TimelineProps> = () => {
  const { entities, setEntities } = useEntitiesStore((store) => ({
    entities: store.entities,
    setEntities: store.setEntities,
  }));

  const { setPlaying } = useRenderStateStore((store) => ({
    setPlaying: store.setPlaying,
  }));

  return (
    <div className="flex flex-col grow shrink h-fit p-4 border transition-colors focus-within:border-gray-400 border-gray-600 rounded-md">
      <div className="flex flex-row">
        <div className="flex flex-row">
          <button onClick={() => setPlaying(true)} className="w-8 h-8">
            <PlayIcon className="text-main" width="100%" height="100%" />
          </button>
          <button onClick={() => setPlaying(false)} className="w-8 h-8">
            <PauseIcon className="text-main" width="100%" height="100%" />
          </button>
        </div>
        <Timestamp />
      </div>
      <ScrollArea.Root>
        <ScrollArea.Viewport className="w-full h-full">
          <div className="gap-1 w-full flex flex-col">
            <div className="z-20 flex flex-row gap-1">
              <div className="flex-shrink-0 min-w-[200px]" />
              <TimePicker />
            </div>
            <Reorder.Group
              className="gap-1 flex flex-col"
              values={entities}
              onReorder={setEntities}
            >
              {entities.map((entity, index) => (
                <Track
                  entity={entity}
                  key={entity.id}
                  name={entity.type}
                  index={index}
                  animationData={entity.animation_data}
                />
              ))}
            </Reorder.Group>
          </div>
        </ScrollArea.Viewport>
        <div className="h-4 sticky bottom-0">
          <ScrollBar orientation="horizontal" />
        </div>
      </ScrollArea.Root>
    </div>
  );
};

export default Timeline;
