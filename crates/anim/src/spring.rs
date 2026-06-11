//! Closed-form damped harmonic oscillator springs.
//!
//! We model the **normalized displacement from the target**, `d(t)`, with
//! `d(0) = 1` (fully displaced) and `d(∞) → 0` (settled). Progress toward the
//! target is `p(t) = 1 − d(t)`, so `p(0) = 0` and `p(∞) → 1`.
//!
//! The governing ODE is `x'' + 2ζω₀ x' + ω₀² x = 0`, with natural angular
//! frequency `ω₀` and damping ratio `ζ`. The solution has three regimes:
//!
//! * **Underdamped** (`ζ < 1`): oscillates while decaying (overshoot/bounce).
//! * **Critically damped** (`ζ = 1`): fastest settle without oscillation.
//! * **Overdamped** (`ζ > 1`): two real decay modes, no oscillation.
//!
//! Each is closed-form, so `progress(t)` is `O(1)` and deterministic — you can
//! jump to any time without integrating from the start. An optional entry
//! velocity `v₀` (in progress-units per second) seeds `p'(0) = v₀`, i.e.
//! `d'(0) = −v₀`, used when chaining spring segments.

use serde::{Deserialize, Serialize};

/// `|ζ − 1|` below this counts as critically damped, avoiding a divide-by-zero
/// as `ωd → 0` (underdamped) or `s → 0` (overdamped) near the boundary.
const CRITICAL_EPS: f64 = 1e-4;

/// User-facing spring parameterization. All three map to the same internal
/// `(ω₀, ζ)`; we store the user's intent so projects round-trip faithfully.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SpringParams {
    /// Physical: `mass`, `stiffness` (k), `damping` (c). The classic model.
    Physical { mass: f64, stiffness: f64, damping: f64 },
    /// Perceptual (SwiftUI `Spring(response:dampingFraction:)`): `response` is
    /// the approximate settling period in seconds, `damping_fraction` is ζ.
    Perceptual { response: f64, damping_fraction: f64 },
    /// Perceptual (SwiftUI `Spring(duration:bounce:)`): `bounce ∈ [-1, 1]`,
    /// where `0` is critical, positive bounces (underdamped), negative is
    /// sluggish (overdamped).
    Bouncy { duration: f64, bounce: f64 },
}

impl SpringParams {
    /// Resolve to `(ω₀, ζ)`, both guaranteed finite and non-negative.
    fn omega0_zeta(&self) -> (f64, f64) {
        match *self {
            SpringParams::Physical { mass, stiffness, damping } => {
                let m = mass.max(1e-9);
                let k = stiffness.max(0.0);
                let omega0 = (k / m).sqrt();
                // ζ = c / (2√(k·m)); guard the denominator.
                let denom = 2.0 * (k * m).sqrt();
                let zeta = if denom > 1e-12 { (damping / denom).max(0.0) } else { 0.0 };
                (omega0, zeta)
            }
            SpringParams::Perceptual { response, damping_fraction } => {
                let r = response.max(1e-6);
                let omega0 = std::f64::consts::TAU / r;
                (omega0, damping_fraction.max(0.0))
            }
            SpringParams::Bouncy { duration, bounce } => {
                let d = duration.max(1e-6);
                let omega0 = std::f64::consts::TAU / d;
                // SwiftUI mapping: bounce >= 0 -> ζ = 1 - bounce;
                //                  bounce <  0 -> ζ = 1 / (1 + bounce).
                let b = bounce.clamp(-0.999, 1.0);
                let zeta = if b >= 0.0 { 1.0 - b } else { 1.0 / (1.0 + b) };
                (omega0, zeta.max(0.0))
            }
        }
    }
}

/// A spring segment: animates a value from its start toward its target with a
/// closed-form damped oscillation. `velocity` is the entry velocity in
/// progress-units (0..1) per second.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Spring {
    pub params: SpringParams,
    #[serde(default)]
    pub velocity: f64,
}

impl Spring {
    /// Physical spring with zero entry velocity.
    pub fn physical(mass: f64, stiffness: f64, damping: f64) -> Self {
        Spring { params: SpringParams::Physical { mass, stiffness, damping }, velocity: 0.0 }
    }
    /// Perceptual `response` / `damping_fraction` spring (SwiftUI mental model).
    pub fn perceptual(response: f64, damping_fraction: f64) -> Self {
        Spring {
            params: SpringParams::Perceptual { response, damping_fraction },
            velocity: 0.0,
        }
    }
    /// Perceptual `duration` / `bounce` spring (SwiftUI mental model).
    pub fn bouncy(duration: f64, bounce: f64) -> Self {
        Spring { params: SpringParams::Bouncy { duration, bounce }, velocity: 0.0 }
    }

    /// Set the entry velocity (progress-units per second) for chained segments.
    pub fn with_velocity(mut self, velocity: f64) -> Self {
        self.velocity = velocity;
        self
    }

