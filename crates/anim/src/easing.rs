//! Easing functions: map a normalized phase `u ∈ [0,1]` to eased progress.
//!
//! `CubicBezier` solves for the curve parameter `s` such that `X(s) == u`
//! (Newton-Raphson with a bisection fallback), then returns `Y(s)` — the exact
//! technique browsers use for CSS `cubic-bezier()`.

use serde::{Deserialize, Serialize};

/// Where a [`Easing::Steps`] curve takes its jumps (CSS `step-start`/`step-end`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepPosition {
    /// Jump at the start of each interval (CSS `start` / `jump-start`).
    Start,
    /// Jump at the end of each interval (CSS `end` / `jump-end`). Default.
    End,
}

/// A scalar easing curve mapping phase `u ∈ [0,1]` to progress.
///
/// Most curves stay in `[0,1]`, but `CubicBezier` may overshoot (e.g. "back"
/// easings) and that is intentional — callers blend with [`crate::Lerp`], which
/// extrapolates.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum Easing {
    /// Step function: hold the start value, jump to the end at `u == 1`.
    Hold,
    /// Identity: `progress == u`.
    #[default]
    Linear,
    /// Cubic Bézier with implicit endpoints `(0,0)` and `(1,1)` and the two
    /// control points `(x1,y1)`, `(x2,y2)`. `x` components are clamped to
    /// `[0,1]` (a function of x must be monotonic), `y` may exceed it.
    CubicBezier { x1: f64, y1: f64, x2: f64, y2: f64 },
    /// `n` discrete steps; jump position selects start vs end.
    Steps { count: u32, position: StepPosition },
}

impl Easing {
    /// CSS `ease` — the default web timing function.
    pub fn ease() -> Self {
        Easing::CubicBezier { x1: 0.25, y1: 0.1, x2: 0.25, y2: 1.0 }
    }
    /// CSS `ease-in`.
    pub fn ease_in() -> Self {
        Easing::CubicBezier { x1: 0.42, y1: 0.0, x2: 1.0, y2: 1.0 }
    }
    /// CSS `ease-out`.
    pub fn ease_out() -> Self {
        Easing::CubicBezier { x1: 0.0, y1: 0.0, x2: 0.58, y2: 1.0 }
    }
    /// CSS `ease-in-out`.
    pub fn ease_in_out() -> Self {
        Easing::CubicBezier { x1: 0.42, y1: 0.0, x2: 0.58, y2: 1.0 }
    }

    /// Evaluate eased progress for a normalized phase. `u` is clamped to `[0,1]`.
    pub fn eval(&self, u: f64) -> f64 {
        let u = u.clamp(0.0, 1.0);
        match *self {
            Easing::Hold => {
                if u >= 1.0 {
                    1.0
                } else {
                    0.0
                }
            }
            Easing::Linear => u,
            Easing::CubicBezier { x1, y1, x2, y2 } => {
                UnitBezier::new(x1, y1, x2, y2).solve(u)
            }
            Easing::Steps { count, position } => {
                let n = count.max(1) as f64;
                let stepped = match position {
                    StepPosition::End => (u * n).floor(),
                    StepPosition::Start => (u * n).ceil(),
                };
                (stepped / n).clamp(0.0, 1.0)
            }
        }
    }
}

/// A unit cubic Bézier `(0,0) → (x1,y1) → (x2,y2) → (1,1)`, parameterized in the
/// WebKit `UnitBezier` style. Coefficients are precomputed so `sample`/`solve`
/// are cheap.
struct UnitBezier {
    // X(s) = ax*s^3 + bx*s^2 + cx*s, similarly for Y.
    ax: f64,
    bx: f64,
    cx: f64,
    ay: f64,
    by: f64,
    cy: f64,
}

