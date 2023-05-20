export type PropertiesProps<E> = {
  entity: E;
  onUpdate: (entity: E) => void;
};
