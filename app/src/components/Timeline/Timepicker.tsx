import { FC } from "react";
import * as Slider from "@radix-ui/react-slider";
import { useRenderStateStore } from "stores/render-state.store";
import { TIMELINE_SCALE } from "./common";
import { useTimelineStore } from "stores/timeline.store";

export type TimePickerProps = {};

const TimePicker: FC<TimePickerProps> = () => {
  const { renderState, setCurrentFrame } = useRenderStateStore();
  const timeline = useTimelineStore();

  return (
    <Slider.Root
      className="relative flex items-center select-none touch-none h-5 shrink-0"
      defaultValue={[50]}
      style={{ width: TIMELINE_SCALE * 10 }}
      value={[renderState.curr_frame]}
      onValueChange={(val) => setCurrentFrame(val[0])}
      max={timeline.fps * timeline.duration}
      step={1}
      aria-label="Current Frame"
    >
      <Slider.Track className="bg-blackA10 relative grow rounded-full h-[3px]">
        <Slider.Range className="absolute bg-white rounded-full h-full" />
      </Slider.Track>
      <Slider.Thumb
        className="block w-5 h-5 bg-white shadow-[0_2px_10px] shadow-blackA7 rounded-[10px] hover:bg-violet3 focus:outline-none focus:shadow-[0_0_0_5px] focus:shadow-blackA8"
        aria-label="Volume"
      />
    </Slider.Root>
  );
};

export default TimePicker;
