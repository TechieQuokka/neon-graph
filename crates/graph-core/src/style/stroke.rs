use super::Color;

/// Shape at the end of open lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

/// Shape at the corners of lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

/// How a line is drawn.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Stroke {
    pub color: Color,
    pub width: f64,
    /// Dash pattern in pixels (`[on, off, ...]`). Empty means a solid line.
    #[cfg_attr(feature = "serde", serde(default))]
    pub dash: Vec<f64>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub cap: LineCap,
    #[cfg_attr(feature = "serde", serde(default))]
    pub join: LineJoin,
}

impl Stroke {
    pub fn new(color: Color, width: f64) -> Self {
        Self {
            color,
            width,
            dash: Vec::new(),
            cap: LineCap::Butt,
            join: LineJoin::Miter,
        }
    }

    pub fn dash(mut self, pattern: impl Into<Vec<f64>>) -> Self {
        self.dash = pattern.into();
        self
    }

    pub fn cap(mut self, cap: LineCap) -> Self {
        self.cap = cap;
        self
    }

    pub fn join(mut self, join: LineJoin) -> Self {
        self.join = join;
        self
    }

    /// Returns `true` when the stroke would draw nothing.
    pub fn is_invisible(&self) -> bool {
        self.width <= 0.0 || self.color.is_transparent()
    }
}
