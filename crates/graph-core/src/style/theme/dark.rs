use super::{GridStyle, SeriesDefaults, Spacing, Theme};
use crate::geom::Insets;
use crate::style::{Color, Glow, Palette, Stroke, TextStyle};

pub(super) fn theme() -> Theme {
    let c = Color::from_u32;
    Theme {
        name: "dark".into(),
        background: c(0x121418),
        plot_background: Color::TRANSPARENT,

        title: TextStyle::new(20.0, 700, c(0xf1f3f5)),
        subtitle: TextStyle::new(14.0, 400, c(0x9aa0a6)),
        axis_label: TextStyle::new(13.0, 500, c(0xc1c7cd)),
        tick_label: TextStyle::new(12.0, 400, c(0x8a9099)),
        legend: TextStyle::new(13.0, 400, c(0xd0d4d9)),

        axis_line: Stroke::new(c(0x3a3f47), 1.0),
        origin_axis_line: Stroke::new(c(0x8a9099), 1.5),
        tick: Stroke::new(c(0x3a3f47), 1.0),
        tick_length: 5.0,

        grid_x: GridStyle {
            visible: false,
            stroke: Stroke::new(c(0x23272e), 1.0),
        },
        grid_y: GridStyle {
            visible: true,
            stroke: Stroke::new(c(0x23272e), 1.0),
        },

        series: SeriesDefaults {
            palette: Palette::new([
                c(0x56b4e9),
                c(0xf0a830),
                c(0x3cc79a),
                c(0xff7a45),
                c(0xe08cc0),
                c(0x9ad0ff),
                c(0xf0e442),
                c(0xbbbbbb),
            ]),
            width: 2.0,
            markers: false,
            marker_radius: 3.0,
            opacity: 1.0,
        },
        glow: Glow::OFF,

        padding: Insets::uniform(24.0),
        spacing: Spacing::default(),
    }
}
