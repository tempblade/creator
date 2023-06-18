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
import { ColorProperties } from "./Values";
import { PropertiesProps } from "./common";
import { useFontsStore } from "stores/fonts.store";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "components/Inputs/Select";

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
      <fieldset>
        <label htmlFor="staggered-text-letter-font">Font</label>
      </fieldset>
      <fieldset>
        <label htmlFor="paint-style-type">PaintStyle</label>

        <Select
          defaultValue={entity.style.type}
          onValueChange={(value) => {
            if (entity.style.type !== value) {
              const paintStyle = { type: value };

              const parsedPaintStyle = PaintStyle.parse(paintStyle);

              onUpdate({ style: parsedPaintStyle });
            }
          }}
        >
          <SelectTrigger>
            <SelectValue placeholder="Choose a paint style" />
          </SelectTrigger>
          <SelectContent id="paint-style-type" className="overflow-hidden">
            {Object.keys(PaintStyleType.Values).map((paintStyleType) => (
              <SelectItem value={paintStyleType}>{paintStyleType}</SelectItem>
            ))}
          </SelectContent>
        </Select>
      </fieldset>
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
      <fieldset>
        <label htmlFor="text-content">Text</label>
        <input
          id="text-content"
          value={entity.text}
          onChange={(e) => onUpdate({ ...entity, text: e.target.value })}
        />
      </fieldset>
      <fieldset>
        <label htmlFor="text-size">Size</label>
        <input
          id="text-size"
          value={entity.paint.size}
          onChange={(e) =>
            onUpdate({
              ...entity,
              paint: { ...entity.paint, size: Number(e.target.value) },
            })
          }
        />
      </fieldset>
      <fieldset>
        <label htmlFor="text-font">Font</label>

        <Select
          defaultValue={entity.paint.font_name}
          onValueChange={(val) => {
            onUpdate({
              ...entity,
              cache: { valid: false },
              paint: { ...entity.paint, font_name: val },
            });
          }}
        >
          <SelectTrigger>
            <SelectValue placeholder="Choose a font" />
          </SelectTrigger>
          <SelectContent>
            {fonts.map((font) => (
              <SelectItem value={font}>{font}</SelectItem>
            ))}
          </SelectContent>
        </Select>
      </fieldset>
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
      <fieldset>
        <label htmlFor="staggered-text-content">Text</label>
        <input
          id="staggered-text-content"
          value={entity.text}
          onChange={(e) =>
            onUpdate({
              ...entity,
              text: e.target.value,
              cache: { valid: false },
            })
          }
        />
      </fieldset>
      <fieldset>
        <label htmlFor="staggered-text-letter-font">Font</label>
        <Select
          defaultValue={entity.letter.paint.font_name}
          onValueChange={(val) => {
            onUpdate({
              ...entity,
              cache: { valid: false },
              letter: {
                ...entity.letter,
                paint: { ...entity.letter.paint, font_name: val },
              },
            });
          }}
        >
          <SelectTrigger>
            <SelectValue placeholder="Choose a font" />
          </SelectTrigger>
          <SelectContent className="overflow-hidden">
            {fonts.map((font) => (
              <SelectItem value={font}>{font}</SelectItem>
            ))}
          </SelectContent>
        </Select>
      </fieldset>
      <fieldset>
        <label htmlFor="staggered-text-letter-size">Size</label>
        <input
          id="staggered-text-letter-size"
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
        />
      </fieldset>
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
    </div>
  );
};
