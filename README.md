# Creator — motion design tool (Rust + Skia + Tauri)

A keyframe-based motion design tool with a **UI-independent Rust engine**,
multiple render backends, headless CLI rendering, and (later) OpenFX support.
See [`PLAN.md`](./PLAN.md) for the full design and rationale.

The central thesis (PLAN.md §0): **pixels are produced in Rust**. The renderer
lives in the engine, not the webview; only edits and small events cross the UI
boundary. The same engine crate drives the desktop app *and* the headless CLI,
which guarantees the editor preview matches the final render.

## What's implemented

This repository implements the **UI-independent engine core + an always-available
CPU render path + a headless CLI** end-to-end, and scaffolds the GPU/desktop/OFX
work that requires native toolchains.

```
crates/
  anim      ✅ easings (Hold/Linear/CubicBezier via Newton-Raphson/Steps) +
               closed-form damped-harmonic-oscillator springs (under/critical/
               over-damped, physical + perceptual params, entry velocity)
  model     ✅ project → comps → layers/groups (parenting + precomps), typed
               animatable Property<T>, keyframes, slotmap stable IDs, linear
               Color, versioned serde JSON
  render    ✅ RenderTree concrete-value language, RenderTarget abstraction,
               CPU rasterizer: AA coverage, affine transforms, fills/strokes,
               linear + radial gradients, rect/ellipse/cubic-Bézier paths, blend
               modes, track mattes (alpha/luma ± inverted, chainable), group
               isolation, blur/tint — premultiplied linear light
  engine    ✅ pure deterministic eval(scene,t)→tree, motion blur (temporal
               supersampling, parallel sub-samples), command bus + undo/redo,
               frame cache keyed by (comp, frame, transitive content-hash)
  media     ✅ image import (sRGB→linear premultiplied)
  export    ✅ PNG (sRGB) + EXR (linear f32) writers, ffmpeg-sidecar builder
  cli       ✅ `creator render/sample/info` — headless, rayon frame-parallel
  gpu       ✅ backend abstraction + CPU bridge; **Skia-on-Vulkan offscreen
               backend** behind the `skia` feature (ash device + DirectContext,
               linear-F16 surface, full RenderTree translation incl. gradients,
               blends, effects, track mattes)
  ofx-host  🟡 host data model + lifecycle skeleton (deferred — Phase 5)
apps/desktop ✅ Tauri v2 shell: engine command API (open/scrub/edit/undo/redo)
               with the viewport rasterizing on **Skia/Vulkan** via a dedicated
               render thread (runtime CPU fallback); tier-3 readback display
               (excluded from the cargo workspace; `cargo build` in apps/desktop)
frontend     ✅ Vite + React + TS editor shell with Zod IPC boundary
               (`npm install && npm run build` in frontend/)
```

Mapping to PLAN.md §12: **Phase 1 (engine core) and the CPU slice of Phase 2
(renderer) are complete and tested; Phase 4's export + CLI parity is functional;
Phases 0/3/5 (GPU viewport bridge, full editor app, OpenFX) are scaffolded.**

### Settled open decisions (PLAN.md §13)

MVP defaults, all the plan's conservative recommendations:

1. Viewport bridge — tier 3 (readback) scaffolded; tiers 1–2 are the next spike.
2. Spring boundaries — **independent segments with a defined entry velocity**
   (fully closed-form).
3. Curve model — **CSS-style cubic-bezier** (AE influence/speed handles later).
4. OFX route — undecided; both routes captured in `ofx-host`.
5. Transforms — **2D** for v1 (2.5D later).
6. Web player — out of scope for now.

## Build & test

```sh
cargo test --workspace      # 67 unit/integration tests across the Rust crates
cargo build --release -p creator-cli
```

## Try the renderer (headless, no GPU required)

```sh
# write an example animated project (rotation, color, a spring bounce, gradient
# fills, motion blur, opacity + blur, a Screen blend, a text placeholder)
./target/release/creator sample demo.ctor

./target/release/creator info demo.ctor
./target/release/creator render demo.ctor --frames 0-90 --out frames --format png
# -> frames/frame_00000.png … frame_00090.png  (480x270)

# linear EXR instead of sRGB PNG:
./target/release/creator render demo.ctor --out exr --format exr

# stitch to video via the ffmpeg sidecar (requires ffmpeg on PATH):
ffmpeg -framerate 30 -i frames/frame_%05d.png -c:v libx264 -pix_fmt yuv420p demo.mp4
```

`--backend cpu` always works (CI / serverless). For GPU rendering build with
the `gpu` feature and pass `--backend vulkan` (offscreen — works headless on a
server with an NVIDIA/AMD/Intel Vulkan driver; no display needed):

```sh
cargo build --release -p creator-cli --features gpu
./target/release/creator render demo.ctor --backend vulkan --out frames
```

GPU output matches the CPU renderer to within AA/blur algorithm differences
(mean channel delta ≈ 0.07/255 on the demo). If the system lacks the
freetype/fontconfig `-dev` packages, `.cargo/config.toml` points the linker at
`.link-shims/` symlinks of the runtime libraries.

Rendering is parallel at two levels (PLAN.md §11): across frames (the CLI) and
inside each frame (row-parallel rasterization/blur and parallel motion-blur
sub-samples) — with bit-deterministic output regardless of thread count, since
all floating-point accumulation runs in a fixed order.

## Architecture invariant

> **`engine` and everything below it have zero dependency on Tauri/windowing.**

This is the hard rule (PLAN.md §3) that makes headless rendering and preview/final
parity possible. The CLI and the desktop app are both thin shells over the same
`creator-engine`.
