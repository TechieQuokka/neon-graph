use std::fmt;
use std::str::FromStr;

/// An sRGB color with a floating-point alpha channel (`0.0..=1.0`).
///
/// Colors can be created from components, from a packed `0xRRGGBB` integer,
/// or parsed from CSS-like strings:
///
/// ```
/// use graph_core::style::Color;
///
/// let a = Color::rgb(30, 136, 229);
/// let b = Color::hex("#1e88e5");
/// let c: Color = "rgba(30, 136, 229, 0.5)".parse().unwrap();
/// assert_eq!(a, b);
/// assert_eq!(c.a, 0.5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl Color {
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const TRANSPARENT: Color = Color::rgba(0, 0, 0, 0.0);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Builds an opaque color from a packed `0xRRGGBB` value.
    pub const fn from_u32(rgb: u32) -> Self {
        Self::rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
    }

    /// Parses a color literal such as `"#1e88e5"`.
    ///
    /// Intended for literals in source code. Use [`str::parse`] /
    /// [`Color::from_str`] for untrusted input.
    ///
    /// # Panics
    ///
    /// Panics if the string is not a valid color.
    #[track_caller]
    pub fn hex(s: &str) -> Self {
        match s.parse() {
            Ok(c) => c,
            Err(e) => panic!("invalid color literal {s:?}: {e}"),
        }
    }

    /// Returns the same color with a different alpha.
    pub const fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }

    pub fn is_opaque(self) -> bool {
        self.a >= 1.0
    }

    pub fn is_transparent(self) -> bool {
        self.a <= 0.0
    }

    /// `#rrggbb` without the alpha channel.
    pub fn to_hex_rgb(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

impl fmt::Display for Color {
    /// Formats as `#rrggbb`, or `#rrggbbaa` when the alpha fits a byte
    /// exactly, otherwise `rgba(r, g, b, a)` so parsing it back is lossless.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_opaque() {
            return write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b);
        }
        let a = self.a.clamp(0.0, 1.0);
        let byte = (a * 255.0).round();
        if byte / 255.0 == a {
            write!(
                f,
                "#{:02x}{:02x}{:02x}{:02x}",
                self.r, self.g, self.b, byte as u8
            )
        } else {
            write!(f, "rgba({}, {}, {}, {a})", self.r, self.g, self.b)
        }
    }
}

/// Error returned when parsing a [`Color`] fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorParseError {
    input: String,
}

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cannot parse color {:?} (expected #rgb, #rgba, #rrggbb, #rrggbbaa, rgb(..), rgba(..) or a basic name)",
            self.input
        )
    }
}

impl std::error::Error for ColorParseError {}

impl FromStr for Color {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || ColorParseError {
            input: s.to_owned(),
        };
        let t = s.trim();
        let lower = t.to_ascii_lowercase();

        if let Some(hex) = lower.strip_prefix('#') {
            return parse_hex(hex).ok_or_else(err);
        }
        if let Some(inner) = strip_fn(&lower, "rgba").or_else(|| strip_fn(&lower, "rgb")) {
            return parse_rgb_fn(inner).ok_or_else(err);
        }
        named(&lower).ok_or_else(err)
    }
}

fn strip_fn<'a>(s: &'a str, name: &str) -> Option<&'a str> {
    s.strip_prefix(name)?
        .trim_start()
        .strip_prefix('(')?
        .strip_suffix(')')
}

fn parse_hex(hex: &str) -> Option<Color> {
    let nibble = |c: u8| (c as char).to_digit(16).map(|d| d as u8);
    let bytes = hex.as_bytes();
    let expand = |i: usize| nibble(bytes[i]).map(|n| n * 17);
    let pair = |i: usize| Some(nibble(bytes[i])? * 16 + nibble(bytes[i + 1])?);

    match bytes.len() {
        3 => Some(Color::rgb(expand(0)?, expand(1)?, expand(2)?)),
        4 => Some(Color::rgba(
            expand(0)?,
            expand(1)?,
            expand(2)?,
            expand(3)? as f32 / 255.0,
        )),
        6 => Some(Color::rgb(pair(0)?, pair(2)?, pair(4)?)),
        8 => Some(Color::rgba(
            pair(0)?,
            pair(2)?,
            pair(4)?,
            pair(6)? as f32 / 255.0,
        )),
        _ => None,
    }
}

fn parse_rgb_fn(inner: &str) -> Option<Color> {
    let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
    let channel = |s: &str| s.parse::<u8>().ok();
    match parts.as_slice() {
        [r, g, b] => Some(Color::rgb(channel(r)?, channel(g)?, channel(b)?)),
        [r, g, b, a] => {
            let a = a.parse::<f32>().ok().filter(|a| (0.0..=1.0).contains(a))?;
            Some(Color::rgba(channel(r)?, channel(g)?, channel(b)?, a))
        }
        _ => None,
    }
}

fn named(name: &str) -> Option<Color> {
    Some(match name {
        "black" => Color::BLACK,
        "white" => Color::WHITE,
        "transparent" | "none" => Color::TRANSPARENT,
        "red" => Color::rgb(255, 0, 0),
        "green" => Color::rgb(0, 128, 0),
        "blue" => Color::rgb(0, 0, 255),
        "gray" | "grey" => Color::rgb(128, 128, 128),
        _ => return None,
    })
}

#[cfg(feature = "serde")]
impl serde::Serialize for Color {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Color {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_forms() {
        assert_eq!("#fff".parse::<Color>().unwrap(), Color::WHITE);
        assert_eq!(
            "#1E88E5".parse::<Color>().unwrap(),
            Color::rgb(30, 136, 229)
        );
        let c: Color = "#00000080".parse().unwrap();
        assert!((c.a - 128.0 / 255.0).abs() < 1e-6);
        assert!("#12345".parse::<Color>().is_err());
        assert!("#gggggg".parse::<Color>().is_err());
    }

    #[test]
    fn parses_rgb_functions_and_names() {
        assert_eq!(
            "rgb(1, 2, 3)".parse::<Color>().unwrap(),
            Color::rgb(1, 2, 3)
        );
        assert_eq!(
            "RGBA(1,2,3,0.25)".parse::<Color>().unwrap(),
            Color::rgba(1, 2, 3, 0.25)
        );
        assert!("rgba(1,2,3,2)".parse::<Color>().is_err());
        assert_eq!("transparent".parse::<Color>().unwrap(), Color::TRANSPARENT);
        assert!("chartreuse-ish".parse::<Color>().is_err());
    }

    #[test]
    fn display_roundtrip() {
        let c = Color::rgba(10, 20, 30, 0.5);
        let back: Color = c.to_string().parse().unwrap();
        assert_eq!(back, c);
        let exact = Color::rgba(1, 2, 3, 128.0 / 255.0);
        assert_eq!(exact.to_string(), "#01020380");
        assert_eq!(exact.to_string().parse::<Color>().unwrap(), exact);
        assert_eq!(Color::from_u32(0x00f0ff).to_string(), "#00f0ff");
    }
}
