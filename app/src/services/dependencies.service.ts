import { invoke } from "@tauri-apps/api";
import { AnimatedEntities } from "primitives/AnimatedEntities";
import { Entities, Entity, EntityType } from "primitives/Entities";
import { z } from "zod";

function typedArrayToBuffer(array: Uint8Array): ArrayBuffer {
  return array.buffer.slice(
    array.byteOffset,
    array.byteLength + array.byteOffset
  );
}

export type Dependencies = {
  fonts: Map<string, ArrayBuffer>;
};

export class DependenciesService {
  dependencies: Dependencies;

  constructor() {
    this.dependencies = {
      fonts: new Map(),
    };
  }

  private async prepare(
    entities: z.output<typeof Entities> | z.output<typeof AnimatedEntities>
  ) {
    const fontNames = new Set<string>();

    entities.forEach((entity) => {
      if (entity.type === EntityType.Enum.Text) {
      }

      switch (entity.type) {
        case EntityType.Enum.Text:
          fontNames.add(entity.paint.fontName);
          break;
        case EntityType.Enum.StaggeredText:
          fontNames.add(entity.letter.paint.fontName);
          break;
        default:
          break;
      }
    });

    await this.loadFonts(fontNames);

    return this.dependencies;
  }

  async prepareForEntities(entities: z.output<typeof Entities>) {
    await this.prepare(entities);
  }

  async prepareForAnimatedEntities(
    animatedEntities: z.output<typeof AnimatedEntities>
  ) {
    await this.prepare(animatedEntities);
  }

  async loadFonts(fontNames: Set<string>) {
    const resolveFonts: Array<Promise<void>> = [];

    fontNames.forEach((fontName) => {
      if (!this.dependencies.fonts.has(fontName)) {
        resolveFonts.push(
          invoke("get_system_font", { fontName }).then((data) => {
            if (Array.isArray(data)) {
              const u8 = new Uint8Array(data as any);
              const buffer = typedArrayToBuffer(u8);
              this.dependencies.fonts.set(fontName, buffer);
            }
          })
        );
      }
    });

    await Promise.all(resolveFonts);

    console.log(this.dependencies);
  }
}
