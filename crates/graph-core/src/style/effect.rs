/// Neon-like glow around series lines.
///
/// The original line stays crisp; a blurred copy is drawn behind it.
/// Glow is only meaningful on dark backgrounds.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Glow {
    pub enabled: bool,
    /// Blur radius (standard deviation) in pixels.
    pub radius: f64,
    /// Opacity of the blurred halo (`0.0..=1.0`).
    pub intensity: f64,
}

impl Glow {
    pub const OFF: Glow = Glow {
        enabled: false,
        radius: 3.0,
        intensity: 0.8,
    };

    pub const fn new(radius: f64, intensity: f64) -> Self {
        Self {
            enabled: true,
            radius,
            intensity,
        }
    }
}

impl Default for Glow {
    fn default() -> Self {
        Self::OFF
    }
}
