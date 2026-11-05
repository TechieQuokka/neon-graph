use crate::style::{Color, LineCap, LineJoin, Stroke, Theme};

/// Per-series style overrides. `None` fields fall back to the theme.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SeriesStyle {
    pub color: Option<Color>,
    pub width: Option<f64>,
    pub dash: Option<Vec<f64>>,
    pub markers: Option<bool>,
    pub marker_radius: Option<f64>,
    pub opacity: Option<f64>,
    pub glow: Option<bool>,
}

impl SeriesStyle {
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self
    }

    /// Dash pattern in pixels, e.g. `[6.0, 4.0]`.
    pub fn dash(mut self, pattern: impl Into<Vec<f64>>) -> Self {
        self.dash = Some(pattern.into());
        self
    }

    pub fn markers(mut self, show: bool) -> Self {
        self.markers = Some(show);
        self
    }

    pub fn marker_radius(mut self, radius: f64) -> Self {
        self.marker_radius = Some(radius);
        self
    }

    pub fn opacity(mut self, opacity: f64) -> Self {
        self.opacity = Some(opacity.clamp(0.0, 1.0));
        self
    }

    /// Enables or disables the glow effect for this series only.
    pub fn glow(mut self, glow: bool) -> Self {
        self.glow = Some(glow);
        self
    }
}

/// A named sequence of `(x, y)` points.
///
/// Non-finite values (`NaN`, infinities) create gaps in the line.
#[derive(Debug, Clone, PartialEq)]
pub struct Series {
    pub name: String,
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
    pub style: SeriesStyle,
}

impl Series {
    /// Creates a series. Extra values in the longer of `xs`/`ys` are ignored.
    pub fn new(name: impl Into<String>, xs: impl AsRef<[f64]>, ys: impl AsRef<[f64]>) -> Self {
        let (xs, ys) = (xs.as_ref(), ys.as_ref());
        let n = xs.len().min(ys.len());
        Self {
            name: name.into(),
            xs: xs[..n].to_vec(),
            ys: ys[..n].to_vec(),
            style: SeriesStyle::default(),
        }
    }

    /// Creates a series whose x values are `1, 2, 3, ...`.
    pub fn from_y(name: impl Into<String>, ys: impl AsRef<[f64]>) -> Self {
        let ys = ys.as_ref();
        let xs: Vec<f64> = (1..=ys.len()).map(|i| i as f64).collect();
        Self::new(name, xs, ys)
    }

    pub fn len(&self) -> usize {
        self.xs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.xs.is_empty()
    }

    pub fn points(&self) -> impl Iterator<Item = (f64, f64)> + '_ {
        self.xs.iter().copied().zip(self.ys.iter().copied())
    }

    /// Final line color for the `index`-th series.
    pub fn color(&self, index: usize, theme: &Theme) -> Color {
        self.style
            .color
            .unwrap_or_else(|| theme.series.palette.get(index))
    }

    /// Final line stroke for the `index`-th series.
    pub fn stroke(&self, index: usize, theme: &Theme) -> Stroke {
        let opacity = self.style.opacity.unwrap_or(theme.series.opacity) as f32;
        let color = self.color(index, theme);
        Stroke::new(
            color.with_alpha(color.a * opacity),
            self.style.width.unwrap_or(theme.series.width),
        )
        .dash(self.style.dash.clone().unwrap_or_default())
        .cap(LineCap::Round)
        .join(LineJoin::Round)
    }

    pub fn markers(&self, theme: &Theme) -> bool {
        self.style.markers.unwrap_or(theme.series.markers)
    }

    pub fn marker_radius(&self, theme: &Theme) -> f64 {
        self.style
            .marker_radius
            .unwrap_or(theme.series.marker_radius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_to_shorter() {
        let s = Series::new("a", [1.0, 2.0, 3.0], [4.0, 5.0]);
        assert_eq!(s.len(), 2);
        assert_eq!(Series::from_y("b", [9.0, 8.0]).xs, vec![1.0, 2.0]);
    }

    #[test]
    fn style_overrides_theme() {
        let theme = Theme::light();
        let mut s = Series::from_y("a", [1.0]);
        assert_eq!(s.color(1, &theme), theme.series.palette.get(1));
        s.style = SeriesStyle::default().color(Color::WHITE).width(5.0);
        let stroke = s.stroke(1, &theme);
        assert_eq!(stroke.color, Color::WHITE);
        assert_eq!(stroke.width, 5.0);
    }
}
