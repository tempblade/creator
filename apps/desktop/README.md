# creator-desktop (Tauri v2 shell)

The desktop editor shell. **Not part of the root cargo workspace** — it needs a
system webview and the Tauri CLI, which aren't present in the headless build
environment. It depends on the same `creator-engine` as the CLI, in-process
(PLAN.md §1: "Backend is Rust, in-process with your engine — no extra FFI/IPC
layer").

## Status

Builds and runs (verified headless via GTK Broadway: `broadwayd :5 &` then
`GDK_BACKEND=broadway BROADWAY_DISPLAY=:5 ./target/release/creator-desktop`).
On a desktop session just run the binary, or use `cargo tauri dev` for the
hot-reloading dev loop.

- ✅ Rust command API over the engine: `open_project`, `scrub`, `apply_edit`
  (rename / enable / opacity / position / background), `undo`, `redo`,
  `render_backend`.
- ✅ Commands flow through `creator_engine::Document`, so undo/redo history is
  authoritative on the Rust side (PLAN.md §2: "UI owns no source of truth").
- ✅ **Viewport rasterizes on Skia/Vulkan** when available (CPU fallback at
  runtime; `--no-default-features` removes the GPU dependency entirely).
  Skia's `DirectContext` is single-threaded, so a dedicated render thread owns
  the backend: `scrub` evaluates the scene to a `RenderTree` (pure + `Send`),
  ships it over a channel, and gets pixels back (`src/renderer.rs`).
- ⏳ Viewport bridge: still **tier 3** (readback → PNG data URL → `<img>`,
  PLAN.md §7). The render thread is the seam where tiers 1–2 (native surface
  composited under a transparent webview) plug in — present instead of reply.
- ⏳ Gizmos/overlays drawn in the DOM layer (kept out of the rendered frame).

## Prerequisites

- Rust + the Tauri v2 CLI: `cargo install tauri-cli --version '^2'`
- Node + the `../../frontend` app
- Platform webview libs:
  - Linux: `webkit2gtk-4.1`, `libsoup-3.0`
  - Windows: WebView2 runtime
  - macOS: WKWebView (system)

## Run

```sh
cd apps/desktop
cargo tauri dev      # builds the frontend (npm) + the Rust backend, opens the window
```

## Viewport bridge — next steps (PLAN.md §7)

The readback path here is fine for static preview but the readback + base64
transfer cost hurts at 60fps/4K scrub. The production path renders Rust/Skia GPU
pixels into a native layer composited with the webview:

1. macOS: render into a `CAMetalLayer` / `IOSurface`, place it under a
   transparent `WKWebView`.
2. Windows: a DirectComposition swapchain visual under WebView2.
3. Linux (WebKitGTK) is the fussiest — may fall back to the child-window tier.

Measure scrub latency at 1080p and 4K before committing (Phase 0 spike #3).
