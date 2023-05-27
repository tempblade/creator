import { ease } from "@unom/style";
import { motion } from "framer-motion";
import {
  AnimatedTextEntity,
  AnimatedRectEntity,
  AnimatedStaggeredTextEntity,
  AnimatedEllipseEntity,
} from "primitives/AnimatedEntities";
import { Paint, PaintStyle, PaintStyleType } from "primitives/Paint";
import { FC } from "react";
import { z } from "zod";
import { AnimatedVec2Properties, ColorProperties } from "./Values";
import { PropertiesProps } from "./common";
import { useFontsStore } from "stores/fonts.store";

type TextPropertiesProps = PropertiesProps<z.input<typeof AnimatedTextEntity>>;
type StaggeredTextPropertiesProps = PropertiesProps<
  z.input<typeof AnimatedStaggeredTextEntity>
>;
type PaintPropertiesProps = PropertiesProps<z.input<typeof Paint>>;
type RectPropertiesProps = PropertiesProps<z.input<typeof AnimatedRectEntity>>;
type EllipsePropertiesProps = PropertiesProps<
  z.input<typeof AnimatedEllipseEntity>
>;

export const PaintProperties: FC<PaintPropertiesProps> = ({
  entity,
  onUpdate,
}) => {
  return (
    <div>
      <label className="flex flex-col items-start">
        <span className="label">PaintStyle</span>
        <select
          value={entity.style.type}
          onChange={(e) => {
            if (entity.style.type !== e.target.value) {
              const paintStyle = { type: e.target.value };

              const parsedPaintStyle = PaintStyle.parse(paintStyle);

              onUpdate({ style: parsedPaintStyle });
            }
          }}
        >
          {Object.keys(PaintStyleType.Values).map((key) => (
            <option key={key} value={key}>
              {key}
            </option>
          ))}
        </select>
      </label>
      {entity.style.color && (
        <ColorProperties
          label="Color"
          onUpdate={(color) =>
            onUpdate({ ...entity, style: { ...entity.style, color } })
          }
          entity={entity.style.color}
        />
      )}
    </div>
  );
};

export const TextProperties: FC<TextPropertiesProps> = ({
  entity,
  onUpdate,
}) => {
  const { fonts } = useFontsStore();

  return (
    <motion.div
      variants={{ enter: { opacity: 1, y: 0 }, from: { opacity: 0, y: 50 } }}
      animate="enter"
      initial="from"
      transition={ease.quint(0.9).out}
    >
      <label className="flex flex-col items-start">
        <span className="label">Text</span>
        <input
          value={entity.text}
          onChange={(e) => onUpdate({ ...entity, text: e.target.value })}
        />
      </label>
      <label className="flex flex-col items-start">
        <span className="label">Size</span>
        <input
          value={entity.paint.size}
          onChange={(e) =>
            onUpdate({
              ...entity,
              paint: { ...entity.paint, size: Number(e.target.value) },
            })
          }
        ></input>
      </label>
      <label className="flex flex-col items-start">
        <span className="label">Font</span>
        <select
          onChange={(e) =>
            onUpdate({
              ...entity,
              cache: { valid: false },
              paint: { ...entity.paint, fontName: e.target.value },
            })
          }
          value={entity.paint.fontName}
        >
          {fonts.map((font) => (
            <option value={font} key={font}>
              {font}
            </option>
          ))}
        </select>
      </label>
      <label>
        <span className="label">Size</span>
        <input
          value={entity.paint.size}
          onChange={(e) =>
            onUpdate({
              ...entity,
              cache: { valid: false },
              paint: { ...entity.paint, size: Number(e.target.value) },
            })
          }
        ></input>
      </label>
      <AnimatedVec2Properties
        onUpdate={(updatedEntity) =>
          onUpdate({ ...entity, origin: updatedEntity })
        }
        label="Origin"
        entity={entity.origin}
      />
    </motion.div>
  );
};

export const StaggeredTextProperties: FC<StaggeredTextPropertiesProps> = ({
  entity,
  onUpdate,
}) => {
  const { fonts } = useFontsStore();

  return (
    <motion.div
      variants={{ enter: { opacity: 1, y: 0 }, from: { opacity: 0, y: 50 } }}
      animate="enter"
      initial="from"
      transition={ease.quint(0.9).out}
    >
      <label className="flex flex-col items-start">
        <span className="label">Text</span>
        <input
          value={entity.text}
          onChange={(e) =>
            onUpdate({
              ...entity,
              text: e.target.value,
              cache: { valid: false },
            })
          }
        />
      </label>
      <label className="flex flex-col items-start">
        <span className="label">Font</span>
        <select
          onChange={(e) => {
            onUpdate({
              ...entity,
              cache: { valid: false },
              letter: {
                ...entity.letter,
                paint: { ...entity.letter.paint, fontName: e.target.value },
              },
            });
          }}
          value={entity.letter.paint.fontName}
        >
          {fonts.map((font) => (
            <option value={font} key={font}>
              {font}
            </option>
          ))}
        </select>
      </label>
      <label className="flex flex-col items-start">
        <span className="label">Size</span>
        <input
          value={entity.letter.paint.size}
          onChange={(e) =>
            onUpdate({
              ...entity,
              cache: { valid: false },
              letter: {
                ...entity.letter,
                paint: {
                  ...entity.letter.paint,
                  size: Number(e.target.value),
                },
              },
            })
          }
        ></input>
      </label>
      <PaintProperties
        entity={entity.letter.paint}
        onUpdate={(paint) =>
          onUpdate({
            ...entity,
            letter: {
              ...entity.letter,
              paint: { ...entity.letter.paint, ...paint },
            },
          })
        }
      />
      <AnimatedVec2Properties
        onUpdate={(updatedEntity) =>
          onUpdate({ ...entity, origin: updatedEntity })
        }
        label="Origin"
        entity={entity.origin}
      />
    </motion.div>
  );
};

export const RectProperties: FC<RectPropertiesProps> = ({
  entity,
  onUpdate,
}) => {
  return (
    <div className="dark:text-white">
      <PaintProperties
        entity={entity.paint}
        onUpdate={(paint) =>
          onUpdate({ ...entity, paint: { ...entity.paint, ...paint } })
        }
      />
      <AnimatedVec2Properties
        onUpdate={(updatedEntity) =>
          onUpdate({ ...entity, position: updatedEntity })
        }
        label="Position"
        entity={entity.position}
      />
      <AnimatedVec2Properties
        onUpdate={(updatedEntity) =>
          onUpdate({ ...entity, size: updatedEntity })
        }
        label="Size"
        entity={entity.size}
      />
    </div>
  );
};

export const EllipseProperties: FC<EllipsePropertiesProps> = ({
  entity,
  onUpdate,
}) => {
  return (
    <div className="dark:text-white">
      <PaintProperties
        entity={entity.paint}
        onUpdate={(paint) =>
          onUpdate({ ...entity, paint: { ...entity.paint, ...paint } })
        }
      />
      <AnimatedVec2Properties
        onUpdate={(updatedEntity) =>
          onUpdate({ ...entity, position: updatedEntity })
        }
        label="Position"
        entity={entity.position}
      />
      <AnimatedVec2Properties
        onUpdate={(updatedEntity) =>
          onUpdate({ ...entity, radius: updatedEntity })
        }
        label="Size"
        entity={entity.radius}
      />
    </div>
  );
};
