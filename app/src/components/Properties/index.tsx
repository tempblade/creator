import { FC, ReactNode } from "react";
import { useEntitiesStore } from "stores/entities.store";

import { shallow } from "zustand/shallow";
import { BoxProperties, EllipseProperties, TextProperties } from "./Primitives";

const PropertiesContainer: FC<{ children: ReactNode }> = ({ children }) => {
  return (
    <div className="w-full rounded-md h-[500px] overflow-auto border transition-colors focus-within:border-gray-400 border-gray-600 flex flex-col items-start p-4">
      {children}
    </div>
  );
};

const Properties = () => {
  const { selectedEntity, entities, updateEntity } = useEntitiesStore(
    (store) => ({
      updateEntity: store.updateEntity,
      selectedEntity: store.selectedEntity,
      entities: store.entities,
    }),
    shallow
  );

  const entity = selectedEntity !== undefined && entities[selectedEntity];

  if (entity) {
    switch (entity.type) {
      case "Text":
        return (
          <TextProperties
            key={selectedEntity}
            onUpdate={(entity) => updateEntity(selectedEntity, entity)}
            entity={entity}
          />
        );

      case "Box":
        return (
          <BoxProperties
            key={selectedEntity}
            onUpdate={(entity) => updateEntity(selectedEntity, entity)}
            entity={entity}
          />
        );

      case "Ellipse":
        return (
          <EllipseProperties
            key={selectedEntity}
            onUpdate={(entity) => updateEntity(selectedEntity, entity)}
            entity={entity}
          />
        );

      default:
        return null;
    }
  }

  return (
    <div>
      <h3>Wähle ein Element aus</h3>
    </div>
  );
};

export { PropertiesContainer };
export default Properties;
