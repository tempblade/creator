import { ease } from "@unom/style";
import { motion } from "framer-motion";
import { AnimationData } from "primitives/AnimatedEntities";
import { Keyframe } from "primitives/Keyframe";
import { FC } from "react";
import { z } from "zod";
import { TIMELINE_SCALE } from "./common";
import { AnimatedNumber, AnimatedVec2, AnimatedVec3 } from "primitives/Values";
import { useKeyframeStore } from "stores/keyframe.store";

const KeyframeIndicator: FC<{
  keyframe: z.input<typeof Keyframe>;
  animationData: z.input<typeof AnimationData>;
}> = ({ keyframe, animationData }) => {
  const { selectedKeyframe, selectKeyframe, deselectKeyframe } =
    useKeyframeStore();

  const selected = selectedKeyframe === keyframe.id;

  return (
    <motion.div
      drag="x"
      onMouseDown={(e) => e.preventDefault()}
      data-selected={selected}
      dragConstraints={{ left: 0 }}
      animate={{
        x: (animationData.offset + keyframe.offset) * TIMELINE_SCALE + 4,
      }}
      transition={ease.quint(0.4).out}
      style={{
        clipPath: "polygon(50% 0%, 100% 50%, 50% 100%, 0% 50%)",
      }}
      onClick={() =>
        selected ? deselectKeyframe() : selectKeyframe(keyframe.id)
      }
      className="bg-indigo-500 data-[selected=true]:bg-indigo-300 absolute w-2 h-2 z-30 select-none"
    />
  );
};

const AnimatedNumberKeyframeIndicator: FC<{
  animatedNumber: z.input<typeof AnimatedNumber>;
  animationData: z.input<typeof AnimationData>;
}> = ({ animatedNumber, animationData }) => {
  return (
    <>
      {animatedNumber.keyframes.values.map((keyframe) => (
        <KeyframeIndicator
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
}> = ({ animatedVec2, animationData, dimension }) => {
  if (dimension) {
    return (
      <AnimatedNumberKeyframeIndicator
        animationData={animationData}
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
}> = ({ animatedVec3, animationData, dimension }) => {
  if (dimension) {
    return (
      <AnimatedNumberKeyframeIndicator
        animationData={animationData}
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
