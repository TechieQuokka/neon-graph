//! Configuration and drawing shared by every chart kind.

use crate::axis::{Axis, AxisPosition, Ticks, ticks};
use crate::geom::{Insets, Point, Rect, Size};
use crate::layout::{ChartLayout, LayoutInput};
use crate::scale::{LinearScale, Scale};
use crate::scene::{Baseline, Group, Path, PathNode, RectNode, Scene, TextAnchor, TextNode};
use crate::style::{Color, Stroke, TextStyle, Theme};
use crate::text::{TextElement, TextMeasurer, TextPreset, TextVisibility, max_width};
use crate::viewport::{DataBounds, Quadrants, Viewport, ViewportSpec};

/// How series names are shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LegendMode {
    /// Horizontal legend below the title.
    #[default]
    Top,
    /// Each name written at the end of its line, in the line's color.
    Direct,
}

impl std::str::FromStr for LegendMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "top" => Ok(LegendMode::Top),
            "direct" => Ok(LegendMode::Direct),
            _ => Err(format!(
                "unknown legend mode {s:?} (expected top or direct)"
            )),
        }
    }
}

/// Settings common to all charts.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartBase {
    pub size: Size,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub theme: Theme,
    pub preset: TextPreset,
    /// Data only: no text, axes, ticks, grid or outer padding. Explicit text
    /// overrides (`show_*`) still apply.
    pub bare: bool,
    pub show_title: Option<bool>,
    pub show_subtitle: Option<bool>,
    pub show_legend: Option<bool>,
    pub legend_mode: LegendMode,
    pub x_axis: Axis,
    pub y_axis: Axis,
    pub quadrants: Option<Quadrants>,
    pub origin: Option<(f64, f64)>,
    pub data_padding: f64,
    pub axis_position: AxisPosition,
    /// Overrides `theme.glow.enabled`.
    pub glow: Option<bool>,
    /// Overrides grid visibility. `None`: theme (off in bare mode).
    pub grid: Option<bool>,
}

impl Default for ChartBase {
    fn default() -> Self {
        Self {
            size: Size::new(800.0, 500.0),
            title: None,
            subtitle: None,
            theme: Theme::default(),
            preset: TextPreset::Full,
            bare: false,
            show_title: None,
            show_subtitle: None,
            show_legend: None,
            legend_mode: LegendMode::Top,
            x_axis: Axis::new(),
            y_axis: Axis::new(),
            quadrants: None,
            origin: None,
            data_padding: ViewportSpec::default().padding,
            axis_position: AxisPosition::Auto,
            glow: None,
            grid: None,
        }
    }
}

impl ChartBase {
    /// Final visibility of every text element (preset + overrides).
    pub fn visibility(&self) -> TextVisibility {
        let preset = if self.bare {
            TextPreset::Minimal
        } else {
            self.preset
        };
        let mut v = preset.visibility();
        let overrides = [
            (TextElement::Title, self.show_title),
            (TextElement::Subtitle, self.show_subtitle),
            (TextElement::Legend, self.show_legend),
            (TextElement::XLabel, self.x_axis.show_label),
            (TextElement::YLabel, self.y_axis.show_label),
            (TextElement::XTickLabels, self.x_axis.show_tick_labels),
            (TextElement::YTickLabels, self.y_axis.show_tick_labels),
        ];
        for (element, value) in overrides {
            if let Some(show) = value {
                v.set(element, show);
            }
        }
        v
    }

    /// Shows or hides one text element, overriding the preset.
    pub fn set_visible(&mut self, element: TextElement, show: bool) {
        let slot = match element {
            TextElement::Title => &mut self.show_title,
            TextElement::Subtitle => &mut self.show_subtitle,
            TextElement::Legend => &mut self.show_legend,
            TextElement::XLabel => &mut self.x_axis.show_label,
            TextElement::YLabel => &mut self.y_axis.show_label,
            TextElement::XTickLabels => &mut self.x_axis.show_tick_labels,
            TextElement::YTickLabels => &mut self.y_axis.show_tick_labels,
        };
        *slot = Some(show);
    }

    pub fn viewport_spec(&self) -> ViewportSpec {
        ViewportSpec {
            x_range: self.x_axis.range,
            y_range: self.y_axis.range,
            quadrants: self.quadrants,
            origin: self.origin,
            padding: self.data_padding,
            x_nice: self.x_axis.nice.unwrap_or(false),
            // Bare charts fit the data tightly unless asked otherwise.
            y_nice: self.y_axis.nice.unwrap_or(!self.bare),
        }
    }

    /// Whether axes cross at the origin.
    pub fn origin_axes(&self) -> bool {
        match self.axis_position {
            AxisPosition::Auto => self.viewport_spec().is_origin_style(),
            AxisPosition::Edge => false,
            AxisPosition::Origin => true,
        }
    }

    pub fn axis_line_visible(&self, axis: &Axis) -> bool {
        axis.show_line && !self.bare
    }

    pub fn axis_ticks_visible(&self, axis: &Axis) -> bool {
        axis.show_ticks && !self.bare
    }

    /// Vertical grid lines (at x ticks).
    pub fn grid_x_visible(&self) -> bool {
        let theme = self.theme.grid_x.visible;
        match self.grid {
            Some(show) => show && theme,
            None => theme && !self.bare,
        }
    }

    /// Horizontal grid lines (at y ticks). When the grid is forced on and the
    /// theme disables both directions, horizontal lines are used.
    pub fn grid_y_visible(&self) -> bool {
        let theme = self.theme.grid_y.visible;
        match self.grid {
            Some(show) => show && (theme || !self.theme.grid_x.visible),
            None => theme && !self.bare,
        }
    }

    /// Outer padding, never smaller than `bleed` (how far drawing reaches past
    /// the data points, e.g. half a line width plus glow).
    pub fn effective_padding(&self, bleed: f64) -> Insets {
        let p = if self.bare {
            Insets::uniform(0.0)
        } else {
            self.theme.padding
        };
        Insets::new(
            p.top.max(bleed),
            p.right.max(bleed),
            p.bottom.max(bleed),
            p.left.max(bleed),
        )
    }

    pub fn glow_enabled(&self) -> bool {
        self.glow.unwrap_or(self.theme.glow.enabled)
    }

    fn tick_target(&self, axis: &Axis, extent: f64, per_tick: f64) -> usize {
        axis.max_ticks
            .unwrap_or_else(|| ((extent / per_tick).round() as usize).clamp(2, 10))
    }
}

/// Builder methods shared by every chart type holding a `base: ChartBase`.
macro_rules! impl_common_builders {
    ($ty:ty) => {
        impl $ty {
            /// Canvas size in pixels (default 800 x 500).
            pub fn size(mut self, width: f64, height: f64) -> Self {
                self.base.size = $crate::geom::Size::new(width.max(1.0), height.max(1.0));
                self
            }

            pub fn title(mut self, title: impl Into<String>) -> Self {
                self.base.title = Some(title.into());
                self
            }

            pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
                self.base.subtitle = Some(subtitle.into());
                self
            }

            /// Replaces the whole theme.
            pub fn theme(mut self, theme: $crate::style::Theme) -> Self {
                self.base.theme = theme;
                self
            }

            /// Edits any theme field in place.
            pub fn theme_with(mut self, f: impl FnOnce(&mut $crate::style::Theme)) -> Self {
                f(&mut self.base.theme);
                self
            }

            pub fn background(mut self, color: $crate::style::Color) -> Self {
                self.base.theme.background = color;
                self
            }

            pub fn plot_background(mut self, color: $crate::style::Color) -> Self {
                self.base.theme.plot_background = color;
                self
            }

            /// Series colors, used in order and cycled.
            pub fn palette(
                mut self,
                colors: impl IntoIterator<Item = $crate::style::Color>,
            ) -> Self {
                self.base.theme.series.palette = $crate::style::Palette::new(colors);
                self
            }

            /// Font family for every text element.
            pub fn font_family(mut self, family: &str) -> Self {
                self.base.theme.set_font_family(family);
                self
            }

            /// Data only: hides all text, axes, ticks and grid, removes the
            /// outer padding and fits both axes tightly to the data, so the
            /// lines fill the whole canvas. Colors, background and glow stay.
            /// `show_*`, `grid(true)` and `x_axis(|a| a.nice(true))` still apply.
            pub fn bare(mut self) -> Self {
                self.base.bare = true;
                self
            }

            pub fn text_preset(mut self, preset: $crate::text::TextPreset) -> Self {
                self.base.preset = preset;
                self
            }

            /// Shows or hides one text element, overriding the preset.
            pub fn show(mut self, element: $crate::text::TextElement, show: bool) -> Self {
                self.base.set_visible(element, show);
                self
            }

            pub fn show_title(self, show: bool) -> Self {
                self.show($crate::text::TextElement::Title, show)
            }

            pub fn show_subtitle(self, show: bool) -> Self {
                self.show($crate::text::TextElement::Subtitle, show)
            }

            pub fn show_legend(self, show: bool) -> Self {
                self.show($crate::text::TextElement::Legend, show)
            }

            pub fn legend_mode(mut self, mode: $crate::chart::LegendMode) -> Self {
                self.base.legend_mode = mode;
                self
            }

            /// Configures the x axis: `.x_axis(|a| a.label("월").max_ticks(6))`.
            pub fn x_axis(
                mut self,
                f: impl FnOnce($crate::axis::Axis) -> $crate::axis::Axis,
            ) -> Self {
                self.base.x_axis = f(std::mem::take(&mut self.base.x_axis));
                self
            }

            /// Configures the y axis: `.y_axis(|a| a.label("금액"))`.
            pub fn y_axis(
                mut self,
                f: impl FnOnce($crate::axis::Axis) -> $crate::axis::Axis,
            ) -> Self {
                self.base.y_axis = f(std::mem::take(&mut self.base.y_axis));
                self
            }

            pub fn x_label(self, label: impl Into<String>) -> Self {
                self.x_axis(|a| a.label(label))
            }

            pub fn y_label(self, label: impl Into<String>) -> Self {
                self.y_axis(|a| a.label(label))
            }

            /// Fixes the visible x range. Highest precedence.
            pub fn x_range(self, min: f64, max: f64) -> Self {
                self.x_axis(|a| a.range(min, max))
            }

            /// Fixes the visible y range. Highest precedence.
            pub fn y_range(self, min: f64, max: f64) -> Self {
                self.y_axis(|a| a.range(min, max))
            }

            /// Shows only the given quadrant(s): `.quadrants(Quadrant::Q1)`.
            /// Clears any origin ratio.
            pub fn quadrants(mut self, q: impl Into<$crate::viewport::Quadrants>) -> Self {
                self.base.quadrants = Some(q.into());
                self.base.origin = None;
                self
            }

            /// Places the origin at a fraction of the plot area, measured from
            /// the bottom-left: `.origin(0.25, 0.25)` gives Q1 the most room.
            /// Clears any quadrant selection.
            pub fn origin(mut self, x: f64, y: f64) -> Self {
                self.base.origin = Some((x, y));
                self.base.quadrants = None;
                self
            }

            pub fn axis_position(mut self, position: $crate::axis::AxisPosition) -> Self {
                self.base.axis_position = position;
                self
            }

            /// Extra room around automatic domains, as a fraction of the data extent.
            pub fn data_padding(mut self, fraction: f64) -> Self {
                self.base.data_padding = fraction.max(0.0);
                self
            }

            /// Turns the glow effect on or off regardless of the theme.
            /// Shows or hides the grid regardless of the theme and bare mode.
            /// Which directions are drawn (and their colors) follows the theme.
            pub fn grid(mut self, show: bool) -> Self {
                self.base.grid = Some(show);
                self
            }

            pub fn glow(mut self, enabled: bool) -> Self {
                self.base.glow = Some(enabled);
                self
            }

            /// Outer padding around the whole chart.
            pub fn padding(mut self, padding: $crate::geom::Insets) -> Self {
                self.base.theme.padding = padding;
                self
            }

            /// Read access to the shared settings.
            pub fn base(&self) -> &$crate::chart::ChartBase {
                &self.base
            }
        }
    };
}

/// A legend swatch: series name and how its line looks.
#[derive(Debug, Clone, PartialEq)]
pub struct LegendEntry {
    pub name: String,
    pub stroke: Stroke,
}

/// Resolved frame of a chart: layout, scales and ticks, plus helpers to draw
/// everything that is not data.
#[derive(Debug, Clone)]
pub struct Frame<'a> {
    pub base: &'a ChartBase,
    pub visibility: TextVisibility,
    pub viewport: Viewport,
    pub layout: ChartLayout,
    pub x_scale: LinearScale,
    pub y_scale: LinearScale,
    pub x_ticks: Ticks,
    pub y_ticks: Ticks,
    pub x_tick_labels: Vec<String>,
    pub y_tick_labels: Vec<String>,
    pub origin_axes: bool,
    /// Pixel position of the vertical (y) axis line.
    pub y_axis_x: f64,
    /// Pixel position of the horizontal (x) axis line.
    pub x_axis_y: f64,
    legend: Vec<LegendEntry>,
}

impl<'a> Frame<'a> {
    pub fn new(
        base: &'a ChartBase,
        data: &DataBounds,
        legend: Vec<LegendEntry>,
        bleed: f64,
        measurer: &dyn TextMeasurer,
    ) -> Self {
        let theme = &base.theme;
        let vis = base.visibility();

        let x_target = base.tick_target(&base.x_axis, base.size.width, 100.0);
        let y_target = base.tick_target(&base.y_axis, base.size.height, 60.0);
        let viewport = Viewport::resolve(&base.viewport_spec(), data, x_target, y_target);

        let x_ticks = ticks(viewport.x.0, viewport.x.1, x_target);
        let y_ticks = ticks(viewport.y.0, viewport.y.1, y_target);
        let x_tick_labels = base
            .x_axis
            .format
            .format_ticks(&x_ticks.values, x_ticks.step);
        let y_tick_labels = base
            .y_axis
            .format
            .format_ticks(&y_ticks.values, y_ticks.step);

        let text =
            |show: bool, s: &'a Option<String>| s.as_deref().filter(|s| show && !s.is_empty());
        let names: Vec<&str> = legend.iter().map(|e| e.name.as_str()).collect();
        let top_legend = vis.legend && base.legend_mode == LegendMode::Top && !names.is_empty();
        let direct = vis.legend && base.legend_mode == LegendMode::Direct && !names.is_empty();
        let right_reserve = if direct {
            max_width(measurer, names.iter().copied(), &theme.legend) + theme.spacing.direct_label
        } else {
            0.0
        };

        let origin_axes = base.origin_axes();
        let empty: [String; 0] = [];
        let layout = ChartLayout::compute(&LayoutInput {
            size: base.size,
            padding: base.effective_padding(bleed),
            theme,
            measurer,
            title: text(vis.title, &base.title),
            subtitle: text(vis.subtitle, &base.subtitle),
            legend: top_legend.then_some(names.as_slice()),
            // In origin mode axis labels sit at the axis ends inside the plot
            // and need no margin.
            x_label: text(vis.x_label && !origin_axes, &base.x_axis.label),
            y_label: text(vis.y_label && !origin_axes, &base.y_axis.label),
            x_tick_labels: if vis.x_tick_labels {
                &x_tick_labels
            } else {
                &empty
            },
            y_tick_labels: if vis.y_tick_labels {
                &y_tick_labels
            } else {
                &empty
            },
            x_ticks: base.axis_ticks_visible(&base.x_axis),
            y_ticks: base.axis_ticks_visible(&base.y_axis),
            origin_axes,
            y_axis_fraction: viewport.x_zero_fraction(),
            x_axis_fraction: viewport.y_zero_fraction(),
            right_reserve,
        });

        let plot = layout.plot;
        let x_scale = LinearScale::new(viewport.x, (plot.left(), plot.right()));
        let y_scale = LinearScale::new(viewport.y, (plot.bottom(), plot.top()));

        let (y_axis_x, x_axis_y) = if origin_axes {
            (
                x_scale.map(0.0f64.clamp(viewport.x.0, viewport.x.1)),
                y_scale.map(0.0f64.clamp(viewport.y.0, viewport.y.1)),
            )
        } else {
            (plot.left(), plot.bottom())
        };

        Self {
            base,
            visibility: vis,
            viewport,
            layout,
            x_scale,
            y_scale,
            x_ticks,
            y_ticks,
            x_tick_labels,
            y_tick_labels,
            origin_axes,
            y_axis_x,
            x_axis_y,
            legend,
        }
    }

    pub fn plot(&self) -> Rect {
        self.layout.plot
    }

    pub fn theme(&self) -> &Theme {
        &self.base.theme
    }

    /// Data -> pixel.
    pub fn map(&self, x: f64, y: f64) -> Point {
        Point::new(self.x_scale.map(x), self.y_scale.map(y))
    }

    pub fn direct_labels(&self) -> bool {
        self.visibility.legend && self.base.legend_mode == LegendMode::Direct
    }

    /// Background, plot background, grid and axis lines. Draw before data.
    pub fn draw_under(&self, scene: &mut Scene) {
        let theme = self.theme();
        let plot = self.plot();
        if !theme.background.is_transparent() {
            scene.background = Some(theme.background);
        }
        if !theme.plot_background.is_transparent() {
            // Bare charts have no frame: the plot color fills the canvas so the
            // bleed margin does not show up as a border.
            let rect = if self.base.bare {
                Rect::from_size(self.base.size)
            } else {
                plot
            };
            scene.push(RectNode {
                rect,
                fill: Some(theme.plot_background),
                stroke: None,
            });
        }

        let mut grid = Group::new("grid");
        if self.base.grid_x_visible() {
            for &v in &self.x_ticks.values {
                let x = self.x_scale.map(v);
                if !self.on_y_axis(x) {
                    let x = crisp(x, &theme.grid_x.stroke);
                    grid.push(line(
                        Point::new(x, plot.top()),
                        Point::new(x, plot.bottom()),
                        &theme.grid_x.stroke,
                    ));
                }
            }
        }
        if self.base.grid_y_visible() {
            for &v in &self.y_ticks.values {
                let y = self.y_scale.map(v);
                if !self.on_x_axis(y) {
                    let y = crisp(y, &theme.grid_y.stroke);
                    grid.push(line(
                        Point::new(plot.left(), y),
                        Point::new(plot.right(), y),
                        &theme.grid_y.stroke,
                    ));
                }
            }
        }
        if !grid.is_empty() {
            scene.push(grid);
        }

        let mut axes = Group::new("axes");
        let x_axis = &self.base.x_axis;
        let y_axis = &self.base.y_axis;

        if self.base.axis_line_visible(x_axis) {
            let stroke = self.axis_stroke(self.zero_inside(self.viewport.y), x_axis.line_color);
            let y = crisp(self.x_axis_y, &stroke);
            axes.push(line(
                Point::new(plot.left(), y),
                Point::new(plot.right(), y),
                &stroke,
            ));
        }
        if self.base.axis_line_visible(y_axis) {
            let stroke = self.axis_stroke(self.zero_inside(self.viewport.x), y_axis.line_color);
            let x = crisp(self.y_axis_x, &stroke);
            axes.push(line(
                Point::new(x, plot.top()),
                Point::new(x, plot.bottom()),
                &stroke,
            ));
        }

        let tick_stroke = &theme.tick;
        let len = theme.tick_length;
        if self.base.axis_ticks_visible(x_axis) && len > 0.0 {
            for &v in &self.x_ticks.values {
                let x = crisp(self.x_scale.map(v), tick_stroke);
                axes.push(line(
                    Point::new(x, self.x_axis_y),
                    Point::new(x, self.x_axis_y + len),
                    tick_stroke,
                ));
            }
        }
        if self.base.axis_ticks_visible(y_axis) && len > 0.0 {
            for &v in &self.y_ticks.values {
                let y = crisp(self.y_scale.map(v), tick_stroke);
                axes.push(line(
                    Point::new(self.y_axis_x - len, y),
                    Point::new(self.y_axis_x, y),
                    tick_stroke,
                ));
            }
        }
        if !axes.is_empty() {
            scene.push(axes);
        }
    }

    /// Tick labels, titles, legend and axis labels. Draw after data.
    pub fn draw_over(&self, scene: &mut Scene) {
        let theme = self.theme();
        let sp = &theme.spacing;
        let vis = self.visibility;

        // Both axes cross inside the plot: print a single "0" at the origin
        // instead of one on each axis.
        let shared_zero = self.origin_axes
            && self.zero_inside(self.viewport.x)
            && self.zero_inside(self.viewport.y);

        let mut labels = Group::new("tick-labels");
        if vis.x_tick_labels {
            let style = with_color(&theme.tick_label, self.base.x_axis.tick_label_color);
            let offset = if self.base.axis_ticks_visible(&self.base.x_axis) {
                theme.tick_length
            } else {
                0.0
            };
            for (v, text) in self.x_ticks.values.iter().zip(&self.x_tick_labels) {
                if shared_zero && *v == 0.0 {
                    continue;
                }
                let pos = Point::new(self.x_scale.map(*v), self.x_axis_y + offset + sp.tick_label);
                labels.push(
                    TextNode::new(text, pos, style.clone())
                        .anchor(TextAnchor::Middle)
                        .tabular(),
                );
            }
        }
        if vis.y_tick_labels {
            let style = with_color(&theme.tick_label, self.base.y_axis.tick_label_color);
            let offset = if self.base.axis_ticks_visible(&self.base.y_axis) {
                theme.tick_length
            } else {
                0.0
            };
            for (v, text) in self.y_ticks.values.iter().zip(&self.y_tick_labels) {
                if shared_zero && *v == 0.0 {
                    continue;
                }
                let pos = Point::new(self.y_axis_x - offset - sp.tick_label, self.y_scale.map(*v));
                labels.push(
                    TextNode::new(text, pos, style.clone())
                        .anchor(TextAnchor::End)
                        .baseline(Baseline::Middle)
                        .tabular(),
                );
            }
        }
        if shared_zero && (vis.x_tick_labels || vis.y_tick_labels) {
            let pos = Point::new(self.y_axis_x - sp.tick_label, self.x_axis_y + sp.tick_label);
            labels.push(
                TextNode::new("0", pos, theme.tick_label.clone())
                    .anchor(TextAnchor::End)
                    .tabular(),
            );
        }
        if !labels.is_empty() {
            scene.push(labels);
        }

        let mut header = Group::new("text");
        if let (Some(p), Some(t)) = (self.layout.title, &self.base.title) {
            header.push(TextNode::new(t, p, theme.title.clone()));
        }
        if let (Some(p), Some(t)) = (self.layout.subtitle, &self.base.subtitle) {
            header.push(TextNode::new(t, p, theme.subtitle.clone()));
        }
        if let (Some(p), Some(t)) = (self.layout.y_label, &self.base.y_axis.label) {
            header.push(TextNode::new(t, p, theme.axis_label.clone()));
        }
        if let (Some(p), Some(t)) = (self.layout.x_label, &self.base.x_axis.label) {
            header.push(
                TextNode::new(t, p, theme.axis_label.clone())
                    .anchor(TextAnchor::End)
                    .baseline(Baseline::Bottom),
            );
        }
        if self.origin_axes {
            self.origin_axis_labels(&mut header);
        }
        if !header.is_empty() {
            scene.push(header);
        }

        if let Some(legend) = &self.layout.legend {
            let mut g = Group::new("legend");
            for (item, entry) in legend.items.iter().zip(&self.legend) {
                let mut stroke = entry.stroke.clone();
                stroke.width = stroke.width.clamp(1.5, 3.0);
                let end = Point::new(item.swatch.x + sp.legend_swatch, item.swatch.y);
                g.push(line(item.swatch, end, &stroke));
                g.push(
                    TextNode::new(&entry.name, item.label, theme.legend.clone())
                        .baseline(Baseline::Middle),
                );
            }
            scene.push(g);
        }
    }

    /// Draws direct labels at the given anchor points (one per legend entry,
    /// `None` to skip), resolving vertical overlaps.
    pub fn draw_direct_labels(&self, scene: &mut Scene, anchors: &[Option<Point>]) {
        if !self.direct_labels() {
            return;
        }
        let theme = self.theme();
        let h = theme.legend.line_height();
        let plot = self.plot();

        let mut items: Vec<(usize, Point)> = anchors
            .iter()
            .enumerate()
            .filter_map(|(i, p)| p.map(|p| (i, p)))
            .collect();
        items.sort_by(|a, b| a.1.y.total_cmp(&b.1.y));

        // Push labels down so they do not overlap, then back up if they
        // overflow the plot bottom.
        let mut ys: Vec<f64> = items.iter().map(|(_, p)| p.y).collect();
        for i in 1..ys.len() {
            ys[i] = ys[i].max(ys[i - 1] + h);
        }
        if let Some(&last) = ys.last() {
            let overflow = last - plot.bottom();
            if overflow > 0.0 {
                for y in &mut ys {
                    *y -= overflow;
                }
                for i in (0..ys.len().saturating_sub(1)).rev() {
                    ys[i] = ys[i].min(ys[i + 1] - h);
                }
            }
        }

        let mut g = Group::new("direct-labels");
        for ((i, p), y) in items.iter().zip(ys) {
            let entry = &self.legend[*i];
            let mut style = theme.legend.clone();
            style.color = entry.stroke.color.with_alpha(1.0);
            let pos = Point::new(p.x + theme.spacing.direct_label, y);
            g.push(TextNode::new(&entry.name, pos, style).baseline(Baseline::Middle));
        }
        scene.push(g);
    }

    /// Math-style axis labels: x at the right end of the x axis, y at the top
    /// of the y axis, flipping sides when the axis hugs the far edge.
    fn origin_axis_labels(&self, group: &mut Group) {
        let theme = self.theme();
        let gap = theme.spacing.tick_label;
        let plot = self.plot();
        let style = &theme.axis_label;
        let label = |show: bool, s: &Option<String>| s.clone().filter(|s| show && !s.is_empty());

        if let Some(t) = label(self.visibility.x_label, &self.base.x_axis.label) {
            let above = self.x_axis_y - gap - style.line_height() >= plot.top();
            let (y, baseline) = if above {
                (self.x_axis_y - gap, Baseline::Bottom)
            } else {
                (self.x_axis_y + gap, Baseline::Top)
            };
            group.push(
                TextNode::new(t, Point::new(plot.right(), y), style.clone())
                    .anchor(TextAnchor::End)
                    .baseline(baseline),
            );
        }
        if let Some(t) = label(self.visibility.y_label, &self.base.y_axis.label) {
            // Tick labels live on the left of the y axis, so the label goes on
            // the right unless the axis hugs the right edge.
            let right_side = self.y_axis_x < plot.left() + plot.width * 0.85;
            let (x, anchor) = if right_side {
                (self.y_axis_x + gap, TextAnchor::Start)
            } else {
                (self.y_axis_x - gap, TextAnchor::End)
            };
            group.push(TextNode::new(t, Point::new(x, plot.top()), style.clone()).anchor(anchor));
        }
    }

    fn zero_inside(&self, (lo, hi): (f64, f64)) -> bool {
        lo < 0.0 && hi > 0.0
    }

    fn on_x_axis(&self, y: f64) -> bool {
        self.base.axis_line_visible(&self.base.x_axis) && (y - self.x_axis_y).abs() < 0.5
    }

    fn on_y_axis(&self, x: f64) -> bool {
        self.base.axis_line_visible(&self.base.y_axis) && (x - self.y_axis_x).abs() < 0.5
    }

    fn axis_stroke(&self, through_zero: bool, color: Option<Color>) -> Stroke {
        let theme = self.theme();
        let mut stroke = if self.origin_axes && through_zero {
            theme.origin_axis_line.clone()
        } else {
            theme.axis_line.clone()
        };
        if let Some(c) = color {
            stroke.color = c;
        }
        stroke
    }
}

fn with_color(style: &TextStyle, color: Option<Color>) -> TextStyle {
    let mut s = style.clone();
    if let Some(c) = color {
        s.color = c;
    }
    s
}

fn line(a: Point, b: Point, stroke: &Stroke) -> PathNode {
    PathNode {
        path: Path::line(a, b),
        stroke: Some(stroke.clone()),
        fill: None,
        opacity: None,
    }
}

/// Aligns thin lines to the pixel grid so they render sharp.
fn crisp(v: f64, stroke: &Stroke) -> f64 {
    let w = stroke.width.round() as i64;
    if w % 2 == 1 {
        v.floor() + 0.5
    } else {
        v.round()
    }
}
