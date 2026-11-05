//! "Nice" tick generation using the 1-2-5 rule.

/// Generated ticks for one axis.
#[derive(Debug, Clone, PartialEq)]
pub struct Ticks {
    /// Distance between consecutive ticks.
    pub step: f64,
    /// Tick values in ascending order, all inside the requested interval.
    pub values: Vec<f64>,
}

/// Picks a step from {1, 2, 5} x 10^k so that `span` splits into roughly
/// `target` intervals (never more).
pub fn nice_step(span: f64, target: usize) -> f64 {
    let span = span.abs();
    if !span.is_finite() || span == 0.0 {
        return 1.0;
    }
    let raw = span / target.max(1) as f64;
    let magnitude = 10f64.powf(raw.log10().floor());
    let norm = raw / magnitude;
    let nice = if norm <= 1.0 {
        1.0
    } else if norm <= 2.0 {
        2.0
    } else if norm <= 5.0 {
        5.0
    } else {
        10.0
    };
    nice * magnitude
}

/// Ticks within `[min, max]`, aiming for about `target` intervals.
pub fn ticks(min: f64, max: f64, target: usize) -> Ticks {
    let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
    if !lo.is_finite() || !hi.is_finite() {
        return Ticks {
            step: 1.0,
            values: Vec::new(),
        };
    }
    if lo == hi {
        return Ticks {
            step: 1.0,
            values: vec![lo],
        };
    }

    let step = nice_step(hi - lo, target);
    // Tolerance absorbs floating-point noise at the interval ends.
    let eps = step * 1e-9;
    let first = (lo / step - 1e-9).ceil() as i64;
    let last = (hi / step + 1e-9).floor() as i64;

    let values = (first..=last)
        .map(|i| snap(i as f64 * step, step))
        .filter(|v| *v >= lo - eps && *v <= hi + eps)
        .collect();

    Ticks { step, values }
}

/// Expands `[min, max]` outwards to multiples of a nice step.
///
/// Returns the expanded domain. Degenerate input is widened first so the
/// result always has a positive span.
pub fn nice_domain(min: f64, max: f64, target: usize) -> (f64, f64) {
    let (mut lo, mut hi) = if min <= max { (min, max) } else { (max, min) };
    if lo == hi {
        let pad = if lo == 0.0 { 1.0 } else { lo.abs() * 0.1 };
        lo -= pad;
        hi += pad;
    }
    // Expanding the domain changes its span, which can change the step that
    // `ticks` would pick. Iterate until the step is stable so the domain ends
    // exactly on ticks.
    let mut step = nice_step(hi - lo, target);
    let mut domain = (lo, hi);
    for _ in 0..8 {
        domain = (
            snap((lo / step + 1e-9).floor() * step, step),
            snap((hi / step - 1e-9).ceil() * step, step),
        );
        let next = nice_step(domain.1 - domain.0, target);
        if next == step {
            break;
        }
        step = next;
    }
    domain
}

/// Number of decimal places needed to print values that are multiples of `step`.
pub fn decimals_for_step(step: f64) -> usize {
    if !step.is_finite() || step <= 0.0 {
        return 0;
    }
    // Smallest d such that step * 10^d is (nearly) an integer.
    (0..=10)
        .find(|d| {
            let scaled = step * 10f64.powi(*d as i32);
            (scaled - scaled.round()).abs() < 1e-6 * scaled.max(1.0)
        })
        .unwrap_or(10)
}

/// Removes floating-point noise such as `0.30000000000000004`.
fn snap(value: f64, step: f64) -> f64 {
    let decimals = decimals_for_step(step) as i32 + 2;
    let factor = 10f64.powi(decimals);
    let v = (value * factor).round() / factor;
    if v == 0.0 { 0.0 } else { v } // normalize -0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nice_steps_follow_1_2_5() {
        assert_eq!(nice_step(100.0, 5), 20.0);
        assert_eq!(nice_step(100.0, 10), 10.0);
        assert_eq!(nice_step(1.0, 4), 0.5);
        assert_eq!(nice_step(7.0, 5), 2.0);
        assert_eq!(nice_step(0.0, 5), 1.0);
    }

    #[test]
    fn ticks_inside_interval() {
        let t = ticks(0.0, 100.0, 5);
        assert_eq!(t.values, vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);

        let t = ticks(-3.0, 7.0, 5);
        assert_eq!(t.step, 2.0);
        assert_eq!(t.values, vec![-2.0, 0.0, 2.0, 4.0, 6.0]);
    }

    #[test]
    fn ticks_have_no_float_noise() {
        let t = ticks(0.0, 1.0, 10);
        assert_eq!(t.values[3], 0.3);
        assert!(t.values.iter().all(|v| v.to_string().len() <= 3));
    }

    #[test]
    fn degenerate_and_reversed_intervals() {
        assert_eq!(ticks(5.0, 5.0, 5).values, vec![5.0]);
        assert_eq!(ticks(10.0, 0.0, 2).values, vec![0.0, 5.0, 10.0]);
        assert!(ticks(f64::NAN, 1.0, 5).values.is_empty());
    }

    #[test]
    fn nice_domain_expands() {
        assert_eq!(nice_domain(3.0, 97.0, 5), (0.0, 100.0));
        assert_eq!(nice_domain(-13.0, 42.0, 5), (-20.0, 60.0));
        let (lo, hi) = nice_domain(5.0, 5.0, 5);
        assert!(lo < 5.0 && hi > 5.0);
    }

    #[test]
    fn nice_domain_ends_on_ticks() {
        for (lo, hi, target) in [
            (950.0, 2480.0, 8),
            (3.0, 97.0, 5),
            (-0.37, 12.9, 7),
            (0.001, 0.0173, 4),
        ] {
            let d = nice_domain(lo, hi, target);
            let t = ticks(d.0, d.1, target);
            assert_eq!(t.values.first().copied(), Some(d.0), "{lo}..{hi}");
            assert_eq!(t.values.last().copied(), Some(d.1), "{lo}..{hi}");
        }
    }

    #[test]
    fn decimals() {
        assert_eq!(decimals_for_step(10.0), 0);
        assert_eq!(decimals_for_step(1.0), 0);
        assert_eq!(decimals_for_step(0.5), 1);
        assert_eq!(decimals_for_step(0.2), 1);
        assert_eq!(decimals_for_step(0.05), 2);
        assert_eq!(decimals_for_step(2.5), 1);
        assert_eq!(decimals_for_step(0.1 + 0.2), 1);
    }
}
