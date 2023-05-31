import { AnimatedEntity } from "primitives/AnimatedEntities";
import { Keyframe } from "primitives/Keyframe";
import { AnimatedNumber, AnimatedVec2, AnimatedVec3 } from "primitives/Values";
import { z } from "zod";
import { ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function flattenAnimatedNumberKeyframes(
  aNumber: z.input<typeof AnimatedNumber>
): Array<z.input<typeof Keyframe>> {
  return aNumber.keyframes.values;
}

function componentToHex(c: number) {
  var hex = c.toString(16);
  return hex.length == 1 ? "0" + hex : hex;
}

export function rgbToHex(r: number, g: number, b: number) {
  return "#" + componentToHex(r) + componentToHex(g) + componentToHex(b);
}

export function flattenAnimatedVec2Keyframes(
  aVec2: z.input<typeof AnimatedVec2>
): Array<z.input<typeof Keyframe>> {
  const keyframes: Array<z.input<typeof Keyframe>> = [
    ...flattenAnimatedNumberKeyframes(aVec2.keyframes[0]),
    ...flattenAnimatedNumberKeyframes(aVec2.keyframes[1]),
  ];

  return keyframes;
}

export function flattenAnimatedVec3Keyframes(
  aVec3: z.input<typeof AnimatedVec3>
): Array<z.input<typeof Keyframe>> {
  const keyframes: Array<z.input<typeof Keyframe>> = [
    ...flattenAnimatedNumberKeyframes(aVec3.keyframes[0]),
    ...flattenAnimatedNumberKeyframes(aVec3.keyframes[1]),
    ...flattenAnimatedNumberKeyframes(aVec3.keyframes[2]),
  ];

  return keyframes;
}

export function flattenedKeyframesByEntity(
  entity: z.input<typeof AnimatedEntity>
): Array<z.input<typeof Keyframe>> {
  const keyframes: Array<z.input<typeof Keyframe>> = [];

  switch (entity.type) {
    case "Text":
      keyframes.push(...flattenAnimatedVec2Keyframes(entity.origin));
      break;
    case "Rect":
      keyframes.push(...flattenAnimatedVec2Keyframes(entity.position));
      keyframes.push(...flattenAnimatedVec2Keyframes(entity.size));
      break;
    case "Ellipse":
      keyframes.push(...flattenAnimatedVec2Keyframes(entity.position));
      keyframes.push(...flattenAnimatedVec2Keyframes(entity.radius));
      break;
    case "StaggeredText":
      keyframes.push(
        ...flattenAnimatedVec3Keyframes(entity.letter.transform.rotate)
      );
      keyframes.push(
        ...flattenAnimatedVec2Keyframes(entity.letter.transform.translate)
      );
      keyframes.push(
        ...flattenAnimatedVec2Keyframes(entity.letter.transform.skew)
      );
      keyframes.push(
        ...flattenAnimatedVec2Keyframes(entity.letter.transform.scale)
      );
      keyframes.push(...flattenAnimatedVec2Keyframes(entity.origin));
      break;
    default:
      break;
  }

  return keyframes;
}

/**
 * Set a value inside an object with its path: example: set({}, 'a.b.c', '...') => { a: { b: { c: '...' } } }
 * If one of the keys in path doesn't exists in object, it'll be created.
 *
 * @param object Object to manipulate
 * @param path Path to the object field that need to be created/updated
 * @param value Value to set
 */
export function set(object: any, path: string, value: any) {
  const decomposedPath = path.split(".");
  const base = decomposedPath[0];

  if (base === undefined) {
    return object;
  }

  // assign an empty object in order to spread object
  if (!object.hasOwnProperty(base)) {
    object[base] = {};
  }

  // Determine if there is still layers to traverse
  value =
    decomposedPath.length <= 1
      ? value
      : set(object[base], decomposedPath.slice(1).join("."), value);

  return {
    ...object,
    [base]: value,
  };
}

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
