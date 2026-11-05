use super::{Point, Size};

/// An axis-aligned rectangle in pixel space.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Builds a rectangle from its edges. Width/height are clamped to `>= 0`.
    pub fn from_edges(left: f64, top: f64, right: f64, bottom: f64) -> Self {
        Self {
            x: left,
            y: top,
            width: (right - left).max(0.0),
            height: (bottom - top).max(0.0),
        }
    }

    pub fn from_size(size: Size) -> Self {
        Self::new(0.0, 0.0, size.width, size.height)
    }

    pub fn left(&self) -> f64 {
        self.x
    }

    pub fn top(&self) -> f64 {
        self.y
    }

    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    pub fn center(&self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Shrinks the rectangle by the given insets.
    pub fn inset(&self, insets: Insets) -> Self {
        Self::from_edges(
            self.left() + insets.left,
            self.top() + insets.top,
            self.right() - insets.right,
            self.bottom() - insets.bottom,
        )
    }

    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.left() && p.x <= self.right() && p.y >= self.top() && p.y <= self.bottom()
    }
}

/// Spacing on each side of a rectangle (margins / paddings).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Insets {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Insets {
    pub const fn new(top: f64, right: f64, bottom: f64, left: f64) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub const fn uniform(v: f64) -> Self {
        Self::new(v, v, v, v)
    }

    pub const fn symmetric(vertical: f64, horizontal: f64) -> Self {
        Self::new(vertical, horizontal, vertical, horizontal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inset_shrinks_and_clamps() {
        let r = Rect::new(0.0, 0.0, 100.0, 50.0).inset(Insets::uniform(10.0));
        assert_eq!(r, Rect::new(10.0, 10.0, 80.0, 30.0));

        let tiny = Rect::new(0.0, 0.0, 10.0, 10.0).inset(Insets::uniform(20.0));
        assert_eq!(tiny.width, 0.0);
        assert_eq!(tiny.height, 0.0);
    }
}
