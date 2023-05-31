import { AnimatedNumber, AnimatedVec2 } from "primitives/Values";
import { PropertiesProps } from "./common";
import { FC } from "react";
import { z } from "zod";
import { produce } from "immer";
import { Interpolation, InterpolationType } from "primitives/Interpolation";
import { Color } from "primitives/Paint";
import { parseCssColor } from "@tempblade/common";
import { rgbToHex } from "utils";
import { SpringInterpolation } from "primitives/Interpolation";
import FloatInput from "components/Inputs/FloatInput";
import { Keyframe } from "primitives/Keyframe";

const SpringInterpolationProperties: FC<
  PropertiesProps<z.input<typeof SpringInterpolation>>
> = ({ entity, onUpdate }) => {
  return <div></div>;
};

export const InterpolationProperties: FC<
  PropertiesProps<z.input<typeof Interpolation>>
> = ({ entity, onUpdate }) => {
  return (
    <fieldset>
      <label className="label" htmlFor="interpolation-type">
        Interpolation Type
      </label>
      <select
        id="interpolation-type"
        onChange={(e) => {
          onUpdate({
            ...entity,
            type: e.target.value as any,
          });
        }}
        value={entity.type}
      >
        {Object.keys(InterpolationType.Values).map((key) => (
          <option key={key} value={key}>
            {key}
          </option>
        ))}
      </select>
    </fieldset>
  );
};

export const KeyframeProperties: FC<
  PropertiesProps<z.input<typeof Keyframe>>
> = ({ entity, onUpdate }) => {
  return (
    <>
      <fieldset>
        <label htmlFor="keyframe-offset">Offset</label>
        <FloatInput
          value={entity.offset}
          onChange={(value) =>
            onUpdate(
              produce(entity, (draft) => {
                draft.offset = value;
              })
            )
          }
          id="keyframe-offset"
        />
      </fieldset>
      <fieldset>
        <label>Value</label>
        <FloatInput
          value={entity.value}
          onChange={(value) =>
            onUpdate(
              produce(entity, (draft) => {
                draft.value = value;
              })
            )
          }
          id="keyframe-value"
        />
      </fieldset>

      {entity.interpolation && (
        <InterpolationProperties
          onUpdate={(updatedEntity) =>
            onUpdate(
              produce(entity, (draft) => {
                draft.interpolation = updatedEntity;
              })
            )
          }
          entity={entity.interpolation}
        />
      )}
    </>
  );
};

const AnimatedNumberProperties: FC<
  PropertiesProps<z.input<typeof AnimatedNumber>> & { label: string }
> = ({ entity, onUpdate, label }) => {
  return (
    <div>
      <span>{label}</span>
      {entity.keyframes.values.map((keyframe, index) => {
        return (
          <div key={index}>
            <KeyframeProperties
              entity={keyframe}
              onUpdate={(nextKeyframe) =>
                onUpdate(
                  produce(entity, (draft) => {
                    draft.keyframes.values[index] = nextKeyframe;
                  })
                )
              }
            />
          </div>
        );
      })}
    </div>
  );
};

export const ColorProperties: FC<
  PropertiesProps<z.input<typeof Color>> & {
    label: string;
    mode?: "RGB" | "Picker";
  }
> = ({ entity, onUpdate, mode = "Picker" }) => {
  if (mode === "Picker") {
    return (
      <label className="flex flex-col items-start">
        <span className="label">Color</span>
        <div className="flex flex-row gap-3">
          <input
            value={rgbToHex(entity.value[0], entity.value[1], entity.value[2])}
            type="color"
            style={{
              width: 32,
              height: 32,
              backgroundColor: rgbToHex(
                entity.value[0],
                entity.value[1],
                entity.value[2]
              ),
            }}
            onChange={(e) =>
              onUpdate(
                produce(entity, (draft) => {
                  const color = parseCssColor(e.target.value);

                  if (color) {
                    draft.value = [...color, 1.0];
                  }
                })
              )
            }
          />
        </div>
      </label>
    );
  }

  return (
    <label className="flex flex-col items-start">
      <span className="label">Color</span>
      <div className="flex flex-row gap-3">
        <input
          value={entity.value[0]}
          type="number"
          max={255}
          onChange={(e) =>
            onUpdate(
              produce(entity, (draft) => {
                draft.value[0] = Number(e.target.value);
              })
            )
          }
        />
        <input
          value={entity.value[1]}
          type="number"
          max={255}
          onChange={(e) =>
            onUpdate(
              produce(entity, (draft) => {
                draft.value[1] = Number(e.target.value);
              })
            )
          }
        />
        <input
          value={entity.value[2]}
          type="number"
          max={255}
          onChange={(e) =>
            onUpdate(
              produce(entity, (draft) => {
                draft.value[2] = Number(e.target.value);
              })
            )
          }
        />
        <input
          value={entity.value[3]}
          type="number"
          max={1}
          onChange={(e) =>
            onUpdate(
              produce(entity, (draft) => {
                draft.value[3] = Number(e.target.value);
              })
            )
          }
        />
      </div>
    </label>
  );
};

export const AnimatedVec2Properties: FC<
  PropertiesProps<z.input<typeof AnimatedVec2>> & { label: string }
> = ({ entity, onUpdate, label }) => {
  return (
    <div>
      <label className="flex flex-col items-start">
        <span className="label">{label}</span>
        <AnimatedNumberProperties
          entity={entity.keyframes[0]}
          label="X"
          onUpdate={(updatedEntity) =>
            onUpdate(
              produce(entity, (draft) => {
                draft.keyframes[0] = {
                  ...draft.keyframes[0],
                  ...updatedEntity,
                };
              })
            )
          }
        />
        <AnimatedNumberProperties
          entity={entity.keyframes[1]}
          label="Y"
          onUpdate={(updatedEntity) =>
            onUpdate(
              produce(entity, (draft) => {
                draft.keyframes[1] = {
                  ...draft.keyframes[1],
                  ...updatedEntity,
                };
              })
            )
          }
        />
      </label>
    </div>
  );
};
