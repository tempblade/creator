import { create } from "zustand";

interface TimelineStore {
  fps: number;
  duration: number;
  size: [number, number];
}

const useTimelineStore = create<TimelineStore>((set) => ({
  fps: 60,
  size: [1920, 1080],
  duration: 10.0,
}));

export { useTimelineStore };
