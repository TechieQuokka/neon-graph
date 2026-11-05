//! Everything visual: colors, strokes, fonts, palettes, effects and themes.
//!
//! Nothing in the rendering pipeline is hard-coded; every color and size comes
//! from a [`Theme`], which can be overridden at chart, element and series level.

mod color;
mod effect;
mod font;
mod palette;
mod stroke;
pub mod theme;

pub use color::{Color, ColorParseError};
pub use effect::Glow;
pub use font::{DEFAULT_FONT_FAMILY, TextStyle};
pub use palette::Palette;
pub use stroke::{LineCap, LineJoin, Stroke};
pub use theme::{GridStyle, SeriesDefaults, Spacing, Theme};
