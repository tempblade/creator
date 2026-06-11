# Motion Design Tool — Build Plan (Rust + Skia + Tauri)

A plan for a keyframe-based motion design tool with a Rust rendering core, multiple GPU backends (Metal/Vulkan), a web-based editor UI, headless CLI rendering, and OpenFX support.

-----

## 0. What went wrong in `creator` v1, and the one thesis for v2

In the previous attempt, Rust did keyframe/timeline math and handed the **values** to the webview, where **Skia CanvasKit (wasm) did the actual drawing**. The scene crossed Tauri IPC every frame. That architecture structurally cannot deliver the things you now want:

- No native Metal/Vulkan (drawing lives in a browser canvas).
- No headless server rendering (needs a browser/wasm runtime to draw).
- IPC pressure: scene/values serialized per frame.
- Interpolation recomputed during playback, nothing cached.

**v2 thesis:** the renderer moves *into* Rust (skia-safe, native GPU). **Pixels are produced in Rust. Only edits and small events cross the UI boundary. Viewport pixels travel over a native surface, never JSON IPC.** Everything below follows from this.

### Concrete v1 → v2 changes

1. Rendering in Rust via `skia-safe` native backends — not CanvasKit-in-webview.
1. The scene never crosses IPC per frame; only commands (edits) and notifications do.
1. The engine is a **UI-independent library** so the desktop app and the CLI share one render path (this is what unlocks server rendering *and* guarantees the preview matches the final render).
1. Cache rendered **frames** (RAM preview), not just interpolation results.
1. Keep **Tauri** (Rust-native, in-process). Skip Electrobun.

-----

## 1. Technology decisions

|Area         |Choice                                                                                                       |Why / alternative                                                                                                                                                                                                                                                                                                                                                     |
|-------------|-------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|UI shell     |**Tauri v2**                                                                                                 |Backend is Rust, in-process with your engine — no extra FFI/IPC layer. Electrobun’s premise is “no Rust” (Bun + Zig + system webview); it would force your engine behind a process/FFI boundary, and Bun’s ongoing Zig→Rust core rewrite puts Electrobun’s binding layer in flux.                                                                                     |
|Frontend     |**Vite + React + TypeScript**                                                                                |Your home turf. Keep Zod for boundary validation.                                                                                                                                                                                                                                                                                                                     |
|2D renderer  |**`skia-safe`** (rust-skia)                                                                                  |Mature (~v0.97+), feature flags `metal`, `vulkan`, `d3d`, `gl`, `textlayout`, `svg`, `skottie`. Alternative: **Vello** (pure-Rust, on `wgpu`) gives Metal/Vulkan/DX12 “for free” through one abstraction, but is younger on advanced filters/text and you’ve already invested in Skia’s text layout. Recommendation: **Skia now**, keep Vello as a known escape hatch.|
|GPU backends |Skia Ganesh/Graphite on **Metal (macOS)**, **Vulkan (Linux/Win)**, **CPU raster** (always, for CI/serverless)|With Skia you manage each backend’s context/surface setup yourself (more control, more code). Vello would hide that behind wgpu.                                                                                                                                                                                                                                      |
|Interpolation|Custom (`anim` crate)                                                                                        |Cubic-bezier easing via Newton-Raphson; springs via **closed-form** damped harmonic oscillator (see §5).                                                                                                                                                                                                                                                              |
|Project file |**serde + JSON** (diffable), optional binary later                                                           |Versioned with explicit migrations.                                                                                                                                                                                                                                                                                                                                   |
|Video export |**ffmpeg sidecar** first, **libav** (`ffmpeg-next`) later                                                    |Sidecar is robust/server-friendly for export; libav for frame-accurate video-layer *decode* when you add video layers.                                                                                                                                                                                                                                                |
|Effects      |Native Skia image filters first → **OpenFX host** later                                                      |No maintained OFX host crate exists; you build it on `openfx-sys` raw bindings (see §8).                                                                                                                                                                                                                                                                              |
|Math/util    |`glam` (vectors/matrices), `slotmap` (stable node IDs), `rayon` (parallel render)                            |—                                                                                                                                                                                                                                                                                                                                                                     |

-----

## 2. High-level architecture

