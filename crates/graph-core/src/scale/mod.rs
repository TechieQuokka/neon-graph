//! Scales map data values (domain) to pixel positions (range).
//!
//! Only [`LinearScale`] exists for now; log, time and band scales can be added
//! as new implementations of [`Scale`].

mod linear;

pub use linear::LinearScale;

/// A mapping from a data domain to a pixel range.
pub trait Scale {
    /// Data value -> pixel position.
    fn map(&self, value: f64) -> f64;
    /// Pixel position -> data value.
    fn invert(&self, pixel: f64) -> f64;
    /// `(start, end)` of the data domain.
    fn domain(&self) -> (f64, f64);
    /// `(start, end)` of the pixel range.
    fn range(&self) -> (f64, f64);

    /// Returns `true` when `value` lies within the domain (inclusive).
    fn contains(&self, value: f64) -> bool {
        let (a, b) = self.domain();
        value >= a.min(b) && value <= a.max(b)
    }
}
