//! [`Lerp`] — linear blending of concrete values — and [`Interp`], the
//! per-segment interpolation stored on keyframes.

use crate::{Easing, Spring};
use serde::{Deserialize, Serialize};

/// Linear interpolation between two values of the same type.
///
/// `t` is the blend factor; `t == 0` returns `self`, `t == 1` returns `other`.
/// Values of `t` outside `[0,1]` extrapolate — important because eased/spring
/// progress can overshoot. `Lerp` lives here (not in `model`) so the math layer
/// owns the trait; foreign-type impls (`f64`, `glam` vectors) are provided
/// below, and value types like `Color` impl it in their own crate.
pub trait Lerp: Clone {
    /// Blend toward `other` by `t`. Named `mix` (not `lerp`) so it never clashes
    /// with the inherent `lerp` that `glam` vectors already provide.
    fn mix(&self, other: &Self, t: f64) -> Self;
}

impl Lerp for f64 {
    #[inline]
    fn mix(&self, other: &Self, t: f64) -> Self {
        self + (other - self) * t
    }
}

impl Lerp for f32 {
    #[inline]
    fn mix(&self, other: &Self, t: f64) -> Self {
        self + (other - self) * t as f32
    }
}

impl Lerp for glam::Vec2 {
    #[inline]
    fn mix(&self, other: &Self, t: f64) -> Self {
        *self + (*other - *self) * t as f32
    }
}

impl Lerp for glam::Vec3 {
    #[inline]
    fn mix(&self, other: &Self, t: f64) -> Self {
        *self + (*other - *self) * t as f32
    }
}

/// Per-segment interpolation: the rule for blending from one keyframe to the
/// next. Stored on a keyframe's `out_interp` (the segment that *leaves* it).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Interp {
    /// Time-warp easing in normalized phase.
    Easing(Easing),
    /// Closed-form spring, parameterized in real seconds.
    Spring(Spring),
}

impl Default for Interp {
    fn default() -> Self {
        Interp::Easing(Easing::Linear)
    }
}

impl Interp {
    /// Convenience: a linear segment.
    pub fn linear() -> Self {
        Interp::Easing(Easing::Linear)
    }
    /// Convenience: a held (stepped) segment.
    pub fn hold() -> Self {
        Interp::Easing(Easing::Hold)
    }

    /// Map a segment to progress.
    ///
    /// * `u` — normalized phase `(t − t0) / (t1 − t0) ∈ [0,1]` (for easings).
    /// * `local_secs` — elapsed seconds since the segment start (for springs,
    ///   which are parameterized in real time, not phase).
    pub fn progress(&self, u: f64, local_secs: f64) -> f64 {
        match self {
            Interp::Easing(e) => e.eval(u),
            Interp::Spring(s) => s.progress(local_secs),
        }
    }

    /// Blend `a → b` for this segment, returning the interpolated value.
    pub fn interpolate<T: Lerp>(&self, a: &T, b: &T, u: f64, local_secs: f64) -> T {
        a.mix(b, self.progress(u, local_secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_mix() {
        assert_eq!(2.0_f64.mix(&4.0, 0.5), 3.0);
        assert_eq!(2.0_f64.mix(&4.0, 0.0), 2.0);
        assert_eq!(2.0_f64.mix(&4.0, 1.0), 4.0);
        // extrapolation
        assert_eq!(0.0_f64.mix(&10.0, 1.5), 15.0);
    }

    #[test]
    fn vec_mix() {
        let a = glam::Vec2::new(0.0, 0.0);
        let b = glam::Vec2::new(10.0, 20.0);
        let m = a.mix(&b, 0.5);
        assert_eq!(m, glam::Vec2::new(5.0, 10.0));
    }

    #[test]
    fn interp_linear_segment() {
        let i = Interp::linear();
        assert_eq!(i.interpolate(&0.0_f64, &100.0, 0.25, 0.25), 25.0);
    }

    #[test]
    fn interp_hold_segment() {
        let i = Interp::hold();
        assert_eq!(i.interpolate(&0.0_f64, &100.0, 0.5, 0.5), 0.0);
        assert_eq!(i.interpolate(&0.0_f64, &100.0, 1.0, 1.0), 100.0);
    }
}
