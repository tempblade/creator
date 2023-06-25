<img src="https://git.unom.io/tempblade/brand/raw/branch/master/creator/logo/macOS_App-Logo_Colored.png" width="150">

# tempblade Creator

tempblade creator is a motion design application, built on top of rust and skia. Its main goal is to be a flexible motion design toolkit, to be used in different environments. Right now it consists of an Editor/UI built with tauri where the ui uses react/typescript and the interpolation/timeline calculations are done in rust. It should also easily possible to run completly in the browser thanks to wasm. The project is currently in an early alpha stage, and there may be larger design changes to the overall structuring
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

### Possible use cases

- Typical motion design creation
- Data driven motion design -> craft your animation and automate the rendering/data population thanks to the soon coming typescript library
- Creation of generative art

### Features currently w.i.p

- Effects system for skias built in image filters like blurring

### Long-term goals

- Integration with OpenFX inside rust
- Standalone rust rust rendering using vulkan and or metal by using rust-skia
- Standalone ts package for drawing (currently the logic is already decoupled from ui)
- Standalone ts package for populating & rendering project files for enabling data driven integrations
- Caching system for the rendered keyframes (currently interpolation calculation happens during playback, this could easily be cached)
- Full pipeline automation with integrations for blender, houdini in more (Already working on this)
