import {
  AnimatedEntity,
  AnimationData,
  getAnimatedPropertiesByAnimatedEntity,
} from "primitives/AnimatedEntities";
import { AnimatedProperty } from "primitives/AnimatedProperty";
import { AnimatedVec2, ValueType } from "primitives/Values";
import { FC, useMemo, useState } from "react";
import { z } from "zod";
import {
  AnimatedNumberKeyframeIndicator,
  AnimatedVec2KeyframeIndicator,
  AnimatedVec3KeyframeIndicator,
} from "./KeyframeIndicator";
import { ToggleGroup, ToggleGroupItem } from "components/ToggleGroup";

const TrackAnimatedPropertyKeyframes: FC<{
  animatedProperty: z.input<typeof AnimatedProperty>;
  animationData: z.input<typeof AnimationData>;
  selectedDimension?: "x" | "y" | "z";
}> = ({ animatedProperty, animationData, selectedDimension }) => {
  switch (animatedProperty.animatedValue.type) {
    case "Number":
      return (
        <AnimatedNumberKeyframeIndicator
          animatedNumber={animatedProperty.animatedValue}
          animationData={animationData}
        />
      );
    case "Vec2":
      return (
        <AnimatedVec2KeyframeIndicator
          dimension={selectedDimension !== "z" ? selectedDimension : undefined}
          animatedVec2={animatedProperty.animatedValue}
          animationData={animationData}
        />
      );

    case "Vec3":
      return (
        <AnimatedVec3KeyframeIndicator
          dimension={selectedDimension}
          animatedVec3={animatedProperty.animatedValue}
          animationData={animationData}
        />
      );
    default:
      return null;
  }
};

const TrackAnimatedProperty: FC<{
  animatedProperty: z.input<typeof AnimatedProperty>;
  animationData: z.input<typeof AnimationData>;
  trackIndex: number;
}> = ({ animatedProperty, animationData }) => {
  const [selectedDimension, setSelectedDimension] = useState<"x" | "y" | "z">();

  return (
    <div className="flex flex-row">
      <div className="min-w-[200px] flex flex-row justify-between">
        <h3>{animatedProperty.label}</h3>
        <ToggleGroup>
          <ToggleGroupItem
            onClick={() => setSelectedDimension("x")}
            selected={selectedDimension === "x"}
          >
            X
          </ToggleGroupItem>
          <ToggleGroupItem
            onClick={() => setSelectedDimension("y")}
            selected={selectedDimension === "y"}
          >
            Y
          </ToggleGroupItem>
          {animatedProperty.animatedValue.type === ValueType.Enum.Vec3 && (
            <ToggleGroupItem
              onClick={() => setSelectedDimension("z")}
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
          animatedProperty={animatedProperty}
          animationData={animationData}
        />
      </div>
    </div>
  );
};

const TrackPropertiesEditor: FC<{ entity: z.input<typeof AnimatedEntity> }> = ({
  entity,
}) => {
  const animatedProperties = useMemo(
    () => getAnimatedPropertiesByAnimatedEntity(entity),
    [entity]
  );

  return (
    <div>
      {animatedProperties.map((animatedProperty, index) => (
        <TrackAnimatedProperty
          trackIndex={index}
          animationData={entity.animation_data}
          key={index}
          animatedProperty={animatedProperty}
        />
      ))}
    </div>
  );
};

export default TrackPropertiesEditor;

AnimatedVec2._def.typeName;
