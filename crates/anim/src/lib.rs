//! `creator-anim` — pure, deterministic interpolation primitives.
//!
//! Everything here is a **pure function of time**: evaluating a property at time
//! `t` never depends on previously evaluated times. This is what makes the
//! timeline randomly scrubbable and the frame cache sound (see PLAN.md §5, §11).
//!
//! Two ingredients:
//! * [`Easing`] — `Hold` / `Linear` / `CubicBezier` (Newton-Raphson) / `Steps`
//!   plus named presets. Maps a normalized phase `u ∈ [0,1]` to eased progress.
//! * [`Spring`] — a damped harmonic oscillator with a **closed-form** solution
//!   for the under-, critically-, and over-damped regimes. Closed-form is
//!   non-negotiable: evaluation at an arbitrary time is `O(1)` and does not
//!   require integrating from the segment start.
//!
//! [`Interp`] unifies them as the per-segment interpolation stored on keyframes,
//! and [`Lerp`] blends concrete values by the resulting progress.

mod easing;
mod interp;
mod spring;

pub use easing::{Easing, StepPosition};
pub use interp::{Interp, Lerp};
pub use spring::{Spring, SpringParams};
