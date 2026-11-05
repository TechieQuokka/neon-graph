//! Themes: the single source of truth for every visual parameter.

mod cyberpunk;
mod dark;
mod light;

use super::{Color, Glow, Palette, Stroke, TextStyle};
use crate::geom::Insets;

/// Grid line configuration for one direction.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GridStyle {
    pub visible: bool,
    pub stroke: Stroke,
}

/// Default styling for data series.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SeriesDefaults {
    pub palette: Palette,
    /// Line width in pixels.
    pub width: f64,
    /// Draw a marker (dot) at every data point.
    pub markers: bool,
    pub marker_radius: f64,
    /// Line opacity (`0.0..=1.0`).
    pub opacity: f64,
}

/// Gaps between layout elements, in pixels.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Spacing {
    /// Between title and subtitle.
    pub title_subtitle: f64,
    /// Between header blocks (title group, legend, axis label) and the plot.
    pub section: f64,
    /// Between a tick (or axis line) and its label.
    pub tick_label: f64,
    /// Between tick labels and the axis label.
    pub axis_label: f64,
    /// Between legend items on the same row.
    pub legend_item: f64,
    /// Between legend rows.
    pub legend_row: f64,
    /// Length of the color swatch in the legend.
    pub legend_swatch: f64,
    /// Between the end of a line and its direct label.
    pub direct_label: f64,
}

impl Default for Spacing {
    fn default() -> Self {
        Self {
            title_subtitle: 4.0,
            section: 14.0,
            tick_label: 6.0,
            axis_label: 6.0,
            legend_item: 18.0,
            legend_row: 6.0,
            legend_swatch: 16.0,
            direct_label: 8.0,
        }
    }
}

/// A complete visual configuration for a chart.
///
/// Use one of the built-in themes ([`Theme::light`], [`Theme::dark`],
/// [`Theme::cyberpunk`]) and tweak any field:
///
/// ```
/// use graph_core::style::{Color, Theme};
///
/// let mut theme = Theme::dark();
/// theme.background = Color::hex("#000000");
/// theme.series.width = 3.0;
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Theme {
    pub name: String,

    /// Whole-canvas background.
    pub background: Color,
    /// Background of the plot area only.
    pub plot_background: Color,

    pub title: TextStyle,
    pub subtitle: TextStyle,
    pub axis_label: TextStyle,
    pub tick_label: TextStyle,
    pub legend: TextStyle,

    /// Axis lines drawn at the plot edges.
    pub axis_line: Stroke,
    /// Axis lines drawn through the origin (quadrant mode).
    pub origin_axis_line: Stroke,
    /// Tick marks.
    pub tick: Stroke,
    pub tick_length: f64,

    /// Vertical grid lines (at x ticks).
    pub grid_x: GridStyle,
    /// Horizontal grid lines (at y ticks).
    pub grid_y: GridStyle,

    pub series: SeriesDefaults,
    pub glow: Glow,

    /// Outer padding around the whole chart.
    pub padding: Insets,
    pub spacing: Spacing,
}

impl Theme {
    /// Names of all built-in themes.
    pub const BUILTIN: [&'static str; 3] = ["light", "dark", "cyberpunk"];

    /// Clean light theme for reports. Default for the library.
    pub fn light() -> Self {
        light::theme()
    }

    /// Neutral dark theme.
    pub fn dark() -> Self {
        dark::theme()
    }

    /// Neon-on-navy theme with glow. Default for the CLI.
    pub fn cyberpunk() -> Self {
        cyberpunk::theme()
    }

    /// Looks up a built-in theme by name (case-insensitive).
    pub fn by_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "light" => Some(Self::light()),
            "dark" => Some(Self::dark()),
            "cyberpunk" => Some(Self::cyberpunk()),
            _ => None,
        }
    }

    /// Sets the font family of every text element at once.
    pub fn set_font_family(&mut self, family: &str) {
        for style in [
            &mut self.title,
            &mut self.subtitle,
            &mut self.axis_label,
            &mut self.tick_label,
            &mut self.legend,
        ] {
            style.family = family.to_owned();
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_lookup() {
        for name in Theme::BUILTIN {
            assert_eq!(Theme::by_name(name).unwrap().name, name);
        }
        assert_eq!(Theme::by_name(" CyberPunk ").unwrap().name, "cyberpunk");
        assert!(Theme::by_name("nope").is_none());
    }

    #[test]
    fn cyberpunk_glows_by_default() {
        assert!(Theme::cyberpunk().glow.enabled);
        assert!(!Theme::light().glow.enabled);
    }
}
