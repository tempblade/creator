//! `creator-ofx-host` — OpenFX host (PLAN.md §8). **Deferred to Phase 5.**
//!
//! Reality check from the ecosystem: there is *no* maintained OFX **host** crate
//! in Rust. `openfx-sys` gives raw bindgen bindings to the OpenFX C headers
//! (suites + constants); the host glue is written here. OFX is a C ABI — structs
//! of function pointers (suites) plus header constants — so a host must:
//!
//! * implement the **suites** (PropertySuite, ImageEffectSuite, MemorySuite,
//!   MultiThreadSuite, MessageSuite first; ParameterSuite, InteractSuite,
//!   DrawSuite later),
//! * drive the **plugin lifecycle** (discover/load `.ofx` bundles, `OfxSetHost`,
//!   enumerate plugins, describe / createInstance / render / destroy),
//! * **map params** OFX ↔ the typed property system in `creator-model`,
//! * manage **image/clip memory** (CPU RGBA float/byte buffers in render
//!   windows; wire the MultiThread suite to `rayon`).
//!
//! This module fixes the *host-side data model and lifecycle states* so the rest
//! of the engine can compile against a stable interface; the actual suite
//! function pointers are filled in on top of `openfx-sys` in Phase 5 (behind an
//! `ofx-sys` feature, since it needs bindgen + the OpenFX C headers).

use creator_render::CpuTarget;

/// The OFX suites a host must provide, in the order this project implements them
/// (PLAN.md §8). Tracking them as data lets the host advertise capability and
/// lets tests assert the MVP set is present.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suite {
    Property,
    ImageEffect,
    Memory,
    MultiThread,
    Message,
    // --- later ---
    Parameter,
    Interact,
    Draw,
}

impl Suite {
    /// The minimum suites required to load and render a basic OFX plugin.
    pub const MVP: [Suite; 5] = [
        Suite::Property,
        Suite::ImageEffect,
        Suite::Memory,
        Suite::MultiThread,
        Suite::Message,
    ];
}

/// Lifecycle of a loaded plugin instance (discover → describe → instance →
/// render → destroy).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    Discovered,
    Described,
    Instanced,
    Rendering,
    Destroyed,
}

/// Which implementation route to take for the suites (PLAN.md §8 "two routes").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostRoute {
    /// Reimplement the suites in Rust on top of `openfx-sys` (clean, more work).
    NativeRust,
    /// Wrap the C++ `HostSupport` reference library via FFI (less from-scratch,
    /// adds a C++ build dependency).
    WrapHostSupport,
}

/// A discovered OFX plugin bundle (metadata only until Phase 5 loads it).
#[derive(Debug, Clone)]
pub struct PluginBundle {
    pub identifier: String,
    pub path: std::path::PathBuf,
    pub state: PluginState,
}

/// The host. Holds the chosen route and the suites it advertises.
pub struct OfxHost {
    pub route: HostRoute,
    pub suites: Vec<Suite>,
}

impl OfxHost {
    /// A host advertising the MVP suite set.
    pub fn new(route: HostRoute) -> Self {
        OfxHost { route, suites: Suite::MVP.to_vec() }
    }

    /// Whether a given suite is advertised by this host.
    pub fn provides(&self, suite: Suite) -> bool {
        self.suites.contains(&suite)
    }

    /// Discover `.ofx` bundles on standard search paths. **Stub:** Phase 5 walks
    /// `OFX_PLUGIN_PATH` and the platform default dirs.
    pub fn discover(&self) -> Vec<PluginBundle> {
        Vec::new()
    }

    /// Apply a plugin's render over a CPU clip. **Stub:** Phase 5 marshals the
    /// buffer into an OFX clip (RGBA float, render window), calls the plugin's
    /// `render` action, and reads the result back. OFX's baseline is CPU
    /// buffers, so GPU effects incur readback↔upload roundtrips — cache
    /// aggressively (PLAN.md §8).
    pub fn render_effect(&self, _bundle: &PluginBundle, input: &CpuTarget) -> CpuTarget {
        input.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mvp_suites_present() {
        let host = OfxHost::new(HostRoute::NativeRust);
        for s in Suite::MVP {
            assert!(host.provides(s));
        }
        assert!(!host.provides(Suite::Draw));
    }

    #[test]
    fn discover_is_empty_until_phase5() {
        let host = OfxHost::new(HostRoute::WrapHostSupport);
        assert!(host.discover().is_empty());
    }
}
