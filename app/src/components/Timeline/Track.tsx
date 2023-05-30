import { ease } from "@unom/style";
import { useDragControls, Reorder, motion } from "framer-motion";
import { AnimationData, AnimatedEntity } from "primitives/AnimatedEntities";
import { Keyframe } from "primitives/Keyframe";
import { FC, useState } from "react";
import { useEntitiesStore } from "stores/entities.store";
import { z } from "zod";
import { shallow } from "zustand/shallow";
import KeyframeIndicator, {
  AnimatedVec2KeyframeIndicator,
  AnimatedVec3KeyframeIndicator,
} from "./KeyframeIndicator";
import { TIMELINE_SCALE, calculateOffset } from "./common";
import { TriangleDownIcon } from "@radix-ui/react-icons";
import TrackPropertiesEditor from "./TrackPropertiesEditor";

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

  const [isExpanded, setIsExpanded] = useState(false);

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
      className="min-h-8 relative flex flex-1 flex-col gap-1 select-none"
    >
      <div className="flex flex-row gap-1 select-none">
        <div
          onMouseDown={(e) => e.preventDefault()}
          onPointerDown={(e) => controls.start(e)}
          className={`h-full transition-all rounded-sm min-w-[200px] p-1 px-2 flex flex-col ${
            selectedEntity === index ? "bg-gray-800" : "bg-gray-900"
          }`}
        >
          <div className="flex flex-row">
            <motion.div
              onClick={() => setIsExpanded(!isExpanded)}
              className="will-change-transform"
              animate={{ rotate: isExpanded ? 0 : -90 }}
            >
              <TriangleDownIcon
                width="32px"
                height="32px"
                className="text-white"
              />
            </motion.div>
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
        </div>

        <div
          style={{ width: TIMELINE_SCALE * 10 }}
          className="flex h-full flex-row relative bg-gray-900 select-none shrink-0"
        >
          {!isExpanded &&
            keyframes.map((keyframe, index) => (
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
            className="z-10 w-4 bg-slate-500 h-8 absolute rounded-md select-none cursor-w-resize"
          />
          <motion.div
            className="z-10 w-4 bg-slate-500 h-8 absolute rounded-md select-none cursor-e-resize"
            onMouseDown={(e) => e.preventDefault()}
            drag="x"
            animate={{
              x:
                (animationData.duration + animationData.offset) *
                  TIMELINE_SCALE -
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
            className="z-5 h-8 absolute rounded-md transition-colors bg-gray-700 hover:bg-gray-600 select-none cursor-grab"
          ></motion.div>
        </div>
      </div>
      {isExpanded && <TrackPropertiesEditor entity={entity} />}
    </Reorder.Item>
  );
};

export default Track;
