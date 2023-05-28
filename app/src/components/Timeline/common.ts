export const TIMELINE_SCALE = 100;

export const calculateOffset = (offset: number) => {
  let nextOffset = offset / TIMELINE_SCALE;

  return nextOffset;
};
