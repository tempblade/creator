import { AnimatedNumber, AnimatedVec2 } from "primitives/Values";
import { PropertiesProps } from "./common";
import { FC } from "react";
import { z } from "zod";
import { produce } from "immer";
import { Interpolation } from "primitives/Interpolation";
import { Color } from "primitives/Paint";
import { colorToString, parseColor, parseCssColor } from "@tempblade/common";
import { rgbToHex } from "utils";

const InterpolationProperties: FC<
  PropertiesProps<z.input<typeof Interpolation>>
> = ({ entity, onUpdate }) => {
  return <div>Interpolation: {entity.type}</div>;
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
            <div className="flex flex-row gap-3">
              <label className="flex flex-col items-start w-16">
                <span className="label text-sm opacity-70">Offset</span>
                <input
                  value={keyframe.offset}
                  onChange={(e) =>
                    onUpdate(
                      produce(entity, (draft) => {
                        draft.keyframes.values[index].offset = Number(
                          e.target.value
                        );
                      })
                    )
                  }
                />
              </label>
              <label className="flex flex-col items-start">
                <span className="label text-sm opacity-70">Value</span>
                <input
                  value={keyframe.value}
                  onChange={(e) =>
                    onUpdate(
                      produce(entity, (draft) => {
                        draft.keyframes.values[index].value = Number(
                          e.target.value
                        );
                      })
                    )
                  }
                />
              </label>
            </div>
            {keyframe.interpolation && (
              <InterpolationProperties
                onUpdate={(updatedEntity) =>
                  onUpdate(
                    produce(entity, (draft) => {
                      draft.keyframes.values[index].interpolation =
                        updatedEntity;
                    })
                  )
                }
                entity={keyframe.interpolation}
              />
            )}
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