```
┌─────────────────────────────────────────────────────────┐
│  Tauri app                                                │
│  ┌───────────────────────┐    ┌──────────────────────┐   │
│  │ React UI (webview)     │    │ Rust backend (in-proc)│   │
│  │  timeline, panels,     │◄──►│  command bus          │   │
│  │  inspector, gizmos     │ИПC │  document model       │   │
│  └───────────┬───────────┘    │  engine (eval+render) │   │
│              │ native surface  │  frame cache          │   │
│  ┌───────────▼───────────┐    └──────────┬───────────┘   │
│  │ Viewport: GPU surface  │◄──────────────┘ pixels         │
│  │ rendered by Rust/Skia  │   (native layer, not IPC)      │
│  └───────────────────────┘                                │
└─────────────────────────────────────────────────────────┘
        the SAME engine crate is also driven by:
┌─────────────────────────────────────────────────────────┐
│  CLI (headless): load project → eval@t → render → encode  │
└─────────────────────────────────────────────────────────┘
```

- UI sends **commands**; receives **patches/events**. UI owns no source of truth.
- Scrubbing = UI sends “playhead → t”; Rust evaluates + renders + presents to the viewport surface. (Contrast v1, where the UI received values and drew them.)

-----

## 3. Workspace layout

```
crates/
  model        # document: comps, layers, groups, typed animatable properties, IDs, serde
  anim         # interpolation: easings, springs, evaluation (pure, deterministic)
  render       # render graph: evaluated tree → Skia draw; compositing, blend, masks
  gpu          # surface/context mgmt per backend (metal/vulkan/cpu), offscreen targets
  ofx-host     # OpenFX host (later phase)
  media        # image/video import + decode
  export       # encode image sequences / video
  engine       # ties model+anim+render+cache: "evaluate at t and render to a target"
  cli          # headless binary (clap)
apps/
  desktop      # Tauri backend: depends on `engine`, exposes commands to the UI
frontend/      # Vite + React + TS (Tauri frontend)
```

**Hard rule:** `engine` and everything below it have **zero** dependency on Tauri/windowing. That is what makes headless rendering and preview/final parity possible.

-----

## 4. Document model

- **Scene graph**: project → compositions → layers/groups (precomps allowed). Stable IDs via `slotmap` so the UI can reference nodes safely across edits.
- **Properties** are typed and animatable: `f64`, `Vec2`, `Vec3`, `Color`, `Path`, `Enum`, etc. A property is either constant or animated (a list of keyframes).
- **Keyframe**: `{ time, value, in_interp, out_interp }`.
- **Layer kinds (MVP)**: shape (rect/ellipse/path), text, group/precomp, (image/video later, adjustment later).
- **Transform**: anchor, position, scale, rotation, skew, opacity (2D affine). 2.5D/3D transforms optional later.
- **Effects**: ordered list per layer (native filters first; OFX later).
- **Undo/redo**: command-pattern bus. Each command is invertible and pushed to an undo stack. UI mutations go *only* through commands; Rust emits patches back so panels update. (Simpler and more controllable than event-sourcing or persistent data structures for an editor; revisit if snapshots become cheap-enough to prefer.)

-----

## 5. Animation & interpolation (`anim`)

The evaluation function must be a **pure, deterministic function of time** so the timeline is randomly scrubbable and caching is sound. `eval(scene, t) -> flattened render tree`.

### Easings

`Easing` enum: `Hold` (step), `Linear`, `CubicBezier(p1x,p1y,p2x,p2y)`, `Steps(n)`, plus named presets. Cubic-bezier maps `t → progress` by solving for the curve’s x (Newton-Raphson with a bisection fallback), the same technique browsers use for `cubic-bezier()`.

Consider AE-style temporal interpolation (influence/speed handles) later — it’s a richer model than CSS-style beziers and matches what motion designers expect from a curve editor.

### Springs (the important detail)

Use a **damped harmonic oscillator with a closed-form solution** for the under/critically/over-damped cases. Closed-form is non-negotiable: it makes evaluation at arbitrary `t` O(1) and deterministic, so springs work under scrubbing. (A naive `react-spring`-style step simulation is path-dependent — you can’t jump to time `t` without integrating from the start, which breaks the timeline model.)

Two parameterizations, expose both:

- **Physical**: mass, stiffness, damping, initial velocity.
- **Perceptual**: response/duration + damping fraction (or `bounce`), matching the SwiftUI/Motion mental model you already know.

