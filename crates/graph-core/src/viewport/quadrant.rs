use std::fmt;
use std::str::FromStr;

/// One of the four quadrants of the Cartesian plane.
///
/// ```text
///   Q2 | Q1        Q1: x >= 0, y >= 0
///  ----+----       Q2: x <= 0, y >= 0
///   Q3 | Q4        Q3: x <= 0, y <= 0
///                  Q4: x >= 0, y <= 0
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Quadrant {
    Q1,
    Q2,
    Q3,
    Q4,
}

impl Quadrant {
    pub const ALL: [Quadrant; 4] = [Quadrant::Q1, Quadrant::Q2, Quadrant::Q3, Quadrant::Q4];

    const fn bit(self) -> u8 {
        match self {
            Quadrant::Q1 => 1,
            Quadrant::Q2 => 2,
            Quadrant::Q3 => 4,
            Quadrant::Q4 => 8,
        }
    }

    /// Returns `(x_positive, y_positive)`.
    const fn signs(self) -> (bool, bool) {
        match self {
            Quadrant::Q1 => (true, true),
            Quadrant::Q2 => (false, true),
            Quadrant::Q3 => (false, false),
            Quadrant::Q4 => (true, false),
        }
    }
}

/// Which side(s) of zero an axis must show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sides {
    Positive,
    Negative,
    Both,
}

/// A set of visible quadrants.
///
/// The visible area is always a rectangle, so the set is reduced to the
/// smallest rectangle covering every selected quadrant: `{Q1, Q3}` therefore
/// shows all four quadrants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Quadrants(u8);

impl Quadrants {
    pub const NONE: Quadrants = Quadrants(0);
    pub const ALL: Quadrants = Quadrants(0b1111);

    pub fn with(self, q: Quadrant) -> Self {
        Self(self.0 | q.bit())
    }

    pub fn contains(self, q: Quadrant) -> bool {
        self.0 & q.bit() != 0
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn iter(self) -> impl Iterator<Item = Quadrant> {
        Quadrant::ALL.into_iter().filter(move |q| self.contains(*q))
    }

    /// Sides of zero the x axis must cover. `None` for an empty set.
    pub fn x_sides(self) -> Option<Sides> {
        self.sides(|q| q.signs().0)
    }

    /// Sides of zero the y axis must cover. `None` for an empty set.
    pub fn y_sides(self) -> Option<Sides> {
        self.sides(|q| q.signs().1)
    }

    fn sides(self, positive: impl Fn(Quadrant) -> bool) -> Option<Sides> {
        let (mut pos, mut neg) = (false, false);
        for q in self.iter() {
            if positive(q) {
                pos = true;
            } else {
                neg = true;
            }
        }
        match (pos, neg) {
            (true, true) => Some(Sides::Both),
            (true, false) => Some(Sides::Positive),
            (false, true) => Some(Sides::Negative),
            (false, false) => None,
        }
    }
}

impl From<Quadrant> for Quadrants {
    fn from(q: Quadrant) -> Self {
        Quadrants::NONE.with(q)
    }
}

impl FromIterator<Quadrant> for Quadrants {
    fn from_iter<T: IntoIterator<Item = Quadrant>>(iter: T) -> Self {
        iter.into_iter().fold(Quadrants::NONE, Quadrants::with)
    }
}

impl<const N: usize> From<[Quadrant; N]> for Quadrants {
    fn from(qs: [Quadrant; N]) -> Self {
        qs.into_iter().collect()
    }
}

/// Error returned when parsing [`Quadrants`] fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuadrantParseError(String);

impl fmt::Display for QuadrantParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid quadrant list {:?} (expected numbers 1-4 like \"1\" or \"1,4\", or \"all\")",
            self.0
        )
    }
}

impl std::error::Error for QuadrantParseError {}

impl FromStr for Quadrants {
    type Err = QuadrantParseError;

    /// Parses `"1"`, `"1,4"`, `"q2"`, `"14"` or `"all"`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || QuadrantParseError(s.to_owned());
        let t = s.trim().to_ascii_lowercase();
        if t == "all" {
            return Ok(Quadrants::ALL);
        }
        let mut set = Quadrants::NONE;
        for ch in t.chars().filter(|c| !matches!(c, ',' | ' ' | 'q')) {
            let q = match ch {
                '1' => Quadrant::Q1,
                '2' => Quadrant::Q2,
                '3' => Quadrant::Q3,
                '4' => Quadrant::Q4,
                _ => return Err(err()),
            };
            set = set.with(q);
        }
        if set.is_empty() { Err(err()) } else { Ok(set) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sides_from_sets() {
        let q1: Quadrants = Quadrant::Q1.into();
        assert_eq!(q1.x_sides(), Some(Sides::Positive));
        assert_eq!(q1.y_sides(), Some(Sides::Positive));

        let right: Quadrants = [Quadrant::Q1, Quadrant::Q4].into();
        assert_eq!(right.x_sides(), Some(Sides::Positive));
        assert_eq!(right.y_sides(), Some(Sides::Both));

        let diag: Quadrants = [Quadrant::Q1, Quadrant::Q3].into();
        assert_eq!(diag.x_sides(), Some(Sides::Both));
        assert_eq!(Quadrants::NONE.x_sides(), None);
    }

    #[test]
    fn parse() {
        assert_eq!("1".parse::<Quadrants>().unwrap(), Quadrant::Q1.into());
        assert_eq!(
            "1, 4".parse::<Quadrants>().unwrap(),
            [Quadrant::Q1, Quadrant::Q4].into()
        );
        assert_eq!("q2".parse::<Quadrants>().unwrap(), Quadrant::Q2.into());
        assert_eq!("all".parse::<Quadrants>().unwrap(), Quadrants::ALL);
        assert!("5".parse::<Quadrants>().is_err());
        assert!("".parse::<Quadrants>().is_err());
    }
}
