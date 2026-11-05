//! The interface every rendering backend implements.

use crate::scene::Scene;

/// Turns a [`Scene`] into a concrete output (SVG text, PNG bytes, ...).
pub trait Renderer {
    type Output;
    type Error;

    fn render(&self, scene: &Scene) -> Result<Self::Output, Self::Error>;
}
