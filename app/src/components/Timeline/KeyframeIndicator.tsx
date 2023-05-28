import { ease } from "@unom/style";
import { motion } from "framer-motion";
import { AnimationData } from "primitives/AnimatedEntities";
import { Keyframe } from "primitives/Keyframe";
import { FC } from "react";
import { z } from "zod";
import { TIMELINE_SCALE } from "./common";

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

export default KeyframeIndicator;