    /// Resolved natural angular frequency `ω₀` and damping ratio `ζ`.
    pub fn omega0_zeta(&self) -> (f64, f64) {
        self.params.omega0_zeta()
    }

    /// Normalized progress `p(t) = 1 − d(t)` at `t` seconds since segment start.
    /// `p(0) = 0`, `p(∞) → 1`. `t < 0` clamps to 0.
    pub fn progress(&self, t: f64) -> f64 {
        if t <= 0.0 {
            return 0.0;
        }
        let (omega0, zeta) = self.omega0_zeta();
        if omega0 <= 0.0 {
            // No restoring force: never moves.
            return 0.0;
        }
        let v0 = self.velocity;

        let d = if (zeta - 1.0).abs() <= CRITICAL_EPS {
            // Critically damped: d(t) = e^{-ω₀ t}(1 + (ω₀ − v₀) t).
            let e = (-omega0 * t).exp();
            e * (1.0 + (omega0 - v0) * t)
        } else if zeta < 1.0 {
            // Underdamped: d(t) = e^{-ζω₀ t}[cos(ωd t) + ((ζω₀ − v₀)/ωd) sin(ωd t)].
            let omega_d = omega0 * (1.0 - zeta * zeta).sqrt();
            let decay = (-zeta * omega0 * t).exp();
            let b = (zeta * omega0 - v0) / omega_d;
            decay * ((omega_d * t).cos() + b * (omega_d * t).sin())
        } else {
            // Overdamped: roots r1 (slow) > r2 (fast), both negative.
            // d(t) = c1 e^{r1 t} + c2 e^{r2 t}, with d(0)=1, d'(0) = −v₀.
            let s = (zeta * zeta - 1.0).sqrt();
            let r1 = -omega0 * (zeta - s);
            let r2 = -omega0 * (zeta + s);
            // c1 + c2 = 1 ; c1 r1 + c2 r2 = −v₀.
            let c2 = (-v0 - r1) / (r2 - r1);
            let c1 = 1.0 - c2;
            c1 * (r1 * t).exp() + c2 * (r2 * t).exp()
        };

        1.0 - d
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64, eps: f64) {
        assert!((a - b).abs() < eps, "{a} != {b} (eps {eps})");
    }

    #[test]
    fn starts_at_zero_settles_at_one() {
        for spring in [
            Spring::perceptual(0.5, 0.5),  // underdamped
            Spring::perceptual(0.5, 1.0),  // critical
            Spring::perceptual(0.5, 2.0),  // overdamped
        ] {
            approx(spring.progress(0.0), 0.0, 1e-12);
            approx(spring.progress(50.0), 1.0, 1e-3);
            // monotone toward 1 in the limit and bounded.
            assert!(spring.progress(1000.0).is_finite());
        }
    }

    #[test]
    fn underdamped_overshoots() {
        // A lightly damped spring must exceed its target at least once.
        let s = Spring::perceptual(0.4, 0.2);
        let mut max = 0.0_f64;
        for i in 0..2000 {
            let t = i as f64 * 0.001;
            max = max.max(s.progress(t));
        }
        assert!(max > 1.0, "expected overshoot, got max {max}");
    }

    #[test]
    fn critical_and_overdamped_do_not_overshoot() {
        for s in [Spring::perceptual(0.4, 1.0), Spring::perceptual(0.4, 1.8)] {
            for i in 0..4000 {
                let t = i as f64 * 0.001;
                assert!(s.progress(t) <= 1.0 + 1e-6, "overshoot at t={t}: {}", s.progress(t));
            }
        }
    }

    #[test]
    fn boundary_continuity_across_regimes() {
        // progress() must be continuous as ζ sweeps through 1 (the critical band).
        let t = 0.1;
        let just_under = Spring::perceptual(0.5, 1.0 - 2e-4).progress(t);
        let critical = Spring::perceptual(0.5, 1.0).progress(t);
        let just_over = Spring::perceptual(0.5, 1.0 + 2e-4).progress(t);
        approx(just_under, critical, 1e-3);
        approx(just_over, critical, 1e-3);
    }

    #[test]
    fn entry_velocity_sets_initial_slope() {
        // p'(0) ≈ v0 ; estimate the slope with a tiny forward difference.
        let v0 = 3.0;
        let s = Spring::perceptual(0.5, 0.6).with_velocity(v0);
        let h = 1e-6;
        let slope = (s.progress(h) - s.progress(0.0)) / h;
        approx(slope, v0, 1e-2);
    }

    #[test]
    fn physical_critical_damping_detected() {
        // c = 2√(k m) is exactly critical -> ζ == 1.
        let s = Spring::physical(1.0, 100.0, 2.0 * (100.0_f64).sqrt());
        let (_, zeta) = s.omega0_zeta();
        approx(zeta, 1.0, 1e-9);
    }

    #[test]
    fn serde_round_trip() {
        let s = Spring::bouncy(0.6, 0.3).with_velocity(1.5);
        let json = serde_json::to_string(&s).unwrap();
        let back: Spring = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
