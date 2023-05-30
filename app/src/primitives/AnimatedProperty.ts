import { z } from "zod";
import { AnimatedValue } from "./Values";

export const AnimatedProperty = z.object({
  propertyPath: z.string(),
  animatedValue: AnimatedValue,
  label: z.string(),
});

export const AnimatedProperties = z.array(AnimatedProperty);
