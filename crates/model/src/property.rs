//! Typed animatable properties.
//!
//! A [`Property<T>`] is either constant (no keyframes) or animated (a sorted list
//! of [`Keyframe`]s). Evaluation is a **pure function of time**: `eval(t)` never
//! depends on previously evaluated times, which is what makes scrubbing and the
//! frame cache sound (PLAN.md §5, §11).
//!
//! Keyframe segment semantics: the interpolation **leaving** a keyframe
//! (`out_interp`) governs the segment to the next keyframe. `in_interp` is
//! retained on the struct (the plan stores `{time,value,in_interp,out_interp}`)
//! for a future AE-style curve editor that combines both handles into a single
//! Bézier; the MVP evaluator uses `out_interp`.

use creator_anim::{Interp, Lerp};
use serde::{Deserialize, Serialize};

/// A single keyframe: a value pinned at a time, with incoming/outgoing
/// interpolation handles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe<T> {
    /// Time in seconds.
    pub time: f64,
    pub value: T,
    /// Incoming interpolation (reserved for the curve editor; see module docs).
    #[serde(default)]
    pub in_interp: Interp,
    /// Outgoing interpolation: governs the segment to the next keyframe.
    #[serde(default)]
    pub out_interp: Interp,
}

impl<T> Keyframe<T> {
    /// A keyframe with linear handles.
    pub fn new(time: f64, value: T) -> Self {
        Keyframe { time, value, in_interp: Interp::default(), out_interp: Interp::default() }
    }
    /// Set the outgoing interpolation (the segment to the next keyframe).
    pub fn with_out(mut self, interp: Interp) -> Self {
        self.out_interp = interp;
        self
    }
    /// Set the incoming interpolation.
    pub fn with_in(mut self, interp: Interp) -> Self {
        self.in_interp = interp;
        self
    }
}

/// A typed, animatable property. `value` is the static fallback used when there
/// are no keyframes (the AE model: a property always has a value; keyframes
/// animate it).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Property<T> {
    pub value: T,
    /// Keyframes, sorted ascending by time. `eval` relies on this ordering, so
    /// deserialization sorts (a hand-authored or out-of-order project file must
    /// not break evaluation).
    // `default = "Vec::new"` (not bare `default`) avoids serde adding a
    // `T: Default` bound to the generated impl.
    #[serde(default = "Vec::new", skip_serializing_if = "Vec::is_empty", deserialize_with = "deserialize_sorted")]
    pub keyframes: Vec<Keyframe<T>>,
}

/// Deserialize keyframes and restore the sorted-by-time invariant `eval`
/// depends on (the `pub` field and `Deserialize` derive would otherwise admit
/// unsorted input from a project file).
fn deserialize_sorted<'de, D, T>(d: D) -> Result<Vec<Keyframe<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let mut kfs = Vec::<Keyframe<T>>::deserialize(d)?;
    kfs.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
    Ok(kfs)
}

impl<T: Clone> Property<T> {
    /// A constant (non-animated) property.
    pub fn constant(value: T) -> Self {
        Property { value, keyframes: Vec::new() }
    }

    /// True if this property has keyframes.
    pub fn is_animated(&self) -> bool {
        !self.keyframes.is_empty()
    }
}

impl<T: Lerp> Property<T> {
    /// An animated property from keyframes. They are sorted by time and the
    /// static `value` is seeded from the first keyframe.
    ///
    /// Panics if `keyframes` is empty — use [`Property::constant`] for that.
    pub fn animated(mut keyframes: Vec<Keyframe<T>>) -> Self {
        assert!(!keyframes.is_empty(), "animated property needs at least one keyframe");
        keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
        let value = keyframes[0].value.clone();
        Property { value, keyframes }
    }

