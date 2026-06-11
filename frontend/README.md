# creator-frontend (Vite + React + TypeScript)

The editor UI (PLAN.md §1 Frontend, §12 Phase 3). **Not built in the headless
environment** (no Node toolchain). It is loaded by `apps/desktop` (Tauri) as the
webview frontend.

## Status (Phase 3 scaffold)

- ✅ Panel layout: toolbar, layer list, viewport, inspector, timeline scrubber.
- ✅ Typed IPC boundary in `src/api.ts` with **Zod** validation (PLAN.md §1
  "Keep Zod for boundary validation"). Commands out, summaries/frames in.
- ✅ Viewport shows the Rust-rendered frame (tier-3 readback as a PNG data URL).
- ⏳ Curve editor (bezier handles + spring controls), gizmos/transform handles,
  frame-cache indicator bar — the remaining Phase 3 panels.

## Develop

```sh
cd frontend
npm install
npm run dev        # http://localhost:5173 (usually launched via `cargo tauri dev`)
```

The backend commands it calls (`open_project`, `scrub`, `apply_edit`, `undo`,
`redo`) are defined in `../apps/desktop/src/main.rs`.

## Design notes

- The UI owns **no source of truth**: it dispatches commands and re-pulls state /
  re-scrubs. The engine's `Document` holds the project + undo/redo.
- Viewport **pixels never cross IPC as JSON**. Today they arrive as an image; the
  target state composites a native Rust/Skia GPU surface beneath a transparent
  webview (PLAN.md §7), with DOM-drawn gizmos on top.
