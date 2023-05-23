import { FC } from "react";
import { z } from "zod";
import { AnimationData } from "primitives/AnimatedEntities";
import { motion } from "framer-motion";
import TimePicker from "./TimePicker";
import { shallow } from "zustand/shallow";
import { useEntitiesStore } from "stores/entities.store";
import { ease } from "@unom/style";
import Timestamp from "./Timestamp";
import { Keyframe, Keyframes } from "primitives/Keyframe";
import { flattenedKeyframesByEntity } from "utils";

export type AnimationEntity = {
  offset: number;
  duration: number;
};

type TimelineProps = {};

type TrackProps = {
  animationData: z.input<typeof AnimationData>;
  name: string;
  index: number;
  keyframes: Array<z.input<typeof Keyframe>>;
};

const KeyframeIndicator: FC<{
  keyframe: z.input<typeof Keyframe>;
  animationData: z.input<typeof AnimationData>;
}> = ({ keyframe, animationData }) => {
  return (
    <motion.div
      animate={{
        x: (animationData.offset + keyframe.offset) * 100 + 4,
      }}
      transition={ease.quint(0.4).out}
      style={{
        clipPath: "polygon(50% 0%, 100% 50%, 50% 100%, 0% 50%)",
      }}
      className="bg-indigo-300 absolute w-2 h-2 z-30 top-[39%] select-none"
    ></motion.div>
  );
};

const Track: FC<TrackProps> = ({ keyframes, animationData, index, name }) => {
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
    <div className="h-8 w-100 flex flex-row gap-1">
      <div
        onClick={() =>
          selectedEntity !== undefined && selectedEntity === index
            ? deselectEntity()
            : selectEntity(index)
        }
        className={`h-full transition-all rounded-sm flex-shrink-0 w-96 p-1 px-2 flex flex-row ${
          selectedEntity === index ? "bg-gray-800" : "bg-gray-900"
        }`}
      >
        <h3 className="text-white-800">{name}</h3>
      </div>
      <div
        style={{ width: "1000px" }}
        className="flex w-full h-full flex-row relative bg-gray-900 select-none"
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
            x: animationData.offset * 100,
          }}
          whileHover={{
            scale: 1.1,
          }}
          whileTap={{
            scale: 0.9,
          }}
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
          className="z-10 w-4 bg-slate-500 h-full absolute rounded-md select-none"
        />
        <motion.div
          drag="x"
          animate={{
            x: (animationData.duration + animationData.offset) * 100 - 16,
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
          className="z-10 w-4 bg-slate-500 h-full absolute rounded-md select-none"
        />
        <motion.div
          drag="x"
          animate={{
            width: animationData.duration * 100,
            x: animationData.offset * 100,
          }}
          whileHover={{ scaleY: 1.1 }}
          whileTap={{ scaleY: 0.9 }}
          dragConstraints={{
            left: 0,
            right: 900,
          }}
          transition={ease.circ(0.8).out}
          onDragEnd={(e, info) => {
            let offset = info.offset.x;

            offset *= 0.01;

            offset += animationData.offset;

            console.log(offset);

            updateEntity(index, {
              animation_data: {
                ...animationData,
                offset: offset < 0 ? 0 : offset,
              },
            });
          }}
          className="z-5 h-full absolute rounded-md transition-colors bg-gray-700 hover:bg-gray-600 select-none"
        ></motion.div>
      </div>
    </div>
  );
};

const Timeline: FC<TimelineProps> = () => {
  const { entities } = useEntitiesStore((store) => ({
    entities: store.entities,
  }));

  return (
    <div className="flex flex-col p-4 border transition-colors focus-within:border-gray-400 border-gray-600 rounded-md">
      <Timestamp />
      <div className="gap-1 flex flex-col  overflow-hidden">
        <div className="z-20 flex flex-row gap-2">
          <div className="flex-shrink-0 w-96" />
          <TimePicker />
        </div>

        {entities.map((entity, index) => (
          <Track
            name={entity.type}
            index={index}
            key={index}
            keyframes={flattenedKeyframesByEntity(entity)}
            animationData={entity.animation_data}
          />
        ))}
      </div>
    </div>
  );
};

export default Timeline;
