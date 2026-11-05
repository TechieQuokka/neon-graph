//! Text measurement and per-element text visibility.
//!
//! Layout needs text sizes (e.g. the widest tick label decides the left
//! margin). [`TextMeasurer`] abstracts this so an exact, font-file based
//! implementation can replace [`EstimateMeasurer`] later.

mod estimate;
mod visibility;

pub use estimate::EstimateMeasurer;
pub use visibility::{TextElement, TextElementParseError, TextPreset, TextVisibility};

use crate::geom::Size;
use crate::style::TextStyle;

/// Measures the rendered size of a single line of text.
pub trait TextMeasurer {
    fn measure(&self, text: &str, style: &TextStyle) -> Size;

    fn width(&self, text: &str, style: &TextStyle) -> f64 {
        self.measure(text, style).width
    }
}

/// Width of the widest string, `0.0` for none.
pub fn max_width<'a>(
    measurer: &dyn TextMeasurer,
    texts: impl IntoIterator<Item = &'a str>,
    style: &TextStyle,
) -> f64 {
    texts
        .into_iter()
        .map(|t| measurer.width(t, style))
        .fold(0.0, f64::max)
}
