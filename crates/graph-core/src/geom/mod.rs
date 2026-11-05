//! Primitive geometry types used throughout the pipeline.
//!
//! All coordinates are in pixels, with the origin at the top-left corner and
//! the y axis pointing down (screen convention).

mod point;
mod rect;

pub use point::{Point, Size};
pub use rect::{Insets, Rect};
