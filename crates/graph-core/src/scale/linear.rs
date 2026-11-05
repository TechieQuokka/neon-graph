use super::Scale;

/// A linear mapping `domain -> range`.
///
/// A degenerate domain (`start == end`) maps every value to the middle of the
/// range instead of producing NaN.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearScale {
    domain: (f64, f64),
    range: (f64, f64),
}

impl LinearScale {
    pub fn new(domain: (f64, f64), range: (f64, f64)) -> Self {
        Self { domain, range }
    }

    fn domain_span(&self) -> f64 {
        self.domain.1 - self.domain.0
    }

    fn range_span(&self) -> f64 {
        self.range.1 - self.range.0
    }
}

impl Scale for LinearScale {
    #[inline]
    fn map(&self, value: f64) -> f64 {
        let span = self.domain_span();
        if span == 0.0 {
            return self.range.0 + self.range_span() / 2.0;
        }
        self.range.0 + (value - self.domain.0) / span * self.range_span()
    }

    #[inline]
    fn invert(&self, pixel: f64) -> f64 {
        let span = self.range_span();
        if span == 0.0 {
            return self.domain.0 + self.domain_span() / 2.0;
        }
        self.domain.0 + (pixel - self.range.0) / span * self.domain_span()
    }

    fn domain(&self) -> (f64, f64) {
        self.domain
    }

    fn range(&self) -> (f64, f64) {
        self.range
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_and_inverts() {
        // Typical y scale: data grows upwards, pixels grow downwards.
        let s = LinearScale::new((0.0, 100.0), (400.0, 0.0));
        assert_eq!(s.map(0.0), 400.0);
        assert_eq!(s.map(100.0), 0.0);
        assert_eq!(s.map(25.0), 300.0);
        assert_eq!(s.invert(300.0), 25.0);
        assert!(s.contains(50.0));
        assert!(!s.contains(-1.0));
    }

    #[test]
    fn degenerate_domain_maps_to_center() {
        let s = LinearScale::new((5.0, 5.0), (0.0, 100.0));
        assert_eq!(s.map(5.0), 50.0);
        assert_eq!(s.map(123.0), 50.0);
    }
}
