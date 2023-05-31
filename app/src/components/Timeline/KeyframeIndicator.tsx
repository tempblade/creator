import { ease } from "@unom/style";
import { PanInfo, motion } from "framer-motion";
import { AnimationData } from "primitives/AnimatedEntities";
import { Keyframe } from "primitives/Keyframe";
import { FC, useCallback, useMemo, useState } from "react";
import { z } from "zod";
import { TIMELINE_SCALE, calculateOffset } from "./common";
import { AnimatedNumber, AnimatedVec2, AnimatedVec3 } from "primitives/Values";
import { useKeyframeStore } from "stores/keyframe.store";
import { produce } from "immer";
import KeyframePopover from "./KeyframePopover";
import { Popover, PopoverContent, PopoverTrigger } from "components/Popover";

const KeyframeIndicator: FC<{
  keyframe: z.input<typeof Keyframe>;
  animationData: z.input<typeof AnimationData>;
  onUpdate?: (e: z.input<typeof Keyframe>) => void;
}> = ({ keyframe, animationData, onUpdate }) => {
  const { selectedKeyframe, selectKeyframe, deselectKeyframe } =
    useKeyframeStore();

  const handleUpdate = useCallback(
    (info: PanInfo) => {
      if (onUpdate) {
        let offset = info.offset.x;

        offset = calculateOffset(offset);

        offset += keyframe.offset;

        onUpdate({ ...keyframe, offset: offset < 0 ? 0 : offset });
      }
    },
    [onUpdate, animationData, keyframe]
  );

  const handleValueUpdate = useCallback(
    (keyframe: z.input<typeof Keyframe>) => {
      if (onUpdate) {
        onUpdate(keyframe);
      }
    },
    [onUpdate]
  );

  const selected = useMemo(
    () => selectedKeyframe === keyframe.id,
    [keyframe.id, selectedKeyframe]
  );

  const [isDragged, setIsDragged] = useState(false);

  return (
    <>
      <Popover modal={false} open={selected}>
        <PopoverTrigger asChild>
          <motion.div
            drag="x"
            variants={{
              enter: {},
              from: {},
              exit: {},
              tap: {},
              drag: {},
            }}
            data-selected={selected}
            onDragStart={() => setIsDragged(true)}
            onDragEnd={(e, info) => {
              e.preventDefault();
              setIsDragged(false);
              if (onUpdate) {
                handleUpdate(info);
              }
            }}
            dragConstraints={{ left: 0 }}
            initial={{
              x: (animationData.offset + keyframe.offset) * TIMELINE_SCALE + 2,
              scale: 0,
            }}
            whileTap={{
              scale: 1.6,
            }}
            animate={{
              x: (animationData.offset + keyframe.offset) * TIMELINE_SCALE + 2,
              scale: 1,
            }}
            transition={ease.quint(0.4).out}
            onClick={() => {
              if (isDragged) {
                if (!selected) selectKeyframe(keyframe.id);
              } else {
                selected ? deselectKeyframe() : selectKeyframe(keyframe.id);
              }
            }}
            className="h-full absolute z-30 select-none w-3 flex items-center justify-center filter
      data-[selected=true]:drop-shadow-[0px_2px_6px_rgba(230,230,255,1)] transition-colors"
          >
            <motion.span
              data-selected={selected}
              className="bg-gray-200 
        data-[selected=true]:bg-indigo-600 
        h-full transition-colors"
              style={{
                width: 10,
                height: 10,
                clipPath: "polygon(50% 0%, 100% 50%, 50% 100%, 0% 50%)",
              }}
            />
          </motion.div>
        </PopoverTrigger>
        <PopoverContent className="w-80 backdrop-blur-md bg-slate-700/50">
          <KeyframePopover
            onClose={() => deselectKeyframe()}
            onUpdate={handleValueUpdate}
            keyframe={keyframe}
          />
        </PopoverContent>
      </Popover>
    </>
  );
};

