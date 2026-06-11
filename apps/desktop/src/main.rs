//! Tauri v2 desktop backend.
//!
//! Architecture (PLAN.md §2): the React webview sends **commands** (edits, scrub
//! requests) and receives **patches/events**; it owns no source of truth. The
//! Rust side holds the `creator_engine::Document` (project + undo/redo) and the
//! render path. Viewport pixels are produced in Rust.
//!
//! This scaffold implements the **readback→`<canvas>` MVP tier** of the viewport
//! bridge (PLAN.md §7 tier 3): render in Rust, hand pixels to the webview as a
//! PNG data URL. The target state is tier 1 (a native `CAMetalLayer`/DComp layer
//! composited under a transparent webview) — see README.md. The final/headless
//! render path (the CLI) already proves preview/final parity.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod renderer;

use base64::Engine as _;
use creator_engine::{
    eval, Document, SetBackground, SetLayerEnabled, SetLayerName, SetOpacity, SetPosition,
};
use creator_export::encode_png;
use creator_model::{CompId, Color, Project, Property, Vec2};
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

/// App-wide state: the open document, guarded for cross-thread command access,
/// plus the viewport render thread (Vulkan with CPU fallback).
struct AppState {
    document: Mutex<Option<Document>>,
    comp: Mutex<Option<CompId>>,
    renderer: renderer::Renderer,
}

impl AppState {
    fn new() -> Self {
        AppState {
            document: Mutex::new(None),
            comp: Mutex::new(None),
            renderer: renderer::Renderer::start(),
        }
    }
}

/// A small summary of the open project for the UI to populate panels.
#[derive(Serialize)]
struct ProjectSummary {
    name: String,
    width: u32,
    height: u32,
    frame_rate: f64,
    duration: f64,
    layers: Vec<LayerSummary>,
}

#[derive(Serialize)]
struct LayerSummary {
    name: String,
    enabled: bool,
}

/// Open a project file and make its root comp current.
#[tauri::command]
fn open_project(state: State<AppState>, path: String) -> Result<ProjectSummary, String> {
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let project = Project::from_json(&text).map_err(|e| e.to_string())?;
    let comp_id = project.root.ok_or("project has no root composition")?;
    let summary = summarize(&project, comp_id)?;
    *state.document.lock().unwrap() = Some(Document::new(project));
    *state.comp.lock().unwrap() = Some(comp_id);
    Ok(summary)
}

fn summarize(project: &Project, comp_id: CompId) -> Result<ProjectSummary, String> {
    let c = project.composition(comp_id).ok_or("missing comp")?;
    Ok(ProjectSummary {
        name: c.name.clone(),
        width: c.width,
        height: c.height,
        frame_rate: c.frame_rate,
        duration: c.duration,
        layers: c
            .order
            .iter()
            .filter_map(|id| c.layer(*id))
            .map(|l| LayerSummary { name: l.name.clone(), enabled: l.enabled })
            .collect(),
    })
}

/// Scrub the playhead to `time` (seconds): evaluate on this thread (pure +
/// cheap), rasterize on the render thread — Skia/Vulkan when available — and
/// return the frame as a `data:image/png;base64,...` URL for the viewport.
#[tauri::command]
fn scrub(state: State<AppState>, time: f64) -> Result<String, String> {
    // Evaluate under the lock, then release it before rendering so edits are
    // never blocked behind rasterization.
    let tree = {
        let guard = state.document.lock().unwrap();
        let doc = guard.as_ref().ok_or("no project open")?;
        let comp = state.comp.lock().unwrap().ok_or("no comp")?;
        eval(&doc.project, comp, time)
    };
    let target = state.renderer.render(tree).ok_or("render thread unavailable")?;
    let bytes = encode_png(&target).map_err(|e| e.to_string())?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(format!("data:image/png;base64,{b64}"))
}

/// Which rasterizer backs the viewport ("vulkan" or "cpu") — for UI display.
#[tauri::command]
fn render_backend(state: State<AppState>) -> &'static str {
    state.renderer.backend
}

/// Edit commands the UI can dispatch. Each maps to an invertible engine command,
/// keeping the undo/redo history authoritative on the Rust side.
#[derive(serde::Deserialize)]
#[serde(tag = "type")]
enum Edit {
    Rename { layer_index: usize, name: String },
    SetEnabled { layer_index: usize, enabled: bool },
    SetOpacity { layer_index: usize, opacity: f64 },
    SetPosition { layer_index: usize, x: f32, y: f32 },
    SetBackground { r: f32, g: f32, b: f32, a: f32 },
}

#[tauri::command]
fn apply_edit(state: State<AppState>, edit: Edit) -> Result<(), String> {
    let mut guard = state.document.lock().unwrap();
    let doc = guard.as_mut().ok_or("no project open")?;
    let comp = state.comp.lock().unwrap().ok_or("no comp")?;
    let layer_id = |index: usize| -> Result<_, String> {
        doc.project
            .composition(comp)
            .and_then(|c| c.order.get(index).copied())
            .ok_or_else(|| format!("no layer at index {index}"))
    };
    match edit {
        Edit::Rename { layer_index, name } => {
            let layer = layer_id(layer_index)?;
            doc.execute(Box::new(SetLayerName { comp, layer, name }));
        }
        Edit::SetEnabled { layer_index, enabled } => {
            let layer = layer_id(layer_index)?;
            doc.execute(Box::new(SetLayerEnabled { comp, layer, enabled }));
        }
        Edit::SetOpacity { layer_index, opacity } => {
            let layer = layer_id(layer_index)?;
            doc.execute(Box::new(SetOpacity { comp, layer, opacity: Property::constant(opacity) }));
        }
        Edit::SetPosition { layer_index, x, y } => {
            let layer = layer_id(layer_index)?;
            doc.execute(Box::new(SetPosition { comp, layer, position: Property::constant(Vec2::new(x, y)) }));
        }
        Edit::SetBackground { r, g, b, a } => {
            doc.execute(Box::new(SetBackground { comp, color: Color::linear(r, g, b, a) }));
        }
    }
    Ok(())
}

#[tauri::command]
fn undo(state: State<AppState>) -> Option<String> {
    state.document.lock().unwrap().as_mut().and_then(|d| d.undo())
}

#[tauri::command]
fn redo(state: State<AppState>) -> Option<String> {
    state.document.lock().unwrap().as_mut().and_then(|d| d.redo())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            open_project,
            scrub,
            apply_edit,
            undo,
            redo,
            render_backend
        ])
        .run(tauri::generate_context!())
        .expect("error while running creator desktop");
}
