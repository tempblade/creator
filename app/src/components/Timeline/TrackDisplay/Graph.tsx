import { FC } from "react";
import { extent, bisector } from "d3-array";
import { curveNatural } from "@visx/curve";

import { scaleLinear } from "@visx/scale";
import { LinePath } from "@visx/shape";
import { Group } from "@visx/group";

const HEIGHT = 300;
const WIDTH = 1200;

type PropertyValue = {
  value: number;
  frame: number;
};

const getValue = (d: PropertyValue) => d.value;
const getFrame = (d: PropertyValue) => d.frame;

const PropertyGraph: FC<{
  values: Array<{ frame: number; value: number }>;
}> = ({ values }) => {
  const framesScale = scaleLinear({
    range: [0, WIDTH],
    domain: extent(values, getFrame) as [number, number],
    nice: true,
  });

  const valuesScale = scaleLinear({
    range: [HEIGHT, 0],
    domain: extent(values, getValue) as [number, number],
    nice: true,
  });

  return (
    <Group>
      <LinePath
        curve={curveNatural}
        stroke="white"
        strokeWidth={3}
        data={values}
        x={(d) => framesScale(getFrame(d)) ?? 0}
        y={(d) => valuesScale(getValue(d)) ?? 0}
      />
    </Group>
  );
};

const Graphs: FC<{ values: Array<Array<number>> }> = ({ values }) => {
  return (
    <svg width={WIDTH} height={HEIGHT}>
      {values.map((propertyValues) => (
        <PropertyGraph
          values={propertyValues.map((val, index) => ({
            frame: index,
            value: val,
          }))}
        />
      ))}
    </svg>
  );
};

export default Graphs;
