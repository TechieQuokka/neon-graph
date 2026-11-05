use super::{GridStyle, SeriesDefaults, Spacing, Theme};
use crate::geom::Insets;
use crate::style::{Color, Glow, Palette, Stroke, TextStyle};

/// Neon lines on a deep navy background.
///
/// Readability rules: neon colors are reserved for data; text stays in bright
/// neutral tones so Hangul never looks blurry, and grid lines are barely visible.
pub(super) fn theme() -> Theme {
    let c = Color::from_u32;
    Theme {
        name: "cyberpunk".into(),
        background: c(0x0a0e17),
        plot_background: c(0x0d1321),

        title: TextStyle::new(20.0, 700, c(0xe6f1ff)),
        subtitle: TextStyle::new(14.0, 400, c(0x8892b0)),
        axis_label: TextStyle::new(13.0, 500, c(0xa8b2d1)),
        tick_label: TextStyle::new(12.0, 400, c(0x6b7a99)),
        legend: TextStyle::new(13.0, 400, c(0xccd6f6)),

        axis_line: Stroke::new(c(0x2a3550), 1.0),
        origin_axis_line: Stroke::new(c(0x00f0ff).with_alpha(0.7), 1.5),
        tick: Stroke::new(c(0x2a3550), 1.0),
        tick_length: 5.0,

        grid_x: GridStyle {
            visible: true,
            stroke: Stroke::new(c(0x1a2236), 1.0),
        },
        grid_y: GridStyle {
            visible: true,
            stroke: Stroke::new(c(0x1a2236), 1.0),
        },

        series: SeriesDefaults {
            palette: Palette::new([
                c(0x00f0ff), // cyan
                c(0xff2a6d), // hot pink
                c(0xf9f871), // electric yellow
                c(0x05ffa1), // neon green
                c(0xb967ff), // purple
                c(0xff9f1c), // orange
            ]),
            width: 2.0,
            markers: false,
            marker_radius: 3.0,
            opacity: 1.0,
        },
        glow: Glow::new(3.0, 0.8),

        padding: Insets::uniform(24.0),
        spacing: Spacing::default(),
    }
}
