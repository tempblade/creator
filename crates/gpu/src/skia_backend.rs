//! Skia-on-Vulkan offscreen backend (PLAN.md §6/§10, Phase 0 spike #2).
//!
//! Headless: no window, no swapchain — render straight to a GPU image and read
//! the pixels back. The Vulkan instance/device come from `ash`; Skia gets the
//! raw handles plus a `vkGetProcAddr` trampoline and drives the GPU through its
//! own Ganesh backend.
//!
//! Parity with the CPU rasterizer: the surface is **linear F16 premultiplied**
//! (`creator-render` composites in premultiplied linear light), colors/gradients
//! are fed as linear `Color4f`, and readback converts to the same
//! `CpuTarget` form the CPU path produces, so the existing PNG/EXR writers and
//! sRGB transfer apply unchanged.

use crate::{Backend, BackendKind, GpuError, Surface as GpuSurface};
use ash::vk as avk;
use ash::vk::Handle as _;
use creator_render::{
    CpuTarget, NodeContent, RenderNode, RenderTree, ResolvedEffect, ResolvedGeometry,
    ResolvedPaint, ResolvedShape, ResolvedText,
};
use skia_safe::{
    canvas::SaveLayerRec,
    color_filters,
    gpu::{direct_contexts, surfaces, vk, Budgeted, DirectContext, SurfaceOrigin},
    image_filters, luma_color_filter,
    paint::Style,
    AlphaType, BlendMode as SkBlend, Canvas, Color4f, ColorMatrix, ColorSpace, ColorType,
    ImageInfo, Matrix, Paint, Path, PathBuilder, Point, Rect, Shader, TileMode,
};
use std::cell::RefCell;
use std::ffi::CString;

/// Select the requested GPU backend. Only Vulkan is implemented (Metal is
/// macOS-only; this build targets Linux servers — PLAN.md §10).
pub fn select(kind: BackendKind) -> Result<Box<dyn Backend>, GpuError> {
    match kind {
        BackendKind::Vulkan => Ok(Box::new(VulkanBackend::new()?)),
        other => Err(GpuError::Init(format!(
            "{other:?} is not implemented in this build (only Vulkan offscreen)"
        ))),
    }
}

/// Owns the ash instance/device and the Skia `DirectContext` built on them.
///
/// Teardown order matters: the Skia context must release its GPU objects while
/// the device is still alive. `Drop::drop` runs before fields drop, so the
/// context lives in an `Option` that `Drop` takes and releases explicitly
/// before `vkDestroyDevice`. (Surfaces hold a context clone and must not
/// outlive the backend.)
pub struct VulkanBackend {
    context: Option<RefCell<DirectContext>>,
    device: ash::Device,
    instance: ash::Instance,
    #[allow(dead_code)]
    entry: ash::Entry,
}

