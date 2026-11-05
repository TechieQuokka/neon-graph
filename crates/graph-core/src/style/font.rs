use super::Color;

/// Default font stack. Korean-capable fonts come first so Hangul renders
/// properly everywhere; viewers fall back along the list.
pub const DEFAULT_FONT_FAMILY: &str = "Pretendard, 'Noto Sans KR', 'Apple SD Gothic Neo', \
     'Malgun Gothic', 'Nanum Gothic', system-ui, sans-serif";

/// Font and color of a text element.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextStyle {
    /// CSS-like font family list.
    pub family: String,
    /// Font size in pixels.
    pub size: f64,
    /// CSS font weight (100..=900).
    pub weight: u16,
    pub color: Color,
}

impl TextStyle {
    pub fn new(size: f64, weight: u16, color: Color) -> Self {
        Self {
            family: DEFAULT_FONT_FAMILY.to_owned(),
            size,
            weight,
            color,
        }
    }

    /// Approximate line height in pixels.
    pub fn line_height(&self) -> f64 {
        self.size * 1.2
    }
}
