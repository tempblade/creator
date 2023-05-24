import { BaseEntity } from "primitives/Entities";
import { z } from "zod";

export interface EntityCache<T> {
  build: () => T;
  get: () => T | undefined;
  set: (id: string, cache: T) => void;
  cleanup: (cache: T) => void;
}

export function handleEntityCache<
  E extends z.output<typeof BaseEntity>,
  C,
  EC extends EntityCache<C>
>(entity: E, cache: EC): C {
  const cached = cache.get();

  if (!entity.cache.valid) {
    console.log("Invalid cache");
    if (cached) {
      cache.cleanup(cached);
    }
    return cache.build();
  } else {
    if (!cached) {
      const nextCache = cache.build();

      cache.set(entity.id, nextCache);

      return nextCache;
    } else {
      return cached;
    }
  }
}
