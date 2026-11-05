use super::TextMeasurer;
use crate::geom::Size;
use crate::style::TextStyle;

/// Fast, font-independent width estimation.
///
/// Uses per-character advance ratios typical for modern sans-serif fonts.
/// CJK (including Hangul) glyphs are full-width. Accurate to within a few
/// percent for typical labels, which is enough for margins and wrapping.
#[derive(Debug, Clone, Copy, Default)]
pub struct EstimateMeasurer;

impl EstimateMeasurer {
    /// Advance width of a character in `em` units.
    fn advance(ch: char) -> f64 {
        match ch {
            c if is_wide(c) => 1.0,
            '0'..='9' => 0.58,
            ' ' => 0.28,
            '.' | ',' | ':' | ';' | '\'' | '!' | '|' | 'i' | 'j' | 'l' => 0.28,
            '(' | ')' | '[' | ']' | '{' | '}' | '-' | 'f' | 't' | 'r' => 0.36,
            'm' | 'w' | 'M' | 'W' | '@' | '%' => 0.86,
            c if c.is_ascii_uppercase() => 0.66,
            c if c.is_ascii() => 0.54,
            _ => 0.62,
        }
    }
}

/// East Asian wide characters (Hangul, CJK ideographs, kana, full-width forms).
fn is_wide(c: char) -> bool {
    matches!(c as u32,
        0x1100..=0x115F       // Hangul Jamo (leading)
        | 0x2E80..=0x303E     // CJK radicals, punctuation
        | 0x3041..=0x33FF     // Kana, compatibility Jamo, CJK symbols
        | 0x3400..=0x4DBF     // CJK extension A
        | 0x4E00..=0x9FFF     // CJK unified ideographs
        | 0xA960..=0xA97F     // Hangul Jamo extended-A
        | 0xAC00..=0xD7A3     // Hangul syllables
        | 0xF900..=0xFAFF     // CJK compatibility ideographs
        | 0xFE30..=0xFE4F     // CJK compatibility forms
        | 0xFF00..=0xFF60     // Full-width forms
        | 0xFFE0..=0xFFE6
        | 0x20000..=0x3FFFD)
}

impl TextMeasurer for EstimateMeasurer {
    fn measure(&self, text: &str, style: &TextStyle) -> Size {
        let em: f64 = text.chars().map(Self::advance).sum();
        // Bold text is slightly wider.
        let weight_factor = 1.0 + (style.weight.saturating_sub(400) as f64 / 300.0) * 0.05;
        Size::new(em * style.size * weight_factor, style.line_height())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Color;

    #[test]
    fn hangul_is_full_width() {
        let style = TextStyle::new(10.0, 400, Color::BLACK);
        let m = EstimateMeasurer;
        assert_eq!(m.width("매출", &style), 20.0);
        assert!(m.width("ab", &style) < m.width("매출", &style));
        assert_eq!(m.width("", &style), 0.0);
    }

    #[test]
    fn bold_is_wider() {
        let m = EstimateMeasurer;
        let regular = TextStyle::new(10.0, 400, Color::BLACK);
        let bold = TextStyle::new(10.0, 700, Color::BLACK);
        assert!(m.width("Title", &bold) > m.width("Title", &regular));
    }
}