impl UnitBezier {
    fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        // x must be monotonic for X(s) to be invertible; y is unconstrained.
        let x1 = x1.clamp(0.0, 1.0);
        let x2 = x2.clamp(0.0, 1.0);
        // Bézier polynomial coefficients (control point 0 at origin).
        let cx = 3.0 * x1;
        let bx = 3.0 * (x2 - x1) - cx;
        let ax = 1.0 - cx - bx;
        let cy = 3.0 * y1;
        let by = 3.0 * (y2 - y1) - cy;
        let ay = 1.0 - cy - by;
        UnitBezier { ax, bx, cx, ay, by, cy }
    }

    #[inline]
    fn sample_x(&self, s: f64) -> f64 {
        ((self.ax * s + self.bx) * s + self.cx) * s
    }
    #[inline]
    fn sample_y(&self, s: f64) -> f64 {
        ((self.ay * s + self.by) * s + self.cy) * s
    }
    #[inline]
    fn sample_dx(&self, s: f64) -> f64 {
        (3.0 * self.ax * s + 2.0 * self.bx) * s + self.cx
    }

    /// Given `x ∈ [0,1]`, find `s` with `X(s) == x`, then return `Y(s)`.
    fn solve(&self, x: f64) -> f64 {
        const EPS: f64 = 1e-7;
        if x <= 0.0 {
            return 0.0;
        }
        if x >= 1.0 {
            return 1.0;
        }
        self.sample_y(self.solve_curve_x(x, EPS))
    }

    /// Invert X(s) = x. Newton-Raphson first (fast when the derivative is sane),
    /// bisection fallback for the flat regions where Newton stalls.
    fn solve_curve_x(&self, x: f64, eps: f64) -> f64 {
        // Newton-Raphson: a good initial guess is x itself.
        let mut s = x;
        for _ in 0..8 {
            let err = self.sample_x(s) - x;
            if err.abs() < eps {
                return s;
            }
            let d = self.sample_dx(s);
            if d.abs() < 1e-6 {
                break; // derivative too small — hand off to bisection
            }
            s -= err / d;
        }

        // Bisection fallback, guaranteed to converge on the monotonic interval.
        let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
        let mut s = x;
        if s < lo {
            s = lo;
        }
        if s > hi {
            s = hi;
        }
        for _ in 0..64 {
            let xs = self.sample_x(s);
            if (xs - x).abs() < eps {
                return s;
            }
            if x > xs {
                lo = s;
            } else {
                hi = s;
            }
            s = 0.5 * (lo + hi);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_is_identity() {
        let e = Easing::Linear;
        for &u in &[0.0, 0.1, 0.5, 0.9, 1.0] {
            assert!((e.eval(u) - u).abs() < 1e-12);
        }
    }

    #[test]
    fn hold_steps_at_end() {
        let e = Easing::Hold;
        assert_eq!(e.eval(0.0), 0.0);
        assert_eq!(e.eval(0.999), 0.0);
        assert_eq!(e.eval(1.0), 1.0);
    }

    #[test]
    fn bezier_endpoints_and_monotonic() {
        let e = Easing::ease_in_out();
        assert!(e.eval(0.0).abs() < 1e-9);
        assert!((e.eval(1.0) - 1.0).abs() < 1e-9);
        // monotonic non-decreasing for a standard ease
        let mut prev = -1.0;
        for i in 0..=100 {
            let y = e.eval(i as f64 / 100.0);
            assert!(y >= prev - 1e-9, "non-monotonic at {i}: {y} < {prev}");
            prev = y;
        }
    }

    #[test]
    fn linear_bezier_matches_identity() {
        // cubic-bezier(0,0,1,1) is the identity line.
        let e = Easing::CubicBezier { x1: 0.0, y1: 0.0, x2: 1.0, y2: 1.0 };
        for i in 0..=20 {
            let u = i as f64 / 20.0;
            assert!((e.eval(u) - u).abs() < 1e-4, "u={u} -> {}", e.eval(u));
        }
    }

    #[test]
    fn bezier_solver_inverts_x_exactly() {
        // For an arbitrary curve, Y(solve(X(s))) must round-trip to Y(s).
        let b = UnitBezier::new(0.42, 0.0, 0.58, 1.0);
        for i in 1..20 {
            let s = i as f64 / 20.0;
            let x = b.sample_x(s);
            let recovered = b.sample_y(b.solve_curve_x(x, 1e-9));
            assert!((recovered - b.sample_y(s)).abs() < 1e-5);
        }
    }

    #[test]
    fn steps_end() {
        let e = Easing::Steps { count: 4, position: StepPosition::End };
        assert_eq!(e.eval(0.0), 0.0);
        assert_eq!(e.eval(0.2), 0.0);
        assert_eq!(e.eval(0.25), 0.25);
        assert_eq!(e.eval(0.99), 0.75);
        assert_eq!(e.eval(1.0), 1.0);
    }

    #[test]
    fn steps_start() {
        let e = Easing::Steps { count: 4, position: StepPosition::Start };
        assert_eq!(e.eval(0.0), 0.0);
        assert_eq!(e.eval(0.01), 0.25);
        assert_eq!(e.eval(0.25), 0.25);
        assert_eq!(e.eval(1.0), 1.0);
    }
}
