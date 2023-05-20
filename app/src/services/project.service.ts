import { useTimelineStore } from "stores/timeline.store";
import { useEntitiesStore } from "stores/entities.store";
import { useRenderStateStore } from "stores/render-state.store";
import { z } from "zod";
import { Timeline } from "primitives/Timeline";

export class ProjectService {
  public saveProject() {
    const timelineStore = useTimelineStore.getState();
    const entitiesStore = useEntitiesStore.getState();
    const renderStateStore = useRenderStateStore.getState();

    const timeline: z.input<typeof Timeline> = {
      ...timelineStore,
      entities: entitiesStore.entities,
      render_state: renderStateStore.renderState,
    };

    const parsedTimeline = Timeline.parse(timeline);

    const serializedTimeline = JSON.stringify(parsedTimeline);

    return serializedTimeline;
  }

  public loadProject() {}
}
