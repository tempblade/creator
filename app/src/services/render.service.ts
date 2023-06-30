import { Timeline } from "primitives/Timeline";
// TODO: publish package maybe provide wrapper etc.
import * as creatorWasm from "../../../lib/creator_rs/pkg";
import { z } from "zod";
import { useTimelineStore } from "stores/timeline.store";
import { useEntitiesStore } from "stores/entities.store";
import { useRenderStateStore } from "stores/render-state.store";

export class RenderService {
    render() {

    }

    calculate() {
        const timelineStore = useTimelineStore.getState();
        const entitiesStore = useEntitiesStore.getState();
        const renderStateStore = useRenderStateStore.getState();


        let timeline: z.input<typeof Timeline> = {
            ...timelineStore,
            entities: entitiesStore.entities,
            render_state: renderStateStore.renderState
        }

        timeline = Timeline.parse(timeline);

        const renderedEntities = creatorWasm.calculate_timeline_from_json_at_curr_frame(JSON.stringify(timeline));

        console.log(renderedEntities);
    }
}