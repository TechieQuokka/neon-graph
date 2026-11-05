//! Computes where every block of a chart goes.
//!
//! Only visible elements take space: hiding the y tick labels, for example,
//! gives their width back to the plot area.

mod legend;

pub use legend::{LegendItem, LegendLayout, layout_legend};

use crate::geom::{Insets, Point, Rect, Size};
use crate::style::Theme;
use crate::text::{TextMeasurer, max_width};

/// Minimum plot size so degenerate configurations still render something.
const MIN_PLOT: f64 = 10.0;

/// Everything the layout needs to know. Hidden elements are `None` / empty.
#[derive(Clone, Copy)]
pub struct LayoutInput<'a> {
    pub size: Size,
    /// Outer padding (usually `theme.padding`).
    pub padding: Insets,
    pub theme: &'a Theme,
    pub measurer: &'a dyn TextMeasurer,
    pub title: Option<&'a str>,
    pub subtitle: Option<&'a str>,
    /// Names for the top legend. `None` when the legend is hidden or drawn
    /// as direct labels.
    pub legend: Option<&'a [&'a str]>,
    pub x_label: Option<&'a str>,
    pub y_label: Option<&'a str>,
    pub x_tick_labels: &'a [String],
    pub y_tick_labels: &'a [String],
    pub x_ticks: bool,
    pub y_ticks: bool,
    /// Axes cross at the origin instead of the plot edges.
    pub origin_axes: bool,
    /// Position of the y axis line as a fraction of the plot width (origin mode).
    pub y_axis_fraction: f64,
    /// Position of the x axis line as a fraction of the plot height from the
    /// bottom (origin mode).
    pub x_axis_fraction: f64,
    /// Extra space to keep free on the right (e.g. direct series labels).
    pub right_reserve: f64,
}

impl std::fmt::Debug for LayoutInput<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayoutInput")
            .field("size", &self.size)
            .field("title", &self.title)
            .field("origin_axes", &self.origin_axes)
            .finish_non_exhaustive()
    }
}

/// Resolved positions. Text positions are top-left anchors unless noted.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartLayout {
    pub title: Option<Point>,
    pub subtitle: Option<Point>,
    pub legend: Option<LegendLayout>,
    pub y_label: Option<Point>,
    /// Bottom-right anchor of the x axis label.
    pub x_label: Option<Point>,
    pub plot: Rect,
}

impl ChartLayout {
    pub fn compute(input: &LayoutInput<'_>) -> Self {
        let theme = input.theme;
        let sp = &theme.spacing;
        let m = input.measurer;
        let content = Rect::from_size(input.size).inset(input.padding);

        // ---- Header (top, stacked) ------------------------------------------
        let mut y = content.top();
        let mut header_used = false;

        let title = input.title.map(|_| {
            let p = Point::new(content.left(), y);
            y += theme.title.line_height();
            header_used = true;
            p
        });
        let subtitle = input.subtitle.map(|_| {
            if title.is_some() {
                y += sp.title_subtitle;
            }
            let p = Point::new(content.left(), y);
            y += theme.subtitle.line_height();
            header_used = true;
            p
        });
        if header_used {
            y += sp.section;
        }

        let legend = input.legend.filter(|n| !n.is_empty()).map(|names| {
            let l = layout_legend(
                names,
                Point::new(content.left(), y),
                content.width,
                &theme.legend,
                sp,
                m,
            );
            y += l.size.height + sp.section;
            l
        });

        let y_label = input.y_label.map(|_| {
            let p = Point::new(content.left(), y);
            y += theme.axis_label.line_height() + sp.axis_label;
            p
        });

        // ---- Space needed around the plot for ticks and labels ---------------
        let tick_h = theme.tick_label.line_height();
        let has_y_labels = !input.y_tick_labels.is_empty();
        let has_x_labels = !input.x_tick_labels.is_empty();

        let left_need = {
            let labels = if has_y_labels {
                max_width(
                    m,
                    input.y_tick_labels.iter().map(String::as_str),
                    &theme.tick_label,
                ) + sp.tick_label
            } else {
                0.0
            };
            let ticks = if input.y_ticks {
                theme.tick_length
            } else {
                0.0
            };
            if labels > 0.0 { labels + ticks } else { 0.0 }
        };
        let bottom_need = {
            let labels = if has_x_labels {
                tick_h + sp.tick_label
            } else {
                0.0
            };
            let ticks = if input.x_ticks {
                theme.tick_length
            } else {
                0.0
            };
            if labels > 0.0 { labels + ticks } else { 0.0 }
        };

        // The top/bottom y tick labels are centered on the plot edges and
        // overflow by half a line; the last x tick label overflows to the right.
        let top_overflow = if has_y_labels { tick_h / 2.0 } else { 0.0 };
        let bottom_overflow = if has_y_labels { tick_h / 2.0 } else { 0.0 };
        let right_overflow = input.x_tick_labels.last().map_or(0.0, |l| {
            (m.width(l, &theme.tick_label) / 2.0 - input.padding.right).max(0.0)
        });

        let x_label_h = if input.x_label.is_some() {
            theme.axis_label.line_height() + sp.axis_label
        } else {
            0.0
        };

        let top = y + top_overflow;
        let right = content.right() - right_overflow.max(input.right_reserve);
        let bottom_base = content.bottom() - x_label_h;

        let (left, bottom) = if input.origin_axes {
            // Axes run through the plot, so tick labels may sit inside it.
            // Only add margin when the axis is too close to the edge:
            // margin + f * (span - margin) >= need.
            let avail_w = right - content.left();
            let avail_h = bottom_base - top;
            let extra_left = edge_margin(left_need, input.y_axis_fraction, avail_w);
            let extra_bottom =
                edge_margin(bottom_need, input.x_axis_fraction, avail_h).max(bottom_overflow);
            (content.left() + extra_left, bottom_base - extra_bottom)
        } else {
            (
                content.left() + left_need,
                bottom_base - bottom_need.max(bottom_overflow),
            )
        };

        let mut plot = Rect::from_edges(left, top, right, bottom);
        plot.width = plot.width.max(MIN_PLOT);
        plot.height = plot.height.max(MIN_PLOT);

        let x_label = input
            .x_label
            .map(|_| Point::new(plot.right(), content.bottom()));

        ChartLayout {
            title,
            subtitle,
            legend,
            y_label,
            x_label,
            plot,
        }
    }
}

