//! Point reduction for large series.
//!
//! When a line has many more points than pixels, most of them are invisible.
//! The M4 algorithm keeps, for every pixel column, the first, last, minimum
//! and maximum point. The rendered line is pixel-identical, but the output
//! stays small (at most four points per column) no matter how big the input.

use crate::geom::Point;

/// Reduces `points` (screen space, x non-decreasing) to at most four points
/// per pixel column. Returns `None` when the input is not sorted by x.
pub fn m4(points: &[Point]) -> Option<Vec<Point>> {
    if points.windows(2).any(|w| w[1].x < w[0].x) {
        return None;
    }

    let mut out = Vec::with_capacity(points.len().min(4096));
    let mut start = 0;
    while start < points.len() {
        let column = points[start].x.floor();
        let mut end = start + 1;
        let (mut min_i, mut max_i) = (start, start);
        while end < points.len() && points[end].x.floor() == column {
            if points[end].y < points[min_i].y {
                min_i = end;
            }
            if points[end].y > points[max_i].y {
                max_i = end;
            }
            end += 1;
        }
        let last = end - 1;

        let mut picks = [start, min_i, max_i, last];
        picks.sort_unstable();
        let mut prev = usize::MAX;
        for i in picks {
            if i != prev {
                out.push(points[i]);
                prev = i;
            }
        }
        start = end;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_extremes_per_column() {
        let pts: Vec<Point> = (0..1000)
            .map(|i| Point::new(i as f64 / 100.0, ((i * 7919) % 101) as f64))
            .collect();
        let reduced = m4(&pts).unwrap();
        assert!(reduced.len() <= 10 * 4);
        // Global extremes survive.
        let max_in = pts.iter().map(|p| p.y).fold(f64::MIN, f64::max);
        let max_out = reduced.iter().map(|p| p.y).fold(f64::MIN, f64::max);
        assert_eq!(max_in, max_out);
        assert_eq!(reduced.first(), pts.first());
        assert_eq!(reduced.last(), pts.last());
    }

    #[test]
    fn rejects_unsorted() {
        assert!(m4(&[Point::new(1.0, 0.0), Point::new(0.0, 0.0)]).is_none());
    }
}