const AnimatedNumberKeyframeIndicator: FC<{
  animatedNumber: z.input<typeof AnimatedNumber>;
  animationData: z.input<typeof AnimationData>;
  onUpdate: (e: z.input<typeof AnimatedNumber>) => void;
}> = ({ animatedNumber, animationData, onUpdate }) => {
  return (
    <>
      {animatedNumber.keyframes.values.map((keyframe, index) => (
        <KeyframeIndicator
          onUpdate={(keyframe) =>
            onUpdate(
              produce(animatedNumber, (draft) => {
                draft.keyframes.values[index] = keyframe;
              })
            )
          }
          key={keyframe.id}
          keyframe={keyframe}
          animationData={animationData}
        />
      ))}
    </>
  );
};

type DimensionsVec2 = "x" | "y";
const VEC2_DIMENSION_INDEX_MAPPING: Record<DimensionsVec2, number> = {
  x: 0,
  y: 1,
};

const AnimatedVec2KeyframeIndicator: FC<{
  animatedVec2: z.input<typeof AnimatedVec2>;
  dimension?: DimensionsVec2;
  animationData: z.input<typeof AnimationData>;
  onUpdate: (e: z.input<typeof AnimatedVec2>) => void;
}> = ({ animatedVec2, animationData, dimension, onUpdate }) => {
  const handleUpdate = useCallback(
    (
      animatedNumber: z.input<typeof AnimatedNumber>,
      dimensionIndex: number
    ) => {
      onUpdate(
        produce(animatedVec2, (draft) => {
          draft.keyframes[dimensionIndex] = animatedNumber;
        })
      );
    },
    [animatedVec2]
  );

  if (dimension) {
    return (
      <AnimatedNumberKeyframeIndicator
        animationData={animationData}
        onUpdate={(animatedNumber) =>
          handleUpdate(animatedNumber, VEC2_DIMENSION_INDEX_MAPPING[dimension])
        }
        animatedNumber={
          animatedVec2.keyframes[VEC2_DIMENSION_INDEX_MAPPING[dimension]]
        }
      />
    );
  }

  return (
    <>
      {animatedVec2.keyframes.map((animatedNumber, index) => (
        <AnimatedNumberKeyframeIndicator
          onUpdate={(animatedNumber) => handleUpdate(animatedNumber, index)}
          key={index}
          animatedNumber={animatedNumber}
          animationData={animationData}
        />
      ))}
    </>
  );
};

type DimensionsVec3 = "x" | "y" | "z";
const VEC3_DIMENSION_INDEX_MAPPING: Record<DimensionsVec3, number> = {
  x: 0,
  y: 1,
  z: 2,
};

const AnimatedVec3KeyframeIndicator: FC<{
  animatedVec3: z.input<typeof AnimatedVec3>;
  animationData: z.input<typeof AnimationData>;
  dimension?: DimensionsVec3;
  onUpdate: (e: z.input<typeof AnimatedVec3>) => void;
}> = ({ animatedVec3, animationData, dimension, onUpdate }) => {
  const handleUpdate = useCallback(
    (
      animatedNumber: z.input<typeof AnimatedNumber>,
      dimensionIndex: number
    ) => {
      onUpdate(
        produce(animatedVec3, (draft) => {
          draft.keyframes[dimensionIndex] = animatedNumber;
        })
      );
    },
    [animatedVec3]
  );

  if (dimension) {
    return (
      <AnimatedNumberKeyframeIndicator
        animationData={animationData}
        onUpdate={(animatedNumber) =>
          handleUpdate(animatedNumber, VEC3_DIMENSION_INDEX_MAPPING[dimension])
        }
        animatedNumber={
          animatedVec3.keyframes[VEC3_DIMENSION_INDEX_MAPPING[dimension]]
        }
      />
    );
  }

  return (
    <>
      {animatedVec3.keyframes.map((animatedNumber, index) => (
        <AnimatedNumberKeyframeIndicator
          key={index}
          onUpdate={(animatedNumber) => handleUpdate(animatedNumber, index)}
          animatedNumber={animatedNumber}
          animationData={animationData}
        />
      ))}
    </>
  );
};

export {
  AnimatedNumberKeyframeIndicator,
  AnimatedVec3KeyframeIndicator,
  AnimatedVec2KeyframeIndicator,
};
export default KeyframeIndicator;
