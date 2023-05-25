import { FC } from "react";
import * as Slider from "@radix-ui/react-slider";
import { useRenderStateStore } from "stores/render-state.store";
import { TIMELINE_SCALE } from "./Timeline";
import { useTimelineStore } from "stores/timeline.store";

export type TimePickerProps = {};

const TimePicker: FC<TimePickerProps> = () => {
  const { renderState, setCurrentFrame } = useRenderStateStore();
  const timeline = useTimelineStore();

  return (
    <Slider.Root
      className="relative flex select-none h-5 w-full items-center"
      defaultValue={[50]}
      style={{ width: TIMELINE_SCALE * timeline.duration }}
      value={[renderState.curr_frame]}
      onValueChange={(val) => setCurrentFrame(val[0])}
      max={timeline.fps * timeline.duration}
      step={1}
      aria-label="Current Frame"
    >
      <Slider.Track className="SliderTrack">
        <Slider.Range className="SliderRange" />
      </Slider.Track>
      <Slider.Thumb className="SliderThumb" />
    </Slider.Root>
  );
};

export default TimePicker;
