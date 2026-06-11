//! Frame cache / RAM preview (PLAN.md §11).
//!
//! Caches **rendered frames** (not just interpolation results — the real fix for
//! v1's playback pain) keyed by `(composition, frame, content-hash)`. The
//! content hash scopes invalidation to the composition that actually changed:
//! editing comp A leaves comp B's cached frames valid. Eviction is FIFO up to a
//! capacity.

use creator_model::{CompId, LayerKind, Project};
use creator_render::CpuTarget;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Identity of a cached frame.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FrameKey {
    pub comp: CompId,
    pub frame: u64,
    /// Hash of the composition's content at render time.
    pub content_hash: u64,
}

/// A bounded FIFO frame cache.
pub struct FrameCache {
    capacity: usize,
    map: HashMap<FrameKey, Arc<CpuTarget>>,
    order: VecDeque<FrameKey>,
    hits: u64,
    misses: u64,
}

impl FrameCache {
    pub fn new(capacity: usize) -> Self {
        FrameCache {
            capacity: capacity.max(1),
            map: HashMap::new(),
            order: VecDeque::new(),
            hits: 0,
            misses: 0,
        }
    }

    /// Hash a composition's content **transitively**. A rendered frame of `comp`
    /// embeds the rendered content of every precomp it references (eval flattens
    /// `Precomp(nested)` into the parent tree), so the hash must fold in those
    /// referenced comps — otherwise editing only a child comp would leave a
    /// parent's cache key unchanged and serve stale frames. Cycles terminate via
    /// the visited set.
    pub fn content_hash(project: &Project, comp: CompId) -> u64 {
        let mut h = DefaultHasher::new();
        let mut visited = HashSet::new();
        hash_comp_transitive(project, comp, &mut h, &mut visited);
        h.finish()
    }

