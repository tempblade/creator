import { invoke } from "@tauri-apps/api";
import { AnimationData } from "primitives/AnimatedEntities";
import { AnimatedProperty } from "primitives/AnimatedProperty";
import { AnimatedValue, ValueType } from "primitives/Values";
import { FC, useEffect, useState } from "react";
import { z } from "zod";
import Graph from "./Graph";

type TrackPropertyPathProps = {
  animatedProperties: Array<z.input<typeof AnimatedProperty>>;
  animationData: z.input<typeof AnimationData>;
};

const TrackPropertyGraph: FC<TrackPropertyPathProps> = ({
  animatedProperties,
  animationData,
}) => {
  const [values, setValues] = useState<Array<Array<number>>>([]);

  useEffect(() => {
    const tasks: Array<Promise<Array<Array<number>>>> = [];

    animatedProperties.forEach((animatedProperty) => {
      animatedProperty.animatedValue.type;
      const animatedValue = animatedProperty.animatedValue;

      const commonValues: {
        animatedValue: z.input<typeof AnimatedValue>;
        startFrame: number;
        endFrame: number;
        fps: number;
        animationData: z.input<typeof AnimationData>;
      } = {
        animatedValue: AnimatedValue.parse(animatedValue),
        startFrame: 0,
        endFrame: 600,
        fps: 60,
        animationData: AnimationData.parse(animationData),
      };

      switch (animatedValue.type) {
        case ValueType.Enum.Number:
          tasks.push(
            invoke(
              "get_values_at_frame_range_from_animated_float",
              commonValues
            ).then((data) => {
              const numbers = data as Array<number>;

              return [numbers];
            })
          );
          break;
        case ValueType.Enum.Vec2:
          tasks.push(
            invoke(
              "get_values_at_frame_range_from_animated_float_vec2",
              commonValues
            ).then((data) => {
              const vectors = data as [Array<number>, Array<number>];

              const xValues = vectors.map((vec2) => vec2[0]);
              const yValues = vectors.map((vec2) => vec2[1]);

              return [xValues, yValues];
            })
          );
          break;

        case ValueType.Enum.Vec3:
          tasks.push(
            invoke(
              "get_values_at_frame_range_from_animated_float_vec3",
              commonValues
            ).then((data) => {
              const vectors = data as [
                Array<number>,
                Array<number>,
                Array<number>
              ];

              const xValues = vectors.map((vec2) => vec2[0]);
              const yValues = vectors.map((vec2) => vec2[1]);
              const zValues = vectors.map((vec2) => vec2[2]);

              return [xValues, yValues, zValues];
            })
          );
          break;
      }
    });

    Promise.all(tasks).then((values) => {
      const flatValues = values.flat();

      console.log("flattened Values", flatValues);
      setValues(flatValues);
    });
  }, animatedProperties);

  return (
    <div>
      <Graph values={values} />
    </div>
  );
};

export default TrackPropertyGraph;
