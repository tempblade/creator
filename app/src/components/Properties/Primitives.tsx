import { ease } from "@unom/style";
import { motion } from "framer-motion";
import {
  AnimatedTextEntity,
  AnimatedBoxEntity,
  AnimatedEllipseEntity,
} from "primitives/AnimatedEntities";
import { Paint, PaintStyle, PaintStyleType } from "primitives/Paint";
import { FC } from "react";
import { z } from "zod";
import { AnimatedVec2Properties } from "./Values";
import { PropertiesProps } from "./common";

type TextPropertiesProps = PropertiesProps<z.input<typeof AnimatedTextEntity>>;
type PaintPropertiesProps = PropertiesProps<z.input<typeof Paint>>;
type BoxPropertiesProps = PropertiesProps<z.input<typeof AnimatedBoxEntity>>;
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
    </div>
  );
};

export const TextProperties: FC<TextPropertiesProps> = ({
  entity,
  onUpdate,
}) => {
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

export const BoxProperties: FC<BoxPropertiesProps> = ({ entity, onUpdate }) => {
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
