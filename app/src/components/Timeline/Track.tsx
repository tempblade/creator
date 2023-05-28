import { ease } from "@unom/style";
import { useDragControls, Reorder, motion } from "framer-motion";
import { AnimationData, AnimatedEntity } from "primitives/AnimatedEntities";
import { Keyframe } from "primitives/Keyframe";
import { FC } from "react";
import { useEntitiesStore } from "stores/entities.store";
import { z } from "zod";
import { shallow } from "zustand/shallow";
import KeyframeIndicator from "./KeyframeIndicator";
import { TIMELINE_SCALE, calculateOffset } from "./common";

type TrackProps = {
  animationData: z.input<typeof AnimationData>;
  name: string;
  index: number;
  entity: z.input<typeof AnimatedEntity>;
  keyframes: Array<z.input<typeof Keyframe>>;
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
      className="h-8 relative flex flex-1 flex-row gap-1 select-none"
    >
      <div
        onMouseDown={(e) => e.preventDefault()}
        onPointerDown={(e) => controls.start(e)}
        className={`h-full transition-all rounded-sm min-w-[200px] p-1 px-2 flex flex-row ${
          selectedEntity === index ? "bg-gray-800" : "bg-gray-900"
        }`}
      >
        <h3
          onClick={() =>
            selectedEntity !== undefined && selectedEntity === index
              ? deselectEntity()
              : selectEntity(index)
          }
          className="text-white-800 select-none cursor-pointer"
        >
          {name}
        </h3>
      </div>

      <div
        style={{ width: TIMELINE_SCALE * 10 }}
        className="flex h-full flex-row relative bg-gray-900 select-none shrink-0"
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
          dragConstraints={{ left: 0 }}
          onDragEnd={(e, info) => {
            let offset = info.offset.x;

            offset = calculateOffset(offset);

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
          dragConstraints={{ left: 0 }}
          onDragEnd={(e, info) => {
            let offset = info.offset.x;

            offset = calculateOffset(offset);

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
          }}
          onMouseDown={(e) => e.preventDefault()}
          transition={ease.circ(0.8).out}
          onDragEnd={(_e, info) => {
            let offset = info.offset.x;

            offset = calculateOffset(offset);

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

export default Track;
