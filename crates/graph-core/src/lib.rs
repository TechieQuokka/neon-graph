//! Core engine of the `graph` library.
//!
//! The pipeline is split into independent stages so that new chart kinds and
//! new rendering backends can be added without touching each other:
//!
//! ```text
//! Data -> Chart (Line, ...) -> Layout -> Scene -> Renderer (SVG, ...)
//!           ^                   ^          ^
//!         Theme            TextMeasurer  backend-agnostic primitives
//! ```
//!
//! This crate has no required dependencies. Enable the `serde` feature to
//! (de)serialize [`style::Theme`] and related types.

pub mod axis;
pub mod chart;
pub mod geom;
pub mod layout;
pub mod render;
pub mod scale;
pub mod scene;
pub mod style;
pub mod text;
pub mod viewport;
