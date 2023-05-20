import { useRenderStateStore } from "stores/render-state.store";
import { useTimelineStore } from "stores/timeline.store";

const Timestamp = () => {
  const { renderState } = useRenderStateStore();
  const timeline = useTimelineStore();

  return (
    <div>
      <h3>Frame {renderState.curr_frame}</h3>
      <h2 className="text-xl font-bold">
        {((renderState.curr_frame * timeline.fps) / 60 / 60).toPrecision(3)}{" "}
        <span className="text-sm font-light">/ {timeline.fps}FPS</span>
      </h2>
    </div>
  );
};

export default Timestamp;
