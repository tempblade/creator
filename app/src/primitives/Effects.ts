import { z } from "zod";

export const EffectTypeOptions = ["Blur", "Erode", "Displace"] as const;

export const TileModeOptions = ["Clamp", "Decal", "Mirror", "Repeat"] as const;

export const EffectType = z.enum(EffectTypeOptions);
export const TileMode = z.enum(TileModeOptions);

export const EffectLayer = z.object({
  entityId: z.string().uuid(),
});

export const BlurEffectLayer = EffectLayer.extend({
  type: z.literal(EffectType.enum.Blur),
  amountX: z.number().min(0),
  amountY: z.number().min(0),
  tileMode: TileMode,
});
