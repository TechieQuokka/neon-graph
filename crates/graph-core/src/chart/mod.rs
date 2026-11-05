//! Chart types. Each chart turns its data into a [`Scene`].
//!
//! Shared configuration and drawing (axes, grid, titles, legend) lives in
//! [`ChartBase`] and [`Frame`], so a new chart kind only has to draw its data.

#[macro_use]
mod common;
pub mod decimate;
mod line;
mod series;

pub use common::{ChartBase, Frame, LegendEntry, LegendMode};
pub use line::LineChart;
pub use series::{Series, SeriesStyle};

use crate::scene::Scene;
use crate::text::{EstimateMeasurer, TextMeasurer};

/// Anything that can be turned into a [`Scene`].
pub trait Chart {
    /// Builds the scene using a custom text measurer.
    fn build_scene_with(&self, measurer: &dyn TextMeasurer) -> Scene;

    /// Builds the scene using the default [`EstimateMeasurer`].
    fn build_scene(&self) -> Scene {
        self.build_scene_with(&EstimateMeasurer)
    }
}