**Semantics in a keyframe context:** a spring segment animates from `kf[i].value` toward `kf[i+1].value`, settling at the target. Decide how velocity behaves at keyframe boundaries:

- Treat each segment independently with a defined entry velocity (simplest, fully closed-form), **or**
- Pre-bake boundary velocities once on edit so chained springs feel continuous while evaluation stays closed-form at playback.

-----

## 6. Rendering (`render` + `gpu`)

Pipeline per frame:

1. `eval(scene, t)` → flattened tree with concrete values.
1. Walk tree; for each composition allocate a surface and render layers back-to-front.
1. Per layer: build Skia draw ops (paths, fills/strokes/gradients, text via SkParagraph), apply transform + opacity, then its effect chain.
1. Composite: blend modes, masks (clip/alpha), track mattes, adjustment layers, nested precomps.
1. Present to the **viewport surface** (interactive) **or** to an **offscreen surface** (export).

Design choices:

- **Composite in linear light, F16** for correct blending and to match OFX’s float clips. Apply the display transform at the end.
- **Target abstraction**: `RenderTarget` over {window surface (Metal/Vulkan), offscreen image (GPU or CPU raster)}. The engine renders the same scene to any target — this is the preview/final/headless unifier.
- **Text**: `skia-safe` `textlayout` + Harfbuzz/ICU. Carry over your staggered-text work from v1.

-----

## 7. The viewport bridge (the make-or-break risk — prototype first)

Showing a Rust-rendered GPU surface *inside* a Tauri webview, with crisp React chrome on top, is the hardest part. Three tiers:

1. **Native layer / zero-copy (target state).** Render into a `CAMetalLayer`/IOSurface (macOS) or DComp/swapchain visual (Windows) composited beneath/within the WKWebView/WebView2, webview transparent where the viewport sits. Linux (WebKitGTK) is the fussiest. This is what pro tools do.
1. **Separate native child window** layered with the webview (Tauri v2’s improved multi-webview/child-window support). Simpler than true zero-copy, good enough for many cases.
1. **Readback → canvas (MVP fallback).** Render in Rust, copy pixels to the UI via a custom protocol/shared memory, draw to `<canvas>`. Fine for static preview; the readback + transfer cost hurts at 60fps/4K scrub.

**Decouple interactive viewport from final render.** The final/headless path (offscreen surface → buffer → encode) is easy and not real-time-constrained. Front-load only the *interactive* display problem.

**Gizmos/handles**: draw selection/transform overlays in the DOM/canvas layer for crisp UI; do hit-testing either in JS or by asking Rust. Keep them out of the rendered frame.

-----

## 8. OpenFX host (`ofx-host`) — late, isolated, highest-risk

Reality check from the ecosystem: there is **no maintained OFX *host* crate** in Rust. `openfx-sys` gives raw bindgen bindings to the OpenFX C headers (suites + constants); `ofx`/`ofx_sys` are old and plugin-author-oriented. The host glue you write yourself.

OFX is a C ABI: structs of function pointers (suites) plus header constants. Implementing a host means:

- **Suites**: PropertySuite, ImageEffectSuite, MemorySuite, MultiThreadSuite, MessageSuite first; ParameterSuite, InteractSuite, DrawSuite later.
- **Plugin lifecycle**: discover/load `.ofx` bundles, `OfxSetHost`, enumerate plugins, describe/createInstance/render/destroy.
- **Param mapping**: map OFX params ↔ your typed property system.
- **Image/clip memory**: OFX clips are CPU buffers (RGBA float/byte) in regions/render windows. Wire its multithread suite to your `rayon` pool.

Two implementation routes:

- **Reimplement suites in Rust** on top of `openfx-sys` (clean, more work), or
- **Wrap the C++ HostSupport** reference library from the OpenFX SDK via FFI (less from-scratch, adds a C++ build dep).

**Cost you can’t avoid:** OFX’s baseline is CPU buffers, so GPU effects mean readback → process → upload roundtrips. Effects sit in the per-layer chain; cache aggressively. (Newer OpenGL/CUDA/Metal render suites exist but are a later extension.)

**Validate against open-source plugins first** (the OFX example plugins / Natron-compatible filters). Commercial plugins have host-compat quirks — treat them as a later compatibility pass.

-----

## 9. Media & export

