import { FC } from "react";
import { z } from "zod";
import { AnimatedEntity, AnimationData } from "primitives/AnimatedEntities";
import { Reorder, motion, useDragControls } from "framer-motion";
import TimePicker from "./TimePicker";
import { shallow } from "zustand/shallow";
import { useEntitiesStore } from "stores/entities.store";
import { ease } from "@unom/style";
import Timestamp from "./Timestamp";
import { Keyframe } from "primitives/Keyframe";
import { flattenedKeyframesByEntity } from "utils";
import { PauseIcon, PlayIcon } from "@radix-ui/react-icons";
import { useRenderStateStore } from "stores/render-state.store";

export type AnimationEntity = {
  offset: number;
  duration: number;
};

type TimelineProps = {};

export const TIMELINE_SCALE = 50;

type TrackProps = {
  animationData: z.input<typeof AnimationData>;
  name: string;
  index: number;
  entity: z.input<typeof AnimatedEntity>;
  keyframes: Array<z.input<typeof Keyframe>>;
};

const KeyframeIndicator: FC<{
  keyframe: z.input<typeof Keyframe>;
  animationData: z.input<typeof AnimationData>;
}> = ({ keyframe, animationData }) => {
  return (
    <motion.div
      animate={{
        x: (animationData.offset + keyframe.offset) * TIMELINE_SCALE + 4,
      }}
      transition={ease.quint(0.4).out}
      style={{
        clipPath: "polygon(50% 0%, 100% 50%, 50% 100%, 0% 50%)",
      }}
      className="bg-indigo-300 absolute w-2 h-2 z-30 top-[39%] select-none pointer-events-none"
    />
  );
};

const Track: FC<TrackProps> = ({
  keyframes,
  animationData,
  index,
  name,
  entity,
}) => {
  const controls = useDragControls();

  const { updateEntity, selectEntity, selectedEntity, deselectEntity } =
    useEntitiesStore(
      (store) => ({
        updateEntity: store.updateEntity,
        selectedEntity: store.selectedEntity,
        selectEntity: store.selectEntity,
        deselectEntity: store.deselectEntity,
      }),
      shallow
    );

  return (
    <Reorder.Item
      value={entity}
      dragListener={false}
      dragControls={controls}
      className="h-8 w-full flex flex-row gap-1 select-none"
    >
      <div
        onMouseDown={(e) => e.preventDefault()}
        onPointerDown={(e) => controls.start(e)}
        className={`h-full transition-all rounded-sm flex-shrink-0 w-96 p-1 px-2 flex flex-row ${
          selectedEntity === index ? "bg-gray-800" : "bg-gray-900"
        }`}
      >
        <h3
          onClick={() =>
            selectedEntity !== undefined && selectedEntity === index
              ? deselectEntity()
              : selectEntity(index)
          }
          className="text-white-800 select-none pointer-events-none"
        >
          {name}
        </h3>
      </div>

      <div
        style={{ width: "1000px" }}
        className="flex h-full flex-row relative bg-gray-900 select-none"
      >
        {keyframes.map((keyframe, index) => (
          <KeyframeIndicator
            animationData={animationData}
            keyframe={keyframe}
            key={index}
          />
        ))}
        <motion.div
          drag="x"
          animate={{
            x: animationData.offset * TIMELINE_SCALE,
          }}
          whileHover={{
            scale: 1.1,
          }}
          whileTap={{
            scale: 0.9,
          }}
          onMouseDown={(e) => e.preventDefault()}
          transition={ease.circ(0.6).out}
          dragElastic={false}
          dragConstraints={{ left: 0, right: 900 }}
          onDragEnd={(e, info) => {
            let offset = info.offset.x;

            offset *= 0.01;

            const animationOffset =
              animationData.offset + offset < 0
                ? 0
                : animationData.offset + offset;

            const duration = animationData.duration - offset;

            updateEntity(index, {
              animation_data: {
                ...animationData,
                offset: animationOffset < 0 ? 0 : animationOffset,
                duration: duration < 0 ? 0 : duration,
              },
            });
          }}
          className="z-10 w-4 bg-slate-500 h-full absolute rounded-md select-none cursor-w-resize"
        />
        <motion.div
          onMouseDown={(e) => e.preventDefault()}
          drag="x"
          animate={{
            x:
              (animationData.duration + animationData.offset) * TIMELINE_SCALE -
              16,
          }}
          whileHover={{
            scale: 1.1,
          }}
          whileTap={{
            scale: 0.9,
          }}
          transition={ease.circ(0.6).out}
          dragConstraints={{ left: 0, right: 900 }}
          onDragEnd={(e, info) => {
            let offset = info.offset.x;

            offset *= 0.01;

            const duration = animationData.duration + offset;

            updateEntity(index, {
              animation_data: {
                ...animationData,
                duration: duration < 0 ? 0 : duration,
              },
            });
          }}
          className="z-10 w-4 bg-slate-500 h-full absolute rounded-md select-none cursor-e-resize"
        />
        <motion.div
          drag="x"
          animate={{
            width: animationData.duration * TIMELINE_SCALE,
            x: animationData.offset * TIMELINE_SCALE,
          }}
          whileHover={{ scaleY: 1.1 }}
          whileTap={{ scaleY: 0.9 }}
          dragConstraints={{
            left: 0,
            right: 900,
          }}
          onMouseDown={(e) => e.preventDefault()}
          transition={ease.circ(0.8).out}
          onDragEnd={(_e, info) => {
            let offset = info.offset.x;
            offset *= 0.01;
            offset += animationData.offset;
            updateEntity(index, {
              animation_data: {
                ...animationData,
                offset: offset < 0 ? 0 : offset,
              },
            });
          }}
          className="z-5 h-full absolute rounded-md transition-colors bg-gray-700 hover:bg-gray-600 select-none cursor-grab"
        ></motion.div>
      </div>
    </Reorder.Item>
  );
};

const Timeline: FC<TimelineProps> = () => {
  const { entities, setEntities } = useEntitiesStore((store) => ({
    entities: store.entities,
    setEntities: store.setEntities,
  }));

  const { setPlaying } = useRenderStateStore((store) => ({
    setPlaying: store.setPlaying,
  }));

  return (
    <div className="flex flex-col p-4 w-full border transition-colors focus-within:border-gray-400 border-gray-600 rounded-md">
      <div className="flex flex-row">
        <div className="flex flex-row">
          <button onClick={() => setPlaying(true)} className="w-8 h-8">
            <PlayIcon color="white" width="100%" height="100%" />
          </button>
          <button onClick={() => setPlaying(false)} className="w-8 h-8">
            <PauseIcon color="white" width="100%" height="100%" />
          </button>
        </div>
        <Timestamp />
      </div>
      <div className="gap-1 flex flex-col overflow-y-hidden">
        <div className="z-20 flex flex-row gap-2">
          <div className="flex-shrink-0 w-96" />
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
              keyframes={flattenedKeyframesByEntity(entity)}
              animationData={entity.animation_data}
            />
          ))}
        </Reorder.Group>
      </div>
    </div>
  );
};

export default Timeline;