    /// Fetch a cached frame if present.
    pub fn get(&mut self, key: &FrameKey) -> Option<Arc<CpuTarget>> {
        if let Some(frame) = self.map.get(key) {
            self.hits += 1;
            Some(frame.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    /// Get a cached frame or render it via `f`, inserting the result.
    pub fn get_or_render(
        &mut self,
        key: FrameKey,
        f: impl FnOnce() -> CpuTarget,
    ) -> Arc<CpuTarget> {
        if let Some(frame) = self.get(&key) {
            return frame;
        }
        let frame = Arc::new(f());
        self.insert(key, frame.clone());
        frame
    }

    /// Insert (or replace) a frame, evicting the oldest if over capacity.
    pub fn insert(&mut self, key: FrameKey, frame: Arc<CpuTarget>) {
        if self.map.insert(key.clone(), frame).is_none() {
            self.order.push_back(key);
            while self.order.len() > self.capacity {
                if let Some(old) = self.order.pop_front() {
                    self.map.remove(&old);
                }
            }
        }
    }

    /// Drop every cached frame for a composition (e.g. on structural change).
    pub fn invalidate_comp(&mut self, comp: CompId) {
        self.map.retain(|k, _| k.comp != comp);
        self.order.retain(|k| k.comp != comp);
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    pub fn hits(&self) -> u64 {
        self.hits
    }
    pub fn misses(&self) -> u64 {
        self.misses
    }
}

/// Fold a comp's serialized content and that of every precomp it references
/// (transitively) into `h`. `visited` prevents re-hashing and breaks cycles.
fn hash_comp_transitive(
    project: &Project,
    comp: CompId,
    h: &mut DefaultHasher,
    visited: &mut HashSet<CompId>,
) {
    if !visited.insert(comp) {
        return;
    }
    if let Some(c) = project.composition(comp) {
        if let Ok(bytes) = serde_json::to_vec(c) {
            bytes.hash(h);
        }
        for &id in &c.order {
            if let Some(layer) = c.layer(id) {
                if let LayerKind::Precomp(nested) = layer.kind {
                    hash_comp_transitive(project, nested, h, visited);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use creator_model::{Color, Composition, Layer, LayerKind, Project, Shape, Vec2};

    fn project() -> (Project, CompId) {
        let mut p = Project::new("t");
        let mut c = Composition::new("m", 10, 10, 30.0, 1.0);
        c.add_layer(Layer::new("box", LayerKind::Shape(Shape::rect(Vec2::new(4.0, 4.0), Color::WHITE))));
        let id = p.add_composition(c);
        (p, id)
    }

    #[test]
    fn hit_then_miss_on_edit() {
        let (mut p, comp) = project();
        let mut cache = FrameCache::new(8);
        let key = FrameKey { comp, frame: 0, content_hash: FrameCache::content_hash(&p, comp) };

        let mut rendered = false;
        let _ = cache.get_or_render(key.clone(), || {
            rendered = true;
            CpuTarget::new(10, 10)
        });
        assert!(rendered, "first call renders");

        // Same key -> hit, no re-render.
        let mut rendered2 = false;
        let _ = cache.get_or_render(key.clone(), || {
            rendered2 = true;
            CpuTarget::new(10, 10)
        });
        assert!(!rendered2, "second call hits cache");
        assert_eq!(cache.hits(), 1);

        // Edit the comp -> content hash changes -> new key misses.
        p.composition_mut(comp).unwrap().background = Color::WHITE;
        let key2 = FrameKey { comp, frame: 0, content_hash: FrameCache::content_hash(&p, comp) };
        assert_ne!(key.content_hash, key2.content_hash);
        assert!(cache.get(&key2).is_none());
    }

    #[test]
    fn fifo_eviction() {
        let (p, comp) = project();
        let h = FrameCache::content_hash(&p, comp);
        let mut cache = FrameCache::new(2);
        for frame in 0..3 {
            cache.insert(
                FrameKey { comp, frame, content_hash: h },
                Arc::new(CpuTarget::new(10, 10)),
            );
        }
        assert_eq!(cache.len(), 2);
        // frame 0 evicted.
        assert!(cache.get(&FrameKey { comp, frame: 0, content_hash: h }).is_none());
        assert!(cache.get(&FrameKey { comp, frame: 2, content_hash: h }).is_some());
    }

    #[test]
    fn content_hash_is_transitive_over_precomps() {
        // Parent comp references a child as a precomp. Editing ONLY the child
        // must change the PARENT's content hash (its rendered frame embeds the
        // child), so the parent's cached frames correctly invalidate.
        let mut project = Project::new("t");
        let mut child = Composition::new("child", 20, 20, 30.0, 1.0);
        child.add_layer(Layer::new("c", LayerKind::Shape(Shape::rect(Vec2::new(8.0, 8.0), Color::WHITE))));
        let child_id = project.add_composition(child);

        let mut parent = Composition::new("parent", 40, 40, 30.0, 1.0);
        parent.add_layer(Layer::new("nested", LayerKind::Precomp(child_id)));
        let parent_id = project.add_composition(parent);

        let before = FrameCache::content_hash(&project, parent_id);
        // Edit only the child.
        project.composition_mut(child_id).unwrap().background = Color::WHITE;
        let after = FrameCache::content_hash(&project, parent_id);
        assert_ne!(before, after, "editing a precomp must change the parent's hash");
    }

    #[test]
    fn content_hash_terminates_on_precomp_cycle() {
        // A comp referencing itself must not infinitely recurse the hash.
        let mut project = Project::new("t");
        let c = Composition::new("c", 10, 10, 30.0, 1.0);
        let id = project.add_composition(c);
        project.composition_mut(id).unwrap().add_layer(Layer::new("self", LayerKind::Precomp(id)));
        let _ = FrameCache::content_hash(&project, id); // returns, no hang
    }

    #[test]
    fn invalidate_comp_clears_frames() {
        let (p, comp) = project();
        let h = FrameCache::content_hash(&p, comp);
        let mut cache = FrameCache::new(8);
        cache.insert(FrameKey { comp, frame: 0, content_hash: h }, Arc::new(CpuTarget::new(10, 10)));
        cache.invalidate_comp(comp);
        assert!(cache.is_empty());
    }
}