impl VulkanBackend {
    pub fn new() -> Result<Self, GpuError> {
        unsafe {
            let entry = ash::Entry::load()
                .map_err(|e| GpuError::Init(format!("loading libvulkan: {e}")))?;

            let app_name = CString::new("creator").unwrap();
            let app_info = avk::ApplicationInfo::default()
                .application_name(&app_name)
                .api_version(avk::make_api_version(0, 1, 1, 0));
            let create_info = avk::InstanceCreateInfo::default().application_info(&app_info);
            let instance = entry
                .create_instance(&create_info, None)
                .map_err(|e| GpuError::Init(format!("vkCreateInstance: {e}")))?;

            // Prefer a discrete GPU; fall back to the first device (llvmpipe).
            let pdevs = instance
                .enumerate_physical_devices()
                .map_err(|e| GpuError::Init(format!("enumerating GPUs: {e}")))?;
            if pdevs.is_empty() {
                instance.destroy_instance(None);
                return Err(GpuError::Init("no Vulkan physical devices".into()));
            }
            let pdev = *pdevs
                .iter()
                .find(|&&d| {
                    instance.get_physical_device_properties(d).device_type
                        == avk::PhysicalDeviceType::DISCRETE_GPU
                })
                .unwrap_or(&pdevs[0]);

            let qfi = instance
                .get_physical_device_queue_family_properties(pdev)
                .iter()
                .position(|q| q.queue_flags.contains(avk::QueueFlags::GRAPHICS))
                .ok_or_else(|| GpuError::Init("no graphics queue family".into()))?
                as u32;

            let priorities = [1.0f32];
            let queue_info = avk::DeviceQueueCreateInfo::default()
                .queue_family_index(qfi)
                .queue_priorities(&priorities);
            let device_info =
                avk::DeviceCreateInfo::default().queue_create_infos(std::slice::from_ref(&queue_info));
            let device = instance
                .create_device(pdev, &device_info, None)
                .map_err(|e| GpuError::Init(format!("vkCreateDevice: {e}")))?;
            let queue = device.get_device_queue(qfi, 0);

            // Skia resolves every Vulkan function through this trampoline. The
            // closure (and BackendContext) borrow entry/instance, so they live
            // in an inner scope that ends before we move those into the struct.
            let context = {
                let get_proc = |of: vk::GetProcOf| match of {
                    vk::GetProcOf::Instance(inst, name) => entry
                        .get_instance_proc_addr(avk::Instance::from_raw(inst as u64), name)
                        .map_or(std::ptr::null(), |f| f as *const std::ffi::c_void),
                    vk::GetProcOf::Device(dev, name) => (instance.fp_v1_0()
                        .get_device_proc_addr)(
                        avk::Device::from_raw(dev as u64), name
                    )
                    .map_or(std::ptr::null(), |f| f as *const std::ffi::c_void),
                };
                let backend_context = vk::BackendContext::new(
                    instance.handle().as_raw() as _,
                    pdev.as_raw() as _,
                    device.handle().as_raw() as _,
                    (queue.as_raw() as _, qfi as usize),
                    &get_proc,
                );
                direct_contexts::make_vulkan(&backend_context, None)
            };
            let Some(context) = context else {
                device.destroy_device(None);
                instance.destroy_instance(None);
                return Err(GpuError::Init("Skia DirectContext creation failed".into()));
            };

            Ok(VulkanBackend {
                context: Some(RefCell::new(context)),
                device,
                instance,
                entry,
            })
        }
    }
}

impl Drop for VulkanBackend {
    fn drop(&mut self) {
        // Release Skia's GPU resources while the device is still alive, wait
        // for the queue to drain, then tear down Vulkan.
        if let Some(ctx) = self.context.take() {
            let mut ctx = ctx.into_inner();
            ctx.flush_and_submit();
            drop(ctx);
        }
        unsafe {
            let _ = self.device.device_wait_idle();
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

impl Backend for VulkanBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::Vulkan
    }

    fn create_offscreen(&self, width: u32, height: u32) -> Result<Box<dyn GpuSurface>, GpuError> {
        let context = self
            .context
            .as_ref()
            .expect("context only vacated during Drop");
        let info = ImageInfo::new(
            (width as i32, height as i32),
            ColorType::RGBAF16,
            AlphaType::Premul,
            ColorSpace::new_srgb_linear(),
        );
        let surface = surfaces::render_target(
            &mut context.borrow_mut(),
            Budgeted::Yes,
            &info,
            None,
            SurfaceOrigin::TopLeft,
            None,
            false,
            None,
        )
        .ok_or_else(|| GpuError::Init("offscreen surface allocation failed".into()))?;
        Ok(Box::new(VulkanSurface {
            surface: RefCell::new(surface),
            context: context.clone_ref(),
            width,
            height,
        }))
    }
}

// `DirectContext` is refcounted in Skia; cloning shares the context.
trait CloneRef {
    fn clone_ref(&self) -> RefCell<DirectContext>;
}
impl CloneRef for RefCell<DirectContext> {
    fn clone_ref(&self) -> RefCell<DirectContext> {
        RefCell::new(self.borrow().clone())
    }
}

struct VulkanSurface {
    surface: RefCell<skia_safe::Surface>,
    context: RefCell<DirectContext>,
    width: u32,
    height: u32,
}

impl GpuSurface for VulkanSurface {
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }

    fn draw(&mut self, tree: &RenderTree) {
        let mut surface = self.surface.borrow_mut();
        let canvas = surface.canvas();
        // `Canvas::clear` interprets its color as sRGB-encoded and converts to
        // the destination space — our background is already linear, so clear
        // transparent and fill through a paint tagged with the linear space.
        canvas.clear(Color4f::new(0.0, 0.0, 0.0, 0.0));
        if tree.background.a > 0.0 {
            let mut bg = Paint::default();
            bg.set_color4f(color4f(tree.background), Some(&ColorSpace::new_srgb_linear()));
            canvas.draw_paint(&bg);
        }
        draw_node(canvas, &tree.root);
        self.context.borrow_mut().flush_and_submit();
    }

    fn read_back(&self) -> CpuTarget {
        let mut surface = self.surface.borrow_mut();
        let count = (self.width as usize) * (self.height as usize);
        // Read as premultiplied linear F32 — the engine's native pixel form.
        let info = ImageInfo::new(
            (self.width as i32, self.height as i32),
            ColorType::RGBAF32,
            AlphaType::Premul,
            ColorSpace::new_srgb_linear(),
        );
        let mut buf = vec![[0.0f32; 4]; count];
        let bytes: &mut [u8] = bytemuck_cast_slice_mut(&mut buf);
        let row_bytes = (self.width as usize) * 16;
        if !surface.read_pixels(&info, bytes, row_bytes, (0, 0)) {
            // Leave the buffer transparent on failure rather than panicking.
        }
        CpuTarget::from_premultiplied(self.width, self.height, buf)
    }
}

/// Minimal safe cast of `[[f32;4]]` to bytes (avoids a bytemuck dependency).
fn bytemuck_cast_slice_mut(v: &mut [[f32; 4]]) -> &mut [u8] {
    unsafe { std::slice::from_raw_parts_mut(v.as_mut_ptr() as *mut u8, std::mem::size_of_val(v)) }
}

// --- render-tree translation -------------------------------------------------

fn draw_node(canvas: &Canvas, node: &RenderNode) {
    if node.needs_isolation() {
        // Outer layer: composited at restore with the node's opacity + blend
        // (content below draws at full opacity / Normal inside it).
        let mut outer = Paint::default();
        outer.set_alpha_f(node.opacity);
        outer.set_blend_mode(blend_mode(node.blend));
        canvas.save_layer(&SaveLayerRec::default().paint(&outer));

        // Inner layer for the effect chain so effects resolve BEFORE the matte
        // multiplies coverage (matches the CPU compositing order).
        if let Some(filter) = effect_filter(&node.effects) {
            let mut fx = Paint::default();
            fx.set_image_filter(filter);
            canvas.save_layer(&SaveLayerRec::default().paint(&fx));
            draw_transformed_content(canvas, node);
            canvas.restore();
        } else {
            draw_transformed_content(canvas, node);
        }

        // Track matte: draw the matte subtree into its own layer composited
        // with DstIn/DstOut so it scales what's already in the outer layer.
        if let Some((mode, matte_node)) = &node.matte {
            use creator_render::MatteMode;
            let mut matte_paint = Paint::default();
            match mode {
                MatteMode::Alpha => {
                    matte_paint.set_blend_mode(SkBlend::DstIn);
                }
                MatteMode::AlphaInverted => {
                    matte_paint.set_blend_mode(SkBlend::DstOut);
                }
                MatteMode::Luma => {
                    matte_paint.set_blend_mode(SkBlend::DstIn);
                    matte_paint.set_color_filter(luma_color_filter::new());
                }
                MatteMode::LumaInverted => {
                    matte_paint.set_blend_mode(SkBlend::DstOut);
                    matte_paint.set_color_filter(luma_color_filter::new());
                }
            }
            canvas.save_layer(&SaveLayerRec::default().paint(&matte_paint));
            draw_node(canvas, matte_node);
            canvas.restore();
        }

        canvas.restore();
    } else {
        match &node.content {
            NodeContent::Group(_) => draw_transformed_content(canvas, node),
            _ => {
                canvas.save();
                canvas.concat(&matrix(node.transform));
                draw_leaf(canvas, &node.content, node.opacity, blend_mode(node.blend));
                canvas.restore();
            }
        }
    }
}

