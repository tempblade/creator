import {
  AnimatedEntity,
  AnimationData,
  getAnimatedPropertiesByAnimatedEntity,
} from "primitives/AnimatedEntities";
import { AnimatedProperty } from "primitives/AnimatedProperty";
import { AnimatedVec2, ValueType } from "primitives/Values";
import { FC, memo, useCallback, useMemo, useState } from "react";
import { z } from "zod";
import {
  AnimatedNumberKeyframeIndicator,
  AnimatedVec2KeyframeIndicator,
  AnimatedVec3KeyframeIndicator,
} from "./KeyframeIndicator";
import { ToggleGroup, ToggleGroupItem } from "components/ToggleGroup";
import { produce } from "immer";
import set from "lodash.set";
import { useEntitiesStore } from "stores/entities.store";
import { shallow } from "zustand/shallow";
import { AnimatedValue } from "primitives/Values";
import { motion } from "framer-motion";
import { ease } from "@unom/style";

const TrackAnimatedPropertyKeyframes: FC<{
  animatedProperty: z.input<typeof AnimatedProperty>;
  animationData: z.input<typeof AnimationData>;
  onUpdate: (animatedProperty: z.input<typeof AnimatedProperty>) => void;
  selectedDimension?: "x" | "y" | "z";
}> = ({ animatedProperty, animationData, selectedDimension, onUpdate }) => {
  const handleUpdate = useCallback(
    (animatedValue: z.input<typeof AnimatedValue>) => {
      onUpdate({ ...animatedProperty, animatedValue });
    },
    [onUpdate, animatedProperty]
  );

  switch (animatedProperty.animatedValue.type) {
    case "Number":
      return (
        <AnimatedNumberKeyframeIndicator
          animatedNumber={animatedProperty.animatedValue}
          animationData={animationData}
          onUpdate={handleUpdate}
        />
      );
    case "Vec2":
      return (
        <AnimatedVec2KeyframeIndicator
          dimension={selectedDimension !== "z" ? selectedDimension : undefined}
          animatedVec2={animatedProperty.animatedValue}
          animationData={animationData}
          onUpdate={handleUpdate}
        />
      );

    case "Vec3":
      return (
        <AnimatedVec3KeyframeIndicator
          dimension={selectedDimension}
          animatedVec3={animatedProperty.animatedValue}
          animationData={animationData}
          onUpdate={handleUpdate}
        />
      );
    default:
      return null;
  }
};

const TrackAnimatedProperty: FC<{
  animatedProperty: z.input<typeof AnimatedProperty>;
  animationData: z.input<typeof AnimationData>;
  onUpdate: (e: z.input<typeof AnimatedProperty>) => void;
}> = ({ animatedProperty, animationData, onUpdate }) => {
  const [selectedDimension, setSelectedDimension] = useState<"x" | "y" | "z">();

  return (
    <motion.div
      transition={ease.quint(0.8).out}
      variants={{ enter: { y: 0, opacity: 1 }, from: { y: -10, opacity: 0 } }}
      className="flex flex-row bg-neutral-accent ml-2 align-center"
    >
      <div className="min-w-[195px] flex flex-row justify-between px-2">
        <h4 className="text-main/70">{animatedProperty.label}</h4>
        <ToggleGroup>
          <ToggleGroupItem
            onClick={() =>
              selectedDimension === "x"
                ? setSelectedDimension(undefined)
                : setSelectedDimension("x")
            }
            selected={selectedDimension === "x"}
          >
            X
          </ToggleGroupItem>
          <ToggleGroupItem
            onClick={() =>
              selectedDimension === "y"
                ? setSelectedDimension(undefined)
                : setSelectedDimension("y")
            }
            selected={selectedDimension === "y"}
          >
            Y
          </ToggleGroupItem>
          {animatedProperty.animatedValue.type === ValueType.Enum.Vec3 && (
            <ToggleGroupItem
              onClick={() =>
                selectedDimension === "z"
                  ? setSelectedDimension(undefined)
                  : setSelectedDimension("z")
              }
              selected={selectedDimension === "z"}
            >
              Z
            </ToggleGroupItem>
          )}
        </ToggleGroup>
      </div>
      <div className="relative">
        <TrackAnimatedPropertyKeyframes
          selectedDimension={
            animatedProperty.animatedValue.type !== "Number"
              ? selectedDimension
              : undefined
          }
          onUpdate={onUpdate}
          animatedProperty={animatedProperty}
          animationData={animationData}
        />
      </div>
    </motion.div>
  );
};

const TrackPropertiesEditor: FC<{
  entity: z.input<typeof AnimatedEntity>;
}> = ({ entity }) => {
  const animatedProperties = useMemo(
    () => getAnimatedPropertiesByAnimatedEntity(entity),
    [entity]
  );

  const handleUpdate = useCallback(
    (animatedProperty: z.input<typeof AnimatedProperty>) => {
      const entitiesStore = useEntitiesStore.getState();

      const nextValue = produce(entity, (draft) => {
        const animatedValue = animatedProperty.animatedValue;

        set(draft, animatedProperty.propertyPath, animatedValue);
      });

      const parsedEntity = AnimatedEntity.parse(nextValue);

      console.log("reacreated callback");

      entitiesStore.updateEntityById(parsedEntity.id, parsedEntity);
    },
    [entity]
  );

  return (
    <motion.div
      animate="enter"
      initial="from"
      variants={{ enter: {}, from: {} }}
      transition={{ staggerChildren: 0.05 }}
      layout
      className="flex flex-col gap-1"
    >
      {animatedProperties.map((animatedProperty, index) => (
        <TrackAnimatedProperty
          onUpdate={handleUpdate}
          animationData={entity.animation_data}
          key={index}
          animatedProperty={animatedProperty}
        />
      ))}
    </motion.div>
  );
};

export default memo(TrackPropertiesEditor);
