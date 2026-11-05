//! Resolution of the visible data domain.
//!
//! Precedence, per axis:
//!
//! 1. an explicit range (`x_range` / `y_range`),
//! 2. an origin ratio, or a quadrant selection,
//! 3. automatic: the data extent expanded to nice tick boundaries.

mod quadrant;

pub use quadrant::{Quadrant, QuadrantParseError, Quadrants, Sides};

use crate::axis::nice_domain;

/// Inclusive data extent of all series on both axes.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DataBounds {
    pub x: Option<(f64, f64)>,
    pub y: Option<(f64, f64)>,
}

impl DataBounds {
    /// Computes bounds from `(x, y)` points, ignoring non-finite values.
    pub fn from_points(points: impl IntoIterator<Item = (f64, f64)>) -> Self {
        let mut b = DataBounds::default();
        for (x, y) in points {
            if x.is_finite() && y.is_finite() {
                b.x = Some(extend(b.x, x));
                b.y = Some(extend(b.y, y));
            }
        }
        b
    }
}

fn extend(range: Option<(f64, f64)>, v: f64) -> (f64, f64) {
    match range {
        Some((lo, hi)) => (lo.min(v), hi.max(v)),
        None => (v, v),
    }
}

/// User-facing viewport settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportSpec {
    pub x_range: Option<(f64, f64)>,
    pub y_range: Option<(f64, f64)>,
    pub quadrants: Option<Quadrants>,
    /// Origin position as a fraction of the plot area, measured along each
    /// axis direction: `(0.0, 0.0)` = bottom-left, `(1.0, 1.0)` = top-right.
    pub origin: Option<(f64, f64)>,
    /// Extra room around the data as a fraction of its extent.
    pub padding: f64,
    /// Round automatic x domain to tick boundaries.
    pub x_nice: bool,
    /// Round automatic y domain to tick boundaries.
    pub y_nice: bool,
}

impl Default for ViewportSpec {
    fn default() -> Self {
        Self {
            x_range: None,
            y_range: None,
            quadrants: None,
            origin: None,
            padding: 0.05,
            x_nice: true,
            y_nice: true,
        }
    }
}

impl ViewportSpec {
    /// `true` when the user asked for a math-style (quadrant / origin) view.
    pub fn is_origin_style(&self) -> bool {
        self.origin.is_some() || self.quadrants.is_some_and(|q| !q.is_empty())
    }
}

/// The resolved visible domain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub x: (f64, f64),
    pub y: (f64, f64),
}

impl Viewport {
    /// Resolves the visible domain. `x_target` / `y_target` are the desired
    /// tick counts, used to round automatic domains to nice values.
    pub fn resolve(
        spec: &ViewportSpec,
        data: &DataBounds,
        x_target: usize,
        y_target: usize,
    ) -> Self {
        let x = resolve_axis(
            spec.x_range,
            spec.origin.map(|o| o.0),
            spec.quadrants.and_then(Quadrants::x_sides),
            data.x,
            spec.padding,
            spec.x_nice,
            x_target,
        );
        let y = resolve_axis(
            spec.y_range,
            spec.origin.map(|o| o.1),
            spec.quadrants.and_then(Quadrants::y_sides),
            data.y,
            spec.padding,
            spec.y_nice,
            y_target,
        );
        Self { x, y }
    }

    /// Where zero sits along the x axis as a fraction (`0.0` = left edge).
    /// Clamped to `0.0..=1.0`.
    pub fn x_zero_fraction(&self) -> f64 {
        zero_fraction(self.x)
    }

    /// Where zero sits along the y axis as a fraction (`0.0` = bottom edge).
    /// Clamped to `0.0..=1.0`.
    pub fn y_zero_fraction(&self) -> f64 {
        zero_fraction(self.y)
    }
}

fn zero_fraction((lo, hi): (f64, f64)) -> f64 {
    if hi == lo {
        return 0.0;
    }
    ((0.0 - lo) / (hi - lo)).clamp(0.0, 1.0)
}

fn resolve_axis(
    explicit: Option<(f64, f64)>,
    origin_ratio: Option<f64>,
    sides: Option<Sides>,
    data: Option<(f64, f64)>,
    padding: f64,
    nice: bool,
    target: usize,
) -> (f64, f64) {
    if let Some((lo, hi)) = explicit {
        return non_degenerate(lo, hi);
    }
    let (data_lo, data_hi) = data.unwrap_or((0.0, 0.0));

    if let Some(r) = origin_ratio {
        return origin_domain(r, data_lo, data_hi, padding);
    }

    if let Some(sides) = sides {
        return match sides {
            Sides::Positive => {
                let hi = data_hi.max(0.0);
                (
                    0.0,
                    if hi > 0.0 {
                        nice_domain(0.0, hi, target).1
                    } else {
                        1.0
                    },
                )
            }
            Sides::Negative => {
                let lo = data_lo.min(0.0);
                (
                    if lo < 0.0 {
                        nice_domain(lo, 0.0, target).0
                    } else {
                        -1.0
                    },
                    0.0,
                )
            }
            Sides::Both => {
                let (lo, hi) = (data_lo.min(0.0), data_hi.max(0.0));
                if lo == hi {
                    (-1.0, 1.0)
                } else {
                    nice_domain(lo, hi, target)
                }
            }
        };
    }

    match data {
        Some((lo, hi)) if nice => nice_domain(lo, hi, target),
        Some((lo, hi)) => non_degenerate(lo, hi),
        None => (0.0, 1.0),
    }
}

