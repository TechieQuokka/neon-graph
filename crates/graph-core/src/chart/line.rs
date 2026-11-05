use super::common::{ChartBase, Frame, LegendEntry};
use super::decimate::m4;
use super::{Chart, Series, SeriesStyle};
use crate::geom::{Insets, Point};
use crate::scene::{CircleNode, Effect, Group, Path, PathNode, Scene};
use crate::style::Glow;
use crate::text::TextMeasurer;
use crate::viewport::DataBounds;

/// Decimation kicks in above this many points per horizontal pixel.
const DECIMATE_RATIO: f64 = 4.0;

/// A line chart with any number of series.
///
/// ```
/// use graph_core::chart::{Chart, LineChart};
///
/// let chart = LineChart::new()
///     .title("월별 매출")
///     .x_label("월")
///     .y_label("금액")
///     .series("매출", [1.0, 2.0, 3.0], [100.0, 120.0, 90.0])
///     .series("비용", [1.0, 2.0, 3.0], [80.0, 95.0, 70.0]);
/// let scene = chart.build_scene();
/// assert!(scene.texts().contains(&"월별 매출"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct LineChart {
    base: ChartBase,
    series: Vec<Series>,
}

impl_common_builders!(LineChart);

impl LineChart {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a series from x and y values.
    pub fn series(
        mut self,
        name: impl Into<String>,
        xs: impl AsRef<[f64]>,
        ys: impl AsRef<[f64]>,
    ) -> Self {
        self.series.push(Series::new(name, xs, ys));
        self
    }

    /// Adds a series with x values `1, 2, 3, ...`.
    pub fn series_y(mut self, name: impl Into<String>, ys: impl AsRef<[f64]>) -> Self {
        self.series.push(Series::from_y(name, ys));
        self
    }

    /// Adds a series and styles it in one go.
    pub fn series_with(
        mut self,
        name: impl Into<String>,
        xs: impl AsRef<[f64]>,
        ys: impl AsRef<[f64]>,
        style: impl FnOnce(SeriesStyle) -> SeriesStyle,
    ) -> Self {
        let mut s = Series::new(name, xs, ys);
        s.style = style(s.style);
        self.series.push(s);
        self
    }

    /// Adds a prebuilt series.
    pub fn push_series(mut self, series: Series) -> Self {
        self.series.push(series);
        self
    }

    /// Restyles every series with the given name.
    pub fn series_style(mut self, name: &str, style: impl Fn(SeriesStyle) -> SeriesStyle) -> Self {
        for s in self.series.iter_mut().filter(|s| s.name == name) {
            s.style = style(std::mem::take(&mut s.style));
        }
        self
    }

    pub fn get_series(&self) -> &[Series] {
        &self.series
    }

    /// How far drawing reaches beyond the data points: half the widest line
    /// (or the biggest marker), plus the glow halo.
    fn bleed(&self, glow_on: bool) -> f64 {
        let theme = &self.base.theme;
        let mut reach: f64 = 0.0;
        let mut glows = false;
        for s in &self.series {
            let half_width = s.style.width.unwrap_or(theme.series.width) / 2.0;
            let marker = if s.markers(theme) {
                s.marker_radius(theme)
            } else {
                0.0
            };
            reach = reach.max(half_width).max(marker);
            glows |= s.style.glow.unwrap_or(glow_on);
        }
        let halo = if glows { theme.glow.radius * 2.0 } else { 0.0 };
        (reach + halo + 1.0).ceil()
    }

    fn data_bounds(&self) -> DataBounds {
        DataBounds::from_points(self.series.iter().flat_map(Series::points))
    }

    /// Splits a series into continuous runs of finite points, in pixel space.
    fn segments(&self, frame: &Frame<'_>, series: &Series) -> Vec<Vec<Point>> {
        let mut segments = Vec::new();
        let mut current = Vec::new();
        for (x, y) in series.points() {
            if x.is_finite() && y.is_finite() {
                current.push(frame.map(x, y));
            } else if !current.is_empty() {
                segments.push(std::mem::take(&mut current));
            }
        }
        if !current.is_empty() {
            segments.push(current);
        }

        let limit = frame.plot().width * DECIMATE_RATIO;
        for seg in &mut segments {
            if seg.len() as f64 > limit
                && let Some(reduced) = m4(seg)
            {
                *seg = reduced;
            }
        }
        segments
    }
}

impl Chart for LineChart {
    fn build_scene_with(&self, measurer: &dyn TextMeasurer) -> Scene {
        let theme = &self.base.theme;
        let legend: Vec<LegendEntry> = self
            .series
            .iter()
            .enumerate()
            .map(|(i, s)| LegendEntry {
                name: s.name.clone(),
                stroke: s.stroke(i, theme),
            })
            .collect();

        let glow_on = self.base.glow_enabled();
        let bleed = self.bleed(glow_on);
        let frame = Frame::new(&self.base, &self.data_bounds(), legend, bleed, measurer);
        let mut scene = Scene::new(self.base.size);
        frame.draw_under(&mut scene);

        // Let line caps, markers and glow at the domain edges show fully;
        // anything further out is clipped.
        let clip = scene.add_clip(frame.plot().inset(Insets::uniform(-bleed)));

        let glow = Glow {
            enabled: true,
            ..theme.glow
        };
        let mut glowing = Group {
            effect: Some(Effect::Glow(glow)),
            ..Group::new("series-glow")
        };
        let mut plain = Group::new("series");
        let mut anchors = Vec::with_capacity(self.series.len());

        for (i, series) in self.series.iter().enumerate() {
            let stroke = series.stroke(i, theme);
            let segments = self.segments(&frame, series);
            let target = if series.style.glow.unwrap_or(glow_on) {
                &mut glowing
            } else {
                &mut plain
            };

            let markers = series.markers(theme);
            let radius = series.marker_radius(theme);
            let mut series_group = Group::new("line");

            let mut path = Path::builder();
            for seg in &segments {
                if seg.len() == 1 && !markers {
                    // A lone point has no line; show it as a dot.
                    series_group.push(dot(seg[0], radius.max(stroke.width), stroke.color));
                    continue;
                }
                path.move_to(seg[0]);
                for p in &seg[1..] {
                    path.line_to(*p);
                }
            }
            let path = path.build();
            if !path.is_empty() {
                series_group.push(PathNode {
                    path,
                    stroke: Some(stroke.clone()),
                    fill: None,
                    opacity: None,
                });
            }
            if markers {
                for p in segments.iter().flatten() {
                    series_group.push(dot(*p, radius, stroke.color));
                }
            }
            if !series_group.is_empty() {
                target.push(series_group);
            }

            anchors.push(
                segments
                    .last()
                    .and_then(|s| s.last())
                    .copied()
                    .filter(|p| frame.plot().contains(*p)),
            );
        }

        let mut data = Group {
            clip: Some(clip),
            ..Group::new("data")
        };
        if !plain.is_empty() {
            data.push(plain);
        }
        if !glowing.is_empty() {
            data.push(glowing);
        }
        scene.push(data);

        frame.draw_over(&mut scene);
        frame.draw_direct_labels(&mut scene, &anchors);
        scene
    }
}

fn dot(center: Point, radius: f64, color: crate::style::Color) -> CircleNode {
    CircleNode {
        center,
        radius,
        fill: Some(color),
        stroke: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Node;
    use crate::text::{TextElement, TextPreset};
    use crate::viewport::Quadrant;

    fn sample() -> LineChart {
        LineChart::new()
            .title("월별 매출")
            .subtitle("2026년")
            .x_label("월")
            .y_label("금액")
            .series("매출", [1.0, 2.0, 3.0, 4.0], [100.0, 120.0, 90.0, 150.0])
            .series("비용", [1.0, 2.0, 3.0, 4.0], [80.0, 95.0, 70.0, 110.0])
    }

    fn count(scene: &Scene, pred: impl Fn(&Node) -> bool) -> usize {
        let mut n = 0;
        scene.walk(&mut |node| {
            if pred(node) {
                n += 1;
            }
        });
        n
    }

    #[test]
    fn full_chart_has_all_texts() {
        let scene = sample().build_scene();
        let texts = scene.texts();
        for t in ["월별 매출", "2026년", "월", "금액", "매출", "비용"] {
            assert!(texts.contains(&t), "missing {t}: {texts:?}");
        }
        assert_eq!(
            count(
                &scene,
                |n| matches!(n, Node::Path(p) if p.path.commands().len() == 4)
            ),
            2
        );
    }

    #[test]
    fn minimal_preset_has_no_text() {
        let scene = sample().text_preset(TextPreset::Minimal).build_scene();
        assert!(scene.texts().is_empty());
    }

    #[test]
    fn partial_override_on_preset() {
        let scene = sample()
            .text_preset(TextPreset::Minimal)
            .show(TextElement::Legend, true)
            .build_scene();
        assert_eq!(scene.texts(), vec!["매출", "비용"]);
    }

    /// Plot area as seen by the data clip (plot rect grown by the bleed).
    fn plot_area(scene: &Scene) -> f64 {
        let (_, r) = scene.clips().next().expect("data clip");
        r.width * r.height
    }

    #[test]
    fn hiding_text_gives_space_to_the_plot() {
        use TextElement::*;
        // Hide one element at a time; the plot must grow at every step.
        let steps = [
            Title,
            Subtitle,
            Legend,
            YLabel,
            XLabel,
            XTickLabels,
            YTickLabels,
        ];
        let mut chart = sample();
        let mut prev = plot_area(&chart.build_scene());
        for el in steps {
            chart = chart.show(el, false);
            let area = plot_area(&chart.build_scene());
            assert!(
                area > prev,
                "hiding {el} did not enlarge the plot ({prev} -> {area})"
            );
            prev = area;
        }
    }

    #[test]
    fn bare_mode_draws_only_data() {
        let scene = sample()
            .theme(crate::style::Theme::cyberpunk())
            .bare()
            .build_scene();
        assert!(scene.texts().is_empty());

        let mut roles = Vec::new();
        scene.walk(&mut |n| {
            if let Node::Group(g) = n {
                roles.push(g.role.unwrap_or(""));
            }
        });
        for unwanted in ["grid", "axes", "tick-labels", "legend", "text"] {
            assert!(
                !roles.contains(&unwanted),
                "bare scene has {unwanted}: {roles:?}"
            );
        }
        // Background and glow are kept.
        assert!(scene.background.is_some());
        assert!(roles.contains(&"series-glow"));

        // The data fills the canvas except for the bleed (line caps + glow).
        let (_, clip) = scene.clips().next().unwrap();
        assert_eq!((clip.x, clip.y), (0.0, 0.0));
        assert_eq!((clip.width, clip.height), (800.0, 500.0));
    }

    #[test]
    fn bare_mode_fits_data_tightly() {
        let chart = sample().bare();
        let frame = Frame::new(
            chart.base(),
            &chart.data_bounds(),
            Vec::new(),
            0.0,
            &crate::text::EstimateMeasurer,
        );
        assert_eq!(frame.viewport.y, (70.0, 150.0));
        assert_eq!(frame.viewport.x, (1.0, 4.0));
        // Opting back in to nice bounds still works.
        let chart = sample().bare().y_axis(|a| a.nice(true).max_ticks(2));
        let frame = Frame::new(
            chart.base(),
            &chart.data_bounds(),
            Vec::new(),
            0.0,
            &crate::text::EstimateMeasurer,
        );
        assert_eq!(frame.viewport.y, (50.0, 150.0));
    }

    fn has_role(scene: &Scene, role: &str) -> bool {
        let mut found = false;
        scene.walk(&mut |n| {
            if let Node::Group(g) = n {
                found |= g.role == Some(role);
            }
        });
        found
    }

    #[test]
    fn grid_option() {
        use crate::style::Theme;
        // Default: theme decides; bare hides it.
        assert!(has_role(&sample().build_scene(), "grid"));
        assert!(!has_role(&sample().bare().build_scene(), "grid"));
        // Explicit option wins in both modes, in any call order.
        assert!(has_role(&sample().bare().grid(true).build_scene(), "grid"));
        assert!(has_role(&sample().grid(true).bare().build_scene(), "grid"));
        assert!(!has_role(&sample().grid(false).build_scene(), "grid"));
        // Directions follow the theme: cyberpunk draws both.
        let chart = sample().theme(Theme::cyberpunk()).bare().grid(true);
        assert!(chart.base().grid_x_visible() && chart.base().grid_y_visible());
        // Forced on with a theme that disables both: fall back to horizontal.
        let chart = sample()
            .theme_with(|t| {
                t.grid_x.visible = false;
                t.grid_y.visible = false;
            })
            .grid(true);
        assert!(!chart.base().grid_x_visible() && chart.base().grid_y_visible());
    }

    #[test]
    fn bare_mode_respects_explicit_overrides() {
        let scene = sample().bare().show_legend(true).build_scene();
        assert_eq!(scene.texts(), vec!["매출", "비용"]);
    }

    #[test]
    fn nan_creates_gap() {
        let scene = LineChart::new()
            .series(
                "a",
                [1.0, 2.0, 3.0, 4.0, 5.0],
                [1.0, 2.0, f64::NAN, 4.0, 5.0],
            )
            .show_legend(false)
            .build_scene();
        let moves = {
            let mut n = 0;
            scene.walk(&mut |node| {
                if let Node::Path(p) = node
                    && p.stroke.as_ref().is_some_and(|s| s.width == 2.0)
                {
                    n += p
                        .path
                        .commands()
                        .iter()
                        .filter(|c| matches!(c, crate::scene::PathCmd::MoveTo(_)))
                        .count();
                }
            });
            n
        };
        assert_eq!(moves, 2);
    }

    #[test]
    fn huge_series_is_decimated() {
        let xs: Vec<f64> = (0..200_000).map(|i| i as f64).collect();
        let ys: Vec<f64> = xs.iter().map(|x| (x / 1000.0).sin()).collect();
        let scene = LineChart::new().series("big", &xs, &ys).build_scene();
        let mut max_cmds = 0;
        scene.walk(&mut |n| {
            if let Node::Path(p) = n {
                max_cmds = max_cmds.max(p.path.commands().len());
            }
        });
        assert!(max_cmds < 4 * 800 + 10, "not decimated: {max_cmds}");
    }

    #[test]
    fn quadrant_mode_uses_origin_axes() {
        let chart = sample().quadrants(Quadrant::Q1);
        assert!(chart.base().origin_axes());
        let chart = sample().origin(0.5, 0.5);
        assert!(chart.base().quadrants.is_none());
        let scene = chart.build_scene();
        // Shared zero label printed once.
        assert_eq!(scene.texts().iter().filter(|t| **t == "0").count(), 1);
    }

    #[test]
    fn direct_labels_replace_top_legend() {
        let scene = sample()
            .legend_mode(crate::chart::LegendMode::Direct)
            .build_scene();
        let mut legend_groups = 0;
        let mut direct_groups = 0;
        scene.walk(&mut |n| {
            if let Node::Group(g) = n {
                match g.role {
                    Some("legend") => legend_groups += 1,
                    Some("direct-labels") => direct_groups += 1,
                    _ => {}
                }
            }
        });
        assert_eq!((legend_groups, direct_groups), (0, 1));
    }

    #[test]
    fn glow_group_follows_theme() {
        let has_glow = |c: LineChart| {
            let scene = c.build_scene();
            let mut found = false;
            scene.walk(&mut |n| {
                if let Node::Group(g) = n {
                    found |= g.effect.is_some() && !g.is_empty();
                }
            });
            found
        };
        assert!(!has_glow(sample()));
        assert!(has_glow(sample().theme(crate::style::Theme::cyberpunk())));
        assert!(!has_glow(
            sample().theme(crate::style::Theme::cyberpunk()).glow(false)
        ));
    }
}
