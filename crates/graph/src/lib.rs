//! # graph
//!
//! Simple, fast and good-looking charts with first-class Korean text support.
//!
//! ```
//! use graph::prelude::*;
//!
//! let svg = LineChart::new()
//!     .title("월별 매출")
//!     .x_label("월")
//!     .y_label("금액")
//!     .series("매출", [1.0, 2.0, 3.0, 4.0], [100.0, 120.0, 90.0, 150.0])
//!     .series("비용", [1.0, 2.0, 3.0, 4.0], [80.0, 95.0, 70.0, 110.0])
//!     .to_svg();
//! assert!(svg.contains("월별 매출"));
//! ```
//!
//! ## Crate layout
//!
//! * `graph-core` – engine: scales, ticks, viewport, layout, scene, charts.
//! * `graph-svg` – SVG backend (feature `svg`, on by default).
//! * this crate – the public facade and [`prelude`].

pub use graph_core::{axis, chart, geom, layout, render, scale, scene, style, text, viewport};

#[cfg(feature = "svg")]
pub use graph_svg as svg;

/// Everything needed for everyday use.
pub mod prelude {
    pub use graph_core::axis::{Axis, AxisPosition, NumberFormat};
    pub use graph_core::chart::{Chart, LegendMode, LineChart, Series, SeriesStyle};
    pub use graph_core::geom::{Insets, Size};
    pub use graph_core::style::{Color, Glow, Palette, Stroke, TextStyle, Theme};
    pub use graph_core::text::{TextElement, TextPreset};
    pub use graph_core::viewport::{Quadrant, Quadrants};

    #[cfg(feature = "svg")]
    pub use crate::ChartSvgExt;
}

/// Convenience methods to render any [`chart::Chart`] straight to SVG.
#[cfg(feature = "svg")]
pub trait ChartSvgExt: chart::Chart {
    /// Renders the chart to an SVG string.
    fn to_svg(&self) -> String {
        graph_svg::to_svg(&self.build_scene())
    }

    /// Renders the chart and writes it to `path`.
    fn save_svg(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        std::fs::write(path, self.to_svg())
    }
}

#[cfg(feature = "svg")]
impl<T: chart::Chart + ?Sized> ChartSvgExt for T {}