/// Apply the node's transform and draw its content at opacity 1 / Normal.
fn draw_transformed_content(canvas: &Canvas, node: &RenderNode) {
    canvas.save();
    canvas.concat(&matrix(node.transform));
    match &node.content {
        NodeContent::Group(children) => {
            for child in children {
                draw_node(canvas, child);
            }
        }
        other => draw_leaf(canvas, other, 1.0, SkBlend::SrcOver),
    }
    canvas.restore();
}

fn draw_leaf(canvas: &Canvas, content: &NodeContent, opacity: f32, blend: SkBlend) {
    match content {
        NodeContent::Shape(shape) => draw_shape(canvas, shape, opacity, blend),
        NodeContent::Text(text) => draw_text_placeholder(canvas, text, opacity, blend),
        NodeContent::Group(_) => unreachable!("groups handled by draw_node"),
    }
}

fn draw_shape(canvas: &Canvas, shape: &ResolvedShape, opacity: f32, blend: SkBlend) {
    let path = geometry_path(&shape.geometry);

    if let Some(paint_def) = &shape.fill {
        let mut paint = base_paint(opacity, blend);
        apply_fill(&mut paint, paint_def);
        paint.set_style(Style::Fill);
        canvas.draw_path(&path, &paint);
    }
    if let Some(stroke) = shape.stroke {
        let mut paint = base_paint(opacity, blend);
        paint.set_color4f(color4f(stroke.color), &ColorSpace::new_srgb_linear());
        paint.set_style(Style::Stroke);
        paint.set_stroke_width(stroke.width);
        canvas.draw_path(&path, &paint);
    }
}

/// Same crude metrics box the CPU backend draws until SkParagraph text lands.
fn draw_text_placeholder(canvas: &Canvas, text: &ResolvedText, opacity: f32, blend: SkBlend) {
    let chars = text.content.chars().count().max(1) as f32;
    let w = chars * text.font_size * 0.55 + text.tracking * chars;
    let lines = text.content.split('\n').count().max(1) as f32;
    let h = text.font_size * text.line_height * lines;
    let mut paint = base_paint(opacity, blend);
    paint.set_color4f(color4f(text.color), &ColorSpace::new_srgb_linear());
    canvas.draw_rect(Rect::from_xywh(-w * 0.5, -h * 0.5, w, h), &paint);
}

fn base_paint(opacity: f32, blend: SkBlend) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_blend_mode(blend);
    paint.set_alpha_f(opacity);
    paint
}

fn geometry_path(geom: &ResolvedGeometry) -> Path {
    let mut b = PathBuilder::new();
    match geom {
        ResolvedGeometry::Rect { size, corner_radius } => {
            let rect = Rect::from_xywh(-size.x * 0.5, -size.y * 0.5, size.x, size.y);
            if *corner_radius > 0.0 {
                b.add_rrect(
                    skia_safe::RRect::new_rect_xy(rect, *corner_radius, *corner_radius),
                    None,
                    None,
                );
            } else {
                b.add_rect(rect, None, None);
            }
        }
        ResolvedGeometry::Ellipse { size } => {
            b.add_oval(Rect::from_xywh(-size.x * 0.5, -size.y * 0.5, size.x, size.y), None, None);
        }
        ResolvedGeometry::Path { polylines } => {
            for poly in polylines {
                let pts: Vec<Point> = poly.points.iter().map(|p| Point::new(p.x, p.y)).collect();
                if pts.len() >= 2 {
                    b.add_polygon(&pts, poly.closed);
                }
            }
            b.set_fill_type(skia_safe::PathFillType::EvenOdd);
        }
    }
    b.detach()
}

