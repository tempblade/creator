// Typed boundary to the Rust backend.
//
// Per PLAN.md §1/§2: the UI sends *commands* and receives *patches/events*, and
// it owns no source of truth. Every value crossing the Tauri IPC boundary is
// validated with Zod, so a backend/frontend mismatch fails loudly instead of
// corrupting UI state. Viewport *pixels* never cross this boundary as JSON —
// they arrive as an image (tier-3 readback) or, in the target state, on a native
// surface beneath the webview.

import { invoke } from "@tauri-apps/api/core";
import { z } from "zod";

export const LayerSummary = z.object({
  name: z.string(),
  enabled: z.boolean(),
});

export const ProjectSummary = z.object({
  name: z.string(),
  width: z.number().int().positive(),
  height: z.number().int().positive(),
  frame_rate: z.number().positive(),
  duration: z.number().nonnegative(),
  layers: z.array(LayerSummary),
});
export type ProjectSummary = z.infer<typeof ProjectSummary>;

/** Edit commands; the discriminant matches the Rust `Edit` enum's serde tag. */
export type Edit =
  | { type: "Rename"; layer_index: number; name: string }
  | { type: "SetEnabled"; layer_index: number; enabled: boolean }
  | { type: "SetOpacity"; layer_index: number; opacity: number }
  | { type: "SetPosition"; layer_index: number; x: number; y: number }
  | { type: "SetBackground"; r: number; g: number; b: number; a: number };

export async function openProject(path: string): Promise<ProjectSummary> {
  const raw = await invoke("open_project", { path });
  return ProjectSummary.parse(raw);
}

/** Scrub to `time` seconds; returns a `data:image/png;base64,...` URL. */
export async function scrub(time: number): Promise<string> {
  return z.string().parse(await invoke("scrub", { time }));
}

export async function applyEdit(edit: Edit): Promise<void> {
  await invoke("apply_edit", { edit });
}

export async function undo(): Promise<string | null> {
  return (await invoke("undo")) as string | null;
}

export async function redo(): Promise<string | null> {
  return (await invoke("redo")) as string | null;
}

/** Which rasterizer backs the viewport ("vulkan" or "cpu"). */
export async function renderBackend(): Promise<string> {
  return z.string().parse(await invoke("render_backend"));
}