/// Domain such that zero sits at fraction `r` of the axis while every data
/// value stays visible.
fn origin_domain(r: f64, data_lo: f64, data_hi: f64, padding: f64) -> (f64, f64) {
    let r = if r.is_finite() {
        r.clamp(0.0, 1.0)
    } else {
        0.5
    };
    let pos = data_hi.max(0.0) * (1.0 + padding);
    let neg = (-data_lo).max(0.0) * (1.0 + padding);

    if r <= 0.0 {
        return (0.0, if pos > 0.0 { pos } else { 1.0 });
    }
    if r >= 1.0 {
        return (if neg > 0.0 { -neg } else { -1.0 }, 0.0);
    }

    // neg / (neg + pos) == r  <=>  neg == pos * r / (1 - r)
    let mut p = pos.max(neg * (1.0 - r) / r);
    if p == 0.0 {
        p = 1.0;
    }
    (-(p * r / (1.0 - r)), p)
}

fn non_degenerate(lo: f64, hi: f64) -> (f64, f64) {
    let (lo, hi) = (lo.min(hi), lo.max(hi));
    if lo == hi {
        (lo - 1.0, hi + 1.0)
    } else {
        (lo, hi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(x: (f64, f64), y: (f64, f64)) -> DataBounds {
        DataBounds {
            x: Some(x),
            y: Some(y),
        }
    }

    #[test]
    fn auto_uses_nice_domain() {
        let v = Viewport::resolve(
            &ViewportSpec::default(),
            &bounds((1.0, 9.0), (3.0, 97.0)),
            5,
            5,
        );
        assert_eq!(v.x, (0.0, 10.0));
        assert_eq!(v.y, (0.0, 100.0));
    }

    #[test]
    fn tight_domain_fits_data() {
        let spec = ViewportSpec {
            x_nice: false,
            ..Default::default()
        };
        let v = Viewport::resolve(&spec, &bounds((1.0, 12.0), (3.0, 97.0)), 4, 5);
        assert_eq!(v.x, (1.0, 12.0));
        assert_eq!(v.y, (0.0, 100.0));
    }

    #[test]
    fn explicit_range_wins() {
        let spec = ViewportSpec {
            x_range: Some((-2.0, 10.0)),
            quadrants: Some(Quadrant::Q1.into()),
            ..Default::default()
        };
        let v = Viewport::resolve(&spec, &bounds((-5.0, 5.0), (-5.0, 5.0)), 5, 5);
        assert_eq!(v.x, (-2.0, 10.0));
        assert_eq!(v.y.0, 0.0); // y still follows the quadrant selection
    }

    #[test]
    fn first_quadrant_only() {
        let spec = ViewportSpec {
            quadrants: Some(Quadrant::Q1.into()),
            ..Default::default()
        };
        let v = Viewport::resolve(&spec, &bounds((-5.0, 8.0), (-3.0, 42.0)), 5, 5);
        assert_eq!(v.x, (0.0, 8.0));
        assert_eq!(v.y, (0.0, 50.0));
    }

    #[test]
    fn negative_and_both_sides() {
        let spec = ViewportSpec {
            quadrants: Some([Quadrant::Q2, Quadrant::Q3].into()),
            ..Default::default()
        };
        let v = Viewport::resolve(&spec, &bounds((-7.0, 3.0), (-4.0, 4.0)), 5, 5);
        assert_eq!(v.x, (-8.0, 0.0));
        assert!(v.y.0 <= -4.0 && v.y.1 >= 4.0);
    }

    #[test]
    fn origin_ratio_places_zero() {
        let spec = ViewportSpec {
            origin: Some((0.25, 0.75)),
            padding: 0.0,
            ..Default::default()
        };
        let v = Viewport::resolve(&spec, &bounds((-1.0, 9.0), (-6.0, 1.0)), 5, 5);
        assert!((v.x_zero_fraction() - 0.25).abs() < 1e-12);
        assert!((v.y_zero_fraction() - 0.75).abs() < 1e-12);
        // All data must stay visible.
        assert!(v.x.0 <= -1.0 && v.x.1 >= 9.0);
        assert!(v.y.0 <= -6.0 && v.y.1 >= 1.0);
    }

    #[test]
    fn origin_ratio_at_edges() {
        let spec = ViewportSpec {
            origin: Some((0.0, 1.0)),
            padding: 0.0,
            ..Default::default()
        };
        let v = Viewport::resolve(&spec, &bounds((0.0, 4.0), (-2.0, 0.0)), 5, 5);
        assert_eq!(v.x, (0.0, 4.0));
        assert_eq!(v.y, (-2.0, 0.0));
    }

    #[test]
    fn empty_data_has_sane_domain() {
        let v = Viewport::resolve(&ViewportSpec::default(), &DataBounds::default(), 5, 5);
        assert_eq!(v.x, (0.0, 1.0));
        assert_eq!(v.y, (0.0, 1.0));
    }
}
