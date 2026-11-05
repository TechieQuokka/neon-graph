//! Low-level helpers for writing compact SVG text.

use std::fmt::Write;

use graph_core::style::{Color, LineCap, LineJoin, Stroke};

/// Writes `value` with at most `precision` decimals, trimming trailing zeros.
pub fn num(out: &mut String, value: f64, precision: usize) {
    if !value.is_finite() {
        out.push('0');
        return;
    }
    let start = out.len();
    let _ = write!(out, "{value:.precision$}");
    if out[start..].contains('.') {
        while out.ends_with('0') {
            out.pop();
        }
        if out.ends_with('.') {
            out.pop();
        }
    }
    if &out[start..] == "-0" {
        out.truncate(start);
        out.push('0');
    }
}

/// Escapes text content and attribute values.
pub fn escape(out: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
}

/// ` name="value"` with a numeric value.
pub fn attr_num(out: &mut String, name: &str, value: f64, precision: usize) {
    out.push(' ');
    out.push_str(name);
    out.push_str("=\"");
    num(out, value, precision);
    out.push('"');
}

/// ` name="value"` with an escaped string value.
pub fn attr_str(out: &mut String, name: &str, value: &str) {
    out.push(' ');
    out.push_str(name);
    out.push_str("=\"");
    escape(out, value);
    out.push('"');
}

/// Writes a paint attribute (`fill` / `stroke`) plus its opacity if needed.
pub fn paint(out: &mut String, name: &str, color: Color, precision: usize) {
    if color.is_transparent() {
        attr_str(out, name, "none");
        return;
    }
    attr_str(out, name, &color.to_hex_rgb());
    if !color.is_opaque() {
        attr_num(
            out,
            &format!("{name}-opacity"),
            color.a as f64,
            precision.max(2),
        );
    }
}

pub fn stroke(out: &mut String, s: &Stroke, precision: usize) {
    paint(out, "stroke", s.color, precision);
    attr_num(out, "stroke-width", s.width, precision);
    if !s.dash.is_empty() {
        out.push_str(" stroke-dasharray=\"");
        for (i, d) in s.dash.iter().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            num(out, *d, precision);
        }
        out.push('"');
    }
    match s.cap {
        LineCap::Butt => {}
        LineCap::Round => attr_str(out, "stroke-linecap", "round"),
        LineCap::Square => attr_str(out, "stroke-linecap", "square"),
    }
    match s.join {
        LineJoin::Miter => {}
        LineJoin::Round => attr_str(out, "stroke-linejoin", "round"),
        LineJoin::Bevel => attr_str(out, "stroke-linejoin", "bevel"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(v: f64) -> String {
        let mut s = String::new();
        num(&mut s, v, 2);
        s
    }

    #[test]
    fn numbers_are_compact() {
        assert_eq!(n(10.0), "10");
        assert_eq!(n(10.5), "10.5");
        assert_eq!(n(10.256), "10.26");
        assert_eq!(n(-0.001), "0");
        assert_eq!(n(f64::NAN), "0");
    }

    #[test]
    fn escapes() {
        let mut s = String::new();
        escape(&mut s, "a<b & \"c\"");
        assert_eq!(s, "a&lt;b &amp; &quot;c&quot;");
    }
}