/// Margin `m` such that `m + f * (span - m) >= need`, clamped at zero.
fn edge_margin(need: f64, fraction: f64, span: f64) -> f64 {
    if need <= 0.0 {
        return 0.0;
    }
    let f = fraction.clamp(0.0, 1.0);
    if f >= 1.0 {
        return if span >= need { 0.0 } else { need };
    }
    ((need - f * span) / (1.0 - f)).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::EstimateMeasurer;

    fn base<'a>(theme: &'a Theme, x: &'a [String], y: &'a [String]) -> LayoutInput<'a> {
        LayoutInput {
            size: Size::new(800.0, 500.0),
            padding: theme.padding,
            theme,
            measurer: &EstimateMeasurer,
            title: Some("제목"),
            subtitle: None,
            legend: None,
            x_label: None,
            y_label: None,
            x_tick_labels: x,
            y_tick_labels: y,
            x_ticks: true,
            y_ticks: true,
            origin_axes: false,
            y_axis_fraction: 0.0,
            x_axis_fraction: 0.0,
            right_reserve: 0.0,
        }
    }

    #[test]
    fn hidden_elements_free_space() {
        let theme = Theme::light();
        let labels: Vec<String> = vec!["0".into(), "10,000".into()];
        let with = ChartLayout::compute(&base(&theme, &labels, &labels));
        let without = ChartLayout::compute(&LayoutInput {
            title: None,
            ..base(&theme, &[], &[])
        });
        assert!(without.plot.width > with.plot.width);
        assert!(without.plot.height > with.plot.height);
        assert_eq!(without.plot.left(), theme.padding.left);
    }

    #[test]
    fn origin_axes_need_no_margin_when_centered() {
        let theme = Theme::light();
        let labels: Vec<String> = vec!["-10".into(), "10".into()];
        let edge = ChartLayout::compute(&base(&theme, &labels, &labels));
        let centered = ChartLayout::compute(&LayoutInput {
            origin_axes: true,
            y_axis_fraction: 0.5,
            x_axis_fraction: 0.5,
            ..base(&theme, &labels, &labels)
        });
        assert!(centered.plot.width > edge.plot.width);
        assert_eq!(centered.plot.left(), theme.padding.left);
    }

    #[test]
    fn edge_margin_math() {
        assert_eq!(edge_margin(30.0, 0.0, 500.0), 30.0);
        assert_eq!(edge_margin(30.0, 0.5, 500.0), 0.0);
        let m = edge_margin(30.0, 0.02, 500.0);
        assert!((m + 0.02 * (500.0 - m) - 30.0).abs() < 1e-9);
    }
}
