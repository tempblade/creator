<img src="https://git.unom.io/tempblade/brand/raw/branch/master/creator/logo/macOS_App-Logo_Colored.png" width="150">

# tempblade Creator

tempblade creator is a motion design application, build with typescript, skia, rust using tauri.
Its currently in an early alpha stage, and there may be larger design changes to the overall structuring
of the project.

## Why?

Currently there isn't really an open source 2D motion design tool, and there is no tool that runs on linux, even if you're willing
to pay monthly. You could use blender, but its really not optimized for this use case (source: i've tried).

## How does it work?

Currently rust is used for things like keyframe interpolation and overall timeline calculation which then gets passed
to the frontend in javascript/typescript. This happens using tauris IPC.

### Current features

- Interpolate keyframes linear, eased using predefined functions or using spring simulations
- Creation, drawing and animation of the following primitives: rect, ellipse, text and staggered text
- A timeline for handling multiple primitives
- Handle complex staggered text animations, built on skias layout tools
- Stroke and fill paint
- Fully typed
- Multithreaded timeline/keyframe interpolation calculation using rayon
- Runtime typesafety thanks to zod in typescript
- Easy theming thanks to tailwindcss
- Cross platform font discovery and loading in rust thanks to FontKit
- Caching of skia entity instances like fonts etc.
- Pretty fast (if compared to After Effects)

### Features currently w.i.p

- Effects system for skias built in image filters like blurring

### Long-term goals

- Integration with OpenFX inside rust
- Standalone rust rust rendering using vulkan and or metal by using rust-skia
- Standalone package for drawing (currently the logic is already decoupled from ui)
- Caching system for the rendered keyframes (currently interpolation calculation happens during playback, this could easily be cached)