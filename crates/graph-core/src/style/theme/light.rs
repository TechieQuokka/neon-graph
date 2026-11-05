use super::{GridStyle, SeriesDefaults, Spacing, Theme};
use crate::geom::Insets;
use crate::style::{Color, Glow, Palette, Stroke, TextStyle};

pub(super) fn theme() -> Theme {
    let c = Color::from_u32;
    Theme {
        name: "light".into(),
        background: c(0xffffff),
        plot_background: Color::TRANSPARENT,

        title: TextStyle::new(20.0, 700, c(0x1a1a1a)),
        subtitle: TextStyle::new(14.0, 400, c(0x666666)),
        axis_label: TextStyle::new(13.0, 500, c(0x444444)),
        tick_label: TextStyle::new(12.0, 400, c(0x777777)),
        legend: TextStyle::new(13.0, 400, c(0x333333)),

        axis_line: Stroke::new(c(0x999999), 1.0),
        origin_axis_line: Stroke::new(c(0x555555), 1.5),
        tick: Stroke::new(c(0x999999), 1.0),
        tick_length: 5.0,

        grid_x: GridStyle {
            visible: false,
            stroke: Stroke::new(c(0xebebeb), 1.0),
        },
        grid_y: GridStyle {
            visible: true,
            stroke: Stroke::new(c(0xebebeb), 1.0),
        },

        // Okabe-Ito based, color-blind friendly, tuned for white backgrounds.
        series: SeriesDefaults {
            palette: Palette::new([
                c(0x0072b2),
                c(0xe69f00),
                c(0x009e73),
                c(0xd55e00),
                c(0xcc79a7),
                c(0x56b4e9),
                c(0xb8a000),
                c(0x555555),
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
