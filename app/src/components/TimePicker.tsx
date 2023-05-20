import { FC } from "react";
import * as Slider from "@radix-ui/react-slider";
import { useRenderStateStore } from "stores/render-state.store";

export type TimePickerProps = {};

const TimePicker: FC<TimePickerProps> = () => {
  const { renderState, setCurrentFrame } = useRenderStateStore();

  return (
    <Slider.Root
      className="relative flex select-none h-5 w-full items-center"
      defaultValue={[50]}
      style={{ width: 100 * 10 }}
      value={[renderState.curr_frame]}
      onValueChange={(val) => setCurrentFrame(val[0])}
      max={60 * 10}
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