    /// Evaluate the property at `time` (seconds). Times before the first / after
    /// the last keyframe clamp to the endpoint value (hold).
    pub fn eval(&self, time: f64) -> T {
        let kfs = &self.keyframes;
        if kfs.is_empty() {
            return self.value.clone();
        }
        if time <= kfs[0].time {
            return kfs[0].value.clone();
        }
        let last = &kfs[kfs.len() - 1];
        if time >= last.time {
            return last.value.clone();
        }
        // `idx` = last keyframe whose time is <= `time`.
        let pp = kfs.partition_point(|k| k.time <= time);
        if pp == 0 {
            // For sorted, finite `time` this is unreachable (the clamps above
            // handle before-first). It *does* fire for NaN — where every
            // comparison is false — so this guards against a `0 - 1` usize
            // underflow / out-of-bounds panic, returning the first value.
            return kfs[0].value.clone();
        }
        let idx = pp - 1;
        let a = &kfs[idx];
        let b = &kfs[idx + 1];
        let dt = b.time - a.time;
        let u = if dt > 0.0 { (time - a.time) / dt } else { 1.0 };
        let local = time - a.time;
        a.out_interp.interpolate(&a.value, &b.value, u, local)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use creator_anim::{Easing, Interp, Spring};

    #[test]
    fn constant_is_flat() {
        let p = Property::constant(5.0_f64);
        assert_eq!(p.eval(-10.0), 5.0);
        assert_eq!(p.eval(0.0), 5.0);
        assert_eq!(p.eval(100.0), 5.0);
        assert!(!p.is_animated());
    }

    #[test]
    fn linear_segment() {
        let p = Property::animated(vec![
            Keyframe::new(0.0, 0.0_f64),
            Keyframe::new(1.0, 10.0),
        ]);
        assert_eq!(p.eval(0.0), 0.0);
        assert_eq!(p.eval(0.5), 5.0);
        assert_eq!(p.eval(1.0), 10.0);
        // clamps outside range
        assert_eq!(p.eval(-1.0), 0.0);
        assert_eq!(p.eval(2.0), 10.0);
    }

    #[test]
    fn hold_segment_uses_out_interp() {
        let p = Property::animated(vec![
            Keyframe::new(0.0, 0.0_f64).with_out(Interp::hold()),
            Keyframe::new(1.0, 10.0),
        ]);
        assert_eq!(p.eval(0.99), 0.0);
        assert_eq!(p.eval(1.0), 10.0);
    }

    #[test]
    fn unsorted_keyframes_are_sorted() {
        let p = Property::animated(vec![
            Keyframe::new(2.0, 20.0_f64),
            Keyframe::new(0.0, 0.0),
            Keyframe::new(1.0, 10.0),
        ]);
        assert_eq!(p.eval(0.5), 5.0);
        assert_eq!(p.eval(1.5), 15.0);
    }

    #[test]
    fn spring_segment_is_time_parameterized() {
        let p = Property::animated(vec![
            Keyframe::new(0.0, 0.0_f64)
                .with_out(Interp::Spring(Spring::perceptual(0.3, 0.5))),
            Keyframe::new(2.0, 100.0),
        ]);
        assert_eq!(p.eval(0.0), 0.0);
        // settles near the target well before the next keyframe.
        assert!((p.eval(2.0) - 100.0).abs() < 1.0);
    }

    #[test]
    fn deserialized_unsorted_keyframes_are_sorted() {
        // A hand-authored project file may list keyframes out of order; eval
        // must still work (it assumes ascending time).
        let json = r#"{"value":0.0,"keyframes":[
            {"time":2.0,"value":20.0},
            {"time":0.0,"value":0.0},
            {"time":1.0,"value":10.0}
        ]}"#;
        let p: Property<f64> = serde_json::from_str(json).unwrap();
        assert_eq!(p.eval(0.5), 5.0);
        assert_eq!(p.eval(1.5), 15.0);
        assert_eq!(p.eval(-1.0), 0.0); // clamps to earliest
        assert_eq!(p.eval(3.0), 20.0); // clamps to latest
    }

    #[test]
    fn nan_time_does_not_panic() {
        let p = Property::animated(vec![
            Keyframe::new(0.0, 0.0_f64),
            Keyframe::new(1.0, 10.0),
        ]);
        // NaN makes every comparison false; must return gracefully, not panic.
        let v = p.eval(f64::NAN);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn determinism_under_random_access() {
        let p = Property::animated(vec![
            Keyframe::new(0.0, 0.0_f64).with_out(Interp::Easing(Easing::ease_in_out())),
            Keyframe::new(1.0, 10.0),
        ]);
        // evaluating in any order yields identical results.
        let forward: Vec<f64> = (0..=10).map(|i| p.eval(i as f64 / 10.0)).collect();
        let backward: Vec<f64> = (0..=10).rev().map(|i| p.eval(i as f64 / 10.0)).collect();
        let mut b = backward;
        b.reverse();
        assert_eq!(forward, b);
    }
}