#[allow(deprecated)] // the tuple-based gradient API is simpler than the Gradient builder
fn apply_fill(paint: &mut Paint, def: &ResolvedPaint) {
    let linear = ColorSpace::new_srgb_linear();
    match def {
        ResolvedPaint::Solid(c) => {
            paint.set_color4f(color4f(*c), Some(&linear));
        }
        ResolvedPaint::Linear { start, end, stops } => {
            let (colors, positions) = split_stops(stops);
            if let Some(shader) = Shader::linear_gradient_with_interpolation(
                (Point::new(start.x, start.y), Point::new(end.x, end.y)),
                (colors.as_slice(), linear),
                Some(positions.as_slice()),
                TileMode::Clamp,
                gradient_interpolation(),
                None,
            ) {
                paint.set_shader(shader);
            }
        }
        ResolvedPaint::Radial { center, radius, stops } => {
            let (colors, positions) = split_stops(stops);
            if let Some(shader) = Shader::radial_gradient_with_interpolation(
                (Point::new(center.x, center.y), *radius),
                (colors.as_slice(), linear),
                Some(positions.as_slice()),
                TileMode::Clamp,
                gradient_interpolation(),
                None,
            ) {
                paint.set_shader(shader);
            }
        }
    }
}

/// Interpolate gradient stops in premultiplied destination (linear) space,
/// matching the CPU rasterizer's `sample_stops`.
fn gradient_interpolation() -> skia_safe::gradient_shader::Interpolation {
    use skia_safe::gradient_shader::interpolation::{ColorSpace as CS, HueMethod, InPremul};
    skia_safe::gradient_shader::Interpolation {
        in_premul: InPremul::Yes,
        color_space: CS::Destination,
        hue_method: HueMethod::Shorter,
    }
}

fn split_stops(stops: &[(f32, creator_render::Color)]) -> (Vec<Color4f>, Vec<f32>) {
    let colors = stops.iter().map(|(_, c)| color4f(*c)).collect();
    let positions = stops.iter().map(|(o, _)| *o).collect();
    (colors, positions)
}

/// Build the composed image filter for the effect chain (in order).
fn effect_filter(effects: &[ResolvedEffect]) -> Option<skia_safe::ImageFilter> {
    let mut chain: Option<skia_safe::ImageFilter> = None;
    for fx in effects {
        let next = match fx {
            ResolvedEffect::GaussianBlur { radius } => {
                if *radius <= 0.0 {
                    continue;
                }
                // CPU blur = 3 box passes of half-width r; equivalent Gaussian
                // sigma = sqrt(r(r+1)).
                let r = radius.round().max(0.0);
                let sigma = (r * (r + 1.0)).sqrt();
                if sigma <= 0.0 {
                    continue;
                }
                image_filters::blur((sigma, sigma), None, chain.take(), None)
            }
            ResolvedEffect::Tint { color, amount } => {
                let a = amount.clamp(0.0, 1.0);
                // Straight-color lerp toward `color`, preserving alpha:
                // out = (1-a)·in + a·color  (color matrix on unpremul RGBA).
                #[rustfmt::skip]
                let m = ColorMatrix::new(
                    1.0 - a, 0.0, 0.0, 0.0, a * color.r,
                    0.0, 1.0 - a, 0.0, 0.0, a * color.g,
                    0.0, 0.0, 1.0 - a, 0.0, a * color.b,
                    0.0, 0.0, 0.0, 1.0, 0.0,
                );
                let cf = color_filters::matrix(&m, None);
                image_filters::color_filter(cf, chain.take(), None)
            }
        };
        chain = next;
    }
    chain
}

fn blend_mode(b: creator_render::BlendMode) -> SkBlend {
    use creator_render::BlendMode as B;
    match b {
        B::Normal => SkBlend::SrcOver,
        B::Multiply => SkBlend::Multiply,
        B::Screen => SkBlend::Screen,
        B::Add => SkBlend::Plus,
        B::Overlay => SkBlend::Overlay,
        B::Darken => SkBlend::Darken,
        B::Lighten => SkBlend::Lighten,
    }
}

fn color4f(c: creator_render::Color) -> Color4f {
    Color4f::new(c.r, c.g, c.b, c.a)
}

fn matrix(t: glam::Affine2) -> Matrix {
    let m = t.matrix2;
    let v = t.translation;
    // glam col-major [[a,b],[c,d]] maps (x,y) -> (a·x+c·y+tx, b·x+d·y+ty);
    // Matrix::new_all is (scaleX, skewX, transX, skewY, scaleY, transY, ...).
    Matrix::new_all(m.x_axis.x, m.y_axis.x, v.x, m.x_axis.y, m.y_axis.y, v.y, 0.0, 0.0, 1.0)
}