- **Image import**: `image` crate.
- **Video import/decode** (when you add video layers): `ffmpeg-next` (libav) for frame-accurate seeking.
- **Export**: image sequences (PNG/EXR) first; then video via **ffmpeg sidecar** (mp4/H.264, ProRes, webm). Sidecar keeps the core license-clean and server-robust.
- **Audio**: out of scope initially; add a reference track later for timing.
- **Color management**: design pixel formats as linear F16 internally now; add OCIO display transforms later for a correct linear workflow.

-----

## 10. Headless CLI (`cli`)

Same `engine`, no UI:

```
creator render project.ctor \
  --frames 0-120 \
  --out frames/ \
  --format exr|png|mp4 \
  --backend cpu|vulkan|metal \
  --threads N
```

- **CPU raster** always works (CI / serverless, no GPU).
- **Offscreen Vulkan** for Linux servers *with* a GPU (no surface/swapchain — render straight to an image). Metal only on macOS.
- Because the CLI and the app call the identical engine, **headless output is guaranteed to match the editor preview** — this is the payoff of the UI-independent engine and directly serves your “render projects in a server env” and data-driven use cases.

-----

## 11. Caching & performance

- **Frame cache / RAM preview**: cache rendered frames keyed by `(composition, t, input-hash)`; invalidate on edits to that subtree. Show an AE-style timeline cache bar. (This is the real fix for v1’s “interpolation during playback” pain — cache *renders*, not just interpolation.)
- **Layer-level caching**: reuse intermediate layer renders that didn’t change.
- **Parallel rendering**: tile/frame parallelism via `rayon`; background render queue for previews and export.
- **Eval memoization** where continuous-time evaluation is hot.

-----

## 12. Phased roadmap (risks front-loaded)

### Phase 0 — De-risking spikes (do before committing)

1. Skia GPU surface in a window, animating, on **Metal** and on **Vulkan**.
1. Headless: one frame → EXR/PNG via **CPU raster** *and* **offscreen Vulkan**.
1. **Viewport bridge**: a Rust-rendered GPU surface inside a Tauri v2 window with React overlay; measure scrub latency at 1080p and 4K.
1. Lock in Skia (or pivot to Vello) based on 1–3.

### Phase 1 — Engine core (UI-independent)

- `model`: comps/layers/groups, typed animatable properties, stable IDs, serde project format (versioned).
- `anim`: hold/linear/cubic-bezier/steps + closed-form springs.
- Evaluation: `scene@t → render tree`; pure, deterministic, unit-tested.
- Command bus + undo/redo.

### Phase 2 — Renderer

- Render tree → Skia: shapes, fills/strokes/gradients, text, transforms, opacity, groups/precomps.
- Linear-F16 compositing, blend modes, masks, track mattes.
- `RenderTarget` over window/offscreen, CPU/GPU.
- Wire viewport (live scrub) + CLI (offscreen).

### Phase 3 — Editor app

- Tauri shell; React panels: viewport, timeline (keyframes + bezier curve editor + spring controls), layer list, inspector, tools.
- Viewport gizmos (overlay), selection, transform handles.
- Save/load; undo/redo wired through the command bus.
- Frame cache / RAM preview + timeline cache indicator.

### Phase 4 — Media & export

- Image import; export image sequences → then video via ffmpeg sidecar.
- CLI parity pass; backend selection.

### Phase 5 — OpenFX host

- Build on `openfx-sys` (or wrap C++ HostSupport); implement core suites; plugin discovery/lifecycle; param + clip mapping; GPU↔CPU bridging; validate against open-source plugins.

### Phase 6 — Performance, color, polish

- Multi-threaded render queue; smarter cache invalidation.
- OCIO color management.
- Optional: 2.5D transforms, expressions, audio reference, Lottie import/export (`skottie`), wasm web-player target (reuse `anim` + CanvasKit), embeddable data-driven render API.

-----

## 13. Open decisions to settle early

1. **Viewport bridge tier** (native zero-copy vs child window vs readback) — decided by Phase 0 latency numbers and how much Linux you need.
1. **Spring boundary semantics** — independent segments vs pre-baked continuous velocities.
1. **Curve model** — CSS-style cubic-bezier only, or AE-style influence/speed handles in the editor.
1. **OFX host route** — reimplement suites in Rust vs wrap C++ HostSupport.
1. **2D vs 2.5D** transforms for v1 scope.
1. **Web-player target** — is the embeddable wasm/CanvasKit player in scope, or native app + CLI only?
