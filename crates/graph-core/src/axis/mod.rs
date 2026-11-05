//! Axis configuration, tick generation and number formatting.

mod format;
pub mod ticks;

pub use format::NumberFormat;
pub use ticks::{Ticks, nice_domain, nice_step, ticks};

use crate::style::Color;

/// Where axis lines are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum AxisPosition {
    /// `Origin` when a quadrant or origin ratio is configured, `Edge` otherwise.
    #[default]
    Auto,
    /// Bottom (x) and left (y) edges of the plot area.
    Edge,
    /// Through the data origin `(0, 0)`, mathematical style. Falls back to the
    /// nearest edge when zero is outside the visible domain.
    Origin,
}

/// Configuration of one axis.
///
/// Text toggles are `Option<bool>`: `None` means "follow the chart's text
/// preset", `Some(_)` overrides it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Axis {
    pub(crate) label: Option<String>,
    pub(crate) show_label: Option<bool>,
    pub(crate) show_tick_labels: Option<bool>,
    pub(crate) show_ticks: bool,
    pub(crate) show_line: bool,
    pub(crate) max_ticks: Option<usize>,
    pub(crate) format: NumberFormat,
    pub(crate) range: Option<(f64, f64)>,
    pub(crate) nice: Option<bool>,
    pub(crate) line_color: Option<Color>,
    pub(crate) tick_label_color: Option<Color>,
}

impl Axis {
    pub fn new() -> Self {
        Self {
            show_ticks: true,
            show_line: true,
            ..Default::default()
        }
    }

    /// Axis title, e.g. `"금액"`.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = Some(show);
        self
    }

    pub fn show_tick_labels(mut self, show: bool) -> Self {
        self.show_tick_labels = Some(show);
        self
    }

    /// Small tick marks next to the axis line.
    pub fn show_ticks(mut self, show: bool) -> Self {
        self.show_ticks = show;
        self
    }

    pub fn show_line(mut self, show: bool) -> Self {
        self.show_line = show;
        self
    }

    /// Approximate upper bound on the number of tick intervals.
    pub fn max_ticks(mut self, n: usize) -> Self {
        self.max_ticks = Some(n.max(1));
        self
    }

    pub fn format(mut self, format: NumberFormat) -> Self {
        self.format = format;
        self
    }

    /// Fixes the visible data range of this axis. Takes precedence over
    /// quadrant and origin settings.
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.range = Some((min.min(max), min.max(max)));
        self
    }

    /// Round automatic domains outwards to tick boundaries (`true`), or fit
    /// the data tightly (`false`). Defaults: x tight, y nice.
    pub fn nice(mut self, nice: bool) -> Self {
        self.nice = Some(nice);
        self
    }

    /// Overrides the theme's axis line color for this axis only.
    pub fn line_color(mut self, color: Color) -> Self {
        self.line_color = Some(color);
        self
    }

    /// Overrides the theme's tick label color for this axis only.
    pub fn tick_label_color(mut self, color: Color) -> Self {
        self.tick_label_color = Some(color);
        self
    }

    pub fn get_label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn get_range(&self) -> Option<(f64, f64)> {
        self.range
    }
}
