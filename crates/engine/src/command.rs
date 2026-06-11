//! Command bus with undo/redo (PLAN.md §4).
//!
//! Every edit goes through a [`Command`] whose `apply` mutates the project and
//! returns its **inverse**. The [`Document`] pushes inverses onto an undo stack;
//! undo applies an inverse (which yields the redo command), and so on. This is
//! simpler and more controllable than event-sourcing for an editor.
//!
//! The attribute/property/reorder commands are *exactly* invertible (stable
//! ids preserved). [`RemoveLayer`]'s undo re-inserts the layer's data and
//! restores its stacking position, but slotmap assigns a fresh id — a known MVP
//! limitation noted on that command.

use creator_model::{BlendMode, Color, Layer, LayerId, CompId, Project, Property};
use glam::Vec2;

/// An invertible edit. `apply` performs the edit and returns the command that
/// undoes it. `Send + Sync` so a `Document` can live in app state shared across
/// threads (e.g. Tauri's managed state); commands are plain data.
pub trait Command: Send + Sync {
    fn apply(&self, project: &mut Project) -> Box<dyn Command>;
    /// Short human-readable label (for an undo menu).
    fn label(&self) -> String;
}

/// A project plus its undo/redo history. The single entry point for edits.
pub struct Document {
    pub project: Project,
    undo: Vec<Box<dyn Command>>,
    redo: Vec<Box<dyn Command>>,
}

impl Document {
    pub fn new(project: Project) -> Self {
        Document { project, undo: Vec::new(), redo: Vec::new() }
    }

    /// Apply a command and record it for undo. Clears the redo stack.
    pub fn execute(&mut self, cmd: Box<dyn Command>) {
        let inverse = cmd.apply(&mut self.project);
        self.undo.push(inverse);
        self.redo.clear();
    }

    /// Undo the most recent command. Returns its label if anything was undone.
    pub fn undo(&mut self) -> Option<String> {
        let cmd = self.undo.pop()?;
        let label = cmd.label();
        let inverse = cmd.apply(&mut self.project);
        self.redo.push(inverse);
        Some(label)
    }

