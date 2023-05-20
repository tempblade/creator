import { EXAMPLE_ANIMATED_ENTITIES } from "example";
import { produce } from "immer";
import { AnimatedEntities, AnimatedEntity } from "primitives/AnimatedEntities";
import { z } from "zod";
import { create } from "zustand";

interface EntitiesStore {
  entities: z.input<typeof AnimatedEntities>;
  selectedEntity: number | undefined;
  selectEntity: (index: number) => void;
  deselectEntity: () => void;
  updateEntity: (
    index: number,
    entity: Partial<z.input<typeof AnimatedEntity>>
  ) => void;
}

const useEntitiesStore = create<EntitiesStore>((set) => ({
  entities: EXAMPLE_ANIMATED_ENTITIES,
  selectEntity: (index) => set(() => ({ selectedEntity: index })),
  deselectEntity: () => set(() => ({ selectedEntity: undefined })),
  selectedEntity: undefined,
  updateEntity: (index, entity) =>
    set(({ entities }) => {
      const nextEntities = produce(entities, (draft) => {
        draft[index] = { ...draft[index], ...entity } as z.infer<
          typeof AnimatedEntity
        >;
      });

      return { entities: nextEntities };
    }),
}));

export { useEntitiesStore };