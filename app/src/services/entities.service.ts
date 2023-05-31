import { EntityType } from "primitives/Entities";
import { PaintStyleType, TextAlign } from "primitives/Paint";
import { staticAnimatedTransform, staticAnimatedVec2 } from "primitives/Values";
import { useEntitiesStore } from "stores/entities.store";
import { useTimelineStore } from "stores/timeline.store";
import { v4 as uuid } from "uuid";

export class EntitiesService {
  get entitiesStore() {
    return useEntitiesStore.getState();
  }

  private get timelineStore() {
    return useTimelineStore.getState();
  }

  createUuid() {
    return uuid();
  }

  createRect() {
    return this.entitiesStore.createEntity({
      type: EntityType.Enum.Rect,
      id: this.createUuid(),
      cache: {},
      paint: {
        style: {
          type: PaintStyleType.Enum.Fill,
          color: {
            value: [233, 100, 150, 1.0],
          },
        },
      },
      size: staticAnimatedVec2(500, 500),
      origin: staticAnimatedVec2(-250, -250),
      position: staticAnimatedVec2(...this.timelineStore.size),
      transform: staticAnimatedTransform([0, 0], [1, 1], [0, 0, 0], [0, 0]),
      animation_data: {
        offset: 0,
        duration: 3,
      },
    });
  }

  createEllipse() {
    return this.entitiesStore.createEntity({
      type: EntityType.Enum.Ellipse,
      id: this.createUuid(),
      cache: {},
      paint: {
        style: {
          type: PaintStyleType.Enum.Fill,
          color: {
            value: [233, 100, 150, 1.0],
          },
        },
      },
      radius: staticAnimatedVec2(500, 500),
      origin: staticAnimatedVec2(-250, -250),
      position: staticAnimatedVec2(...this.timelineStore.size),
      transform: staticAnimatedTransform([0, 0], [1, 1], [0, 0, 0], [0, 0]),
      animation_data: {
        offset: 0,
        duration: 3,
      },
    });
  }

  createText(text?: string) {
    return this.entitiesStore.createEntity({
      type: EntityType.Enum.Text,
      id: this.createUuid(),
      cache: {},
      text: text || "Hallo Welt",
      paint: {
        align: TextAlign.Enum.Center,
        size: 20,
        font_name: "Helvetica-Bold",
        style: {
          type: PaintStyleType.Enum.Fill,
          color: {
            value: [233, 100, 150, 1.0],
          },
        },
      },
      origin: staticAnimatedVec2(-250, -250),
      transform: staticAnimatedTransform([0, 0], [1, 1], [0, 0, 0], [0, 0]),
      animation_data: {
        offset: 0,
        duration: 3,
      },
    });
  }

  createStaggeredText(text?: string) {
    return this.entitiesStore.createEntity({
      type: EntityType.Enum.StaggeredText,
      id: this.createUuid(),
      cache: {},
      text: text || "Hallo Welt",
      stagger: 0.1,
      letter: {
        paint: {
          align: TextAlign.Enum.Center,
          size: 20,
          font_name: "Helvetica-Bold",
          style: {
            type: PaintStyleType.Enum.Fill,
            color: {
              value: [233, 100, 150, 1.0],
            },
          },
        },
        transform: staticAnimatedTransform([0, 0], [1, 1], [0, 0, 0], [0, 0]),
      },
      origin: staticAnimatedVec2(-250, -250),
      transform: staticAnimatedTransform([0, 0], [1, 1], [0, 0, 0], [0, 0]),
      animation_data: {
        offset: 0,
        duration: 3,
      },
    });
  }
}