    /// Redo the most recently undone command.
    pub fn redo(&mut self) -> Option<String> {
        let cmd = self.redo.pop()?;
        let label = cmd.label();
        let inverse = cmd.apply(&mut self.project);
        self.undo.push(inverse);
        Some(label)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

/// Helper: run `f` on a layer if it exists.
fn with_layer<R>(project: &mut Project, comp: CompId, layer: LayerId, f: impl FnOnce(&mut Layer) -> R) -> Option<R> {
    project.composition_mut(comp)?.layer_mut(layer).map(f)
}

// --- concrete commands ------------------------------------------------------

/// Rename a layer.
pub struct SetLayerName {
    pub comp: CompId,
    pub layer: LayerId,
    pub name: String,
}
impl Command for SetLayerName {
    fn apply(&self, project: &mut Project) -> Box<dyn Command> {
        let old = with_layer(project, self.comp, self.layer, |l| {
            std::mem::replace(&mut l.name, self.name.clone())
        })
        .unwrap_or_default();
        Box::new(SetLayerName { comp: self.comp, layer: self.layer, name: old })
    }
    fn label(&self) -> String {
        format!("Rename layer to \"{}\"", self.name)
    }
}

/// Toggle a layer's enabled flag.
pub struct SetLayerEnabled {
    pub comp: CompId,
    pub layer: LayerId,
    pub enabled: bool,
}
impl Command for SetLayerEnabled {
    fn apply(&self, project: &mut Project) -> Box<dyn Command> {
        let old = with_layer(project, self.comp, self.layer, |l| {
            std::mem::replace(&mut l.enabled, self.enabled)
        })
        .unwrap_or(true);
        Box::new(SetLayerEnabled { comp: self.comp, layer: self.layer, enabled: old })
    }
    fn label(&self) -> String {
        format!("{} layer", if self.enabled { "Show" } else { "Hide" })
    }
}

/// Set a layer's blend mode.
pub struct SetLayerBlend {
    pub comp: CompId,
    pub layer: LayerId,
    pub blend: BlendMode,
}
impl Command for SetLayerBlend {
    fn apply(&self, project: &mut Project) -> Box<dyn Command> {
        let old = with_layer(project, self.comp, self.layer, |l| {
            std::mem::replace(&mut l.blend, self.blend)
        })
        .unwrap_or(BlendMode::Normal);
        Box::new(SetLayerBlend { comp: self.comp, layer: self.layer, blend: old })
    }
    fn label(&self) -> String {
        "Set blend mode".into()
    }
}

/// Replace a layer's opacity property (constant or animated).
pub struct SetOpacity {
    pub comp: CompId,
    pub layer: LayerId,
    pub opacity: Property<f64>,
}
impl Command for SetOpacity {
    fn apply(&self, project: &mut Project) -> Box<dyn Command> {
        let old = with_layer(project, self.comp, self.layer, |l| {
            std::mem::replace(&mut l.opacity, self.opacity.clone())
        })
        .unwrap_or_else(|| Property::constant(1.0));
        Box::new(SetOpacity { comp: self.comp, layer: self.layer, opacity: old })
    }
    fn label(&self) -> String {
        "Set opacity".into()
    }
}

/// Replace a layer's position property.
pub struct SetPosition {
    pub comp: CompId,
    pub layer: LayerId,
    pub position: Property<Vec2>,
}
impl Command for SetPosition {
    fn apply(&self, project: &mut Project) -> Box<dyn Command> {
        let old = with_layer(project, self.comp, self.layer, |l| {
            std::mem::replace(&mut l.transform.position, self.position.clone())
        })
        .unwrap_or_else(|| Property::constant(Vec2::ZERO));
        Box::new(SetPosition { comp: self.comp, layer: self.layer, position: old })
    }
    fn label(&self) -> String {
        "Move layer".into()
    }
}

/// Set a composition's background color.
pub struct SetBackground {
    pub comp: CompId,
    pub color: Color,
}
impl Command for SetBackground {
    fn apply(&self, project: &mut Project) -> Box<dyn Command> {
        let old = project
            .composition_mut(self.comp)
            .map(|c| std::mem::replace(&mut c.background, self.color))
            .unwrap_or(Color::TRANSPARENT);
        Box::new(SetBackground { comp: self.comp, color: old })
    }
    fn label(&self) -> String {
        "Set background".into()
    }
}

/// Move a layer within the draw order from index `from` to index `to`.
pub struct ReorderLayer {
    pub comp: CompId,
    pub from: usize,
    pub to: usize,
}
impl Command for ReorderLayer {
    fn apply(&self, project: &mut Project) -> Box<dyn Command> {
        if let Some(comp) = project.composition_mut(self.comp) {
            let n = comp.order.len();
            if self.from < n && self.to < n {
                let id = comp.order.remove(self.from);
                comp.order.insert(self.to, id);
            }
        }
        // Inverse swaps the endpoints.
        Box::new(ReorderLayer { comp: self.comp, from: self.to, to: self.from })
    }
    fn label(&self) -> String {
        "Reorder layer".into()
    }
}

/// Add a layer to the top of a composition's stack. Inverse removes it.
pub struct AddLayer {
    pub comp: CompId,
    pub layer: Layer,
}
impl Command for AddLayer {
    fn apply(&self, project: &mut Project) -> Box<dyn Command> {
        let id = project
            .composition_mut(self.comp)
            .map(|c| c.add_layer(self.layer.clone()));
        // Inverse removes the just-added layer (top of stack).
        Box::new(RemoveLayer { comp: self.comp, layer: id.unwrap_or_default() })
    }
    fn label(&self) -> String {
        format!("Add layer \"{}\"", self.layer.name)
    }
}

/// Remove a layer. Inverse re-inserts its data at its original stacking index.
///
/// NOTE: the re-inserted layer receives a fresh slotmap id (slotmap can't
/// reuse a removed key). References to the old id (parenting/selection) are not
/// restored — a documented MVP limitation; the layer's data and position are.
pub struct RemoveLayer {
    pub comp: CompId,
    pub layer: LayerId,
}
impl Command for RemoveLayer {
    fn apply(&self, project: &mut Project) -> Box<dyn Command> {
        let mut captured: Option<(Layer, usize)> = None;
        if let Some(comp) = project.composition_mut(self.comp) {
            if let Some(layer) = comp.layers.remove(self.layer) {
                let idx = comp.order.iter().position(|&id| id == self.layer).unwrap_or(comp.order.len());
                if idx < comp.order.len() {
                    comp.order.remove(idx);
                }
                captured = Some((layer, idx));
            }
        }
        match captured {
            Some((layer, index)) => Box::new(InsertLayerAt { comp: self.comp, layer, index }),
            None => Box::new(NoOp),
        }
    }
    fn label(&self) -> String {
        "Delete layer".into()
    }
}

/// Re-insert a captured layer at a specific stacking index (undo of remove).
struct InsertLayerAt {
    comp: CompId,
    layer: Layer,
    index: usize,
}
impl Command for InsertLayerAt {
    fn apply(&self, project: &mut Project) -> Box<dyn Command> {
        let mut new_id = LayerId::default();
        if let Some(comp) = project.composition_mut(self.comp) {
            new_id = comp.layers.insert(self.layer.clone());
            // add_layer pushed to the end; relocate to the captured index.
            if let Some(pos) = comp.order.iter().position(|&id| id == new_id) {
                let id = comp.order.remove(pos);
                let idx = self.index.min(comp.order.len());
                comp.order.insert(idx, id);
            } else {
                let idx = self.index.min(comp.order.len());
                comp.order.insert(idx, new_id);
            }
        }
        Box::new(RemoveLayer { comp: self.comp, layer: new_id })
    }
    fn label(&self) -> String {
        "Restore layer".into()
    }
}

/// A command that does nothing (used when a target no longer exists).
struct NoOp;
impl Command for NoOp {
    fn apply(&self, _project: &mut Project) -> Box<dyn Command> {
        Box::new(NoOp)
    }
    fn label(&self) -> String {
        "No-op".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use creator_model::{Composition, LayerKind, Shape};

    fn doc_with_layer() -> (Document, CompId, LayerId) {
        let mut project = Project::new("t");
        let mut comp = Composition::new("main", 100, 100, 30.0, 1.0);
        let lid = comp.add_layer(Layer::new("box", LayerKind::Shape(Shape::rect(Vec2::new(10.0, 10.0), Color::WHITE))));
        let cid = project.add_composition(comp);
        (Document::new(project), cid, lid)
    }

    #[test]
    fn rename_undo_redo_round_trips() {
        let (mut doc, comp, layer) = doc_with_layer();
        let before = doc.project.composition(comp).unwrap().layer(layer).unwrap().name.clone();

        doc.execute(Box::new(SetLayerName { comp, layer, name: "renamed".into() }));
        assert_eq!(doc.project.composition(comp).unwrap().layer(layer).unwrap().name, "renamed");

        doc.undo();
        assert_eq!(doc.project.composition(comp).unwrap().layer(layer).unwrap().name, before);

        doc.redo();
        assert_eq!(doc.project.composition(comp).unwrap().layer(layer).unwrap().name, "renamed");
    }

    #[test]
    fn execute_clears_redo() {
        let (mut doc, comp, layer) = doc_with_layer();
        doc.execute(Box::new(SetLayerName { comp, layer, name: "a".into() }));
        doc.undo();
        assert!(doc.can_redo());
        doc.execute(Box::new(SetLayerName { comp, layer, name: "b".into() }));
        assert!(!doc.can_redo());
    }

    #[test]
    fn opacity_property_round_trips() {
        let (mut doc, comp, layer) = doc_with_layer();
        let animated = Property::animated(vec![
            creator_model::Keyframe::new(0.0, 1.0_f64),
            creator_model::Keyframe::new(1.0, 0.0),
        ]);
        doc.execute(Box::new(SetOpacity { comp, layer, opacity: animated.clone() }));
        assert!(doc.project.composition(comp).unwrap().layer(layer).unwrap().opacity.is_animated());
        doc.undo();
        assert!(!doc.project.composition(comp).unwrap().layer(layer).unwrap().opacity.is_animated());
    }

    #[test]
    fn reorder_round_trips() {
        let (mut doc, comp, _) = doc_with_layer();
        // add a second layer so there are two to reorder.
        doc.execute(Box::new(AddLayer {
            comp,
            layer: Layer::new("second", LayerKind::Null),
        }));
        let order_before = doc.project.composition(comp).unwrap().order.clone();
        doc.execute(Box::new(ReorderLayer { comp, from: 0, to: 1 }));
        assert_ne!(doc.project.composition(comp).unwrap().order, order_before);
        doc.undo();
        assert_eq!(doc.project.composition(comp).unwrap().order, order_before);
    }

    #[test]
    fn add_then_undo_removes_layer() {
        let (mut doc, comp, _) = doc_with_layer();
        let count_before = doc.project.composition(comp).unwrap().layers.len();
        doc.execute(Box::new(AddLayer { comp, layer: Layer::new("extra", LayerKind::Null) }));
        assert_eq!(doc.project.composition(comp).unwrap().layers.len(), count_before + 1);
        doc.undo();
        assert_eq!(doc.project.composition(comp).unwrap().layers.len(), count_before);
    }

    #[test]
    fn remove_then_undo_restores_data_and_position() {
        let (mut doc, comp, _) = doc_with_layer();
        doc.execute(Box::new(AddLayer { comp, layer: Layer::new("top", LayerKind::Null) }));
        let order = doc.project.composition(comp).unwrap().order.clone();
        let bottom = order[0];
        let bottom_name = doc.project.composition(comp).unwrap().layer(bottom).unwrap().name.clone();

        doc.execute(Box::new(RemoveLayer { comp, layer: bottom }));
        assert_eq!(doc.project.composition(comp).unwrap().layers.len(), 1);

        doc.undo();
        let comp_ref = doc.project.composition(comp).unwrap();
        assert_eq!(comp_ref.layers.len(), 2);
        // restored at index 0 with the same data (fresh id).
        let restored = comp_ref.order[0];
        assert_eq!(comp_ref.layer(restored).unwrap().name, bottom_name);
    }
}
