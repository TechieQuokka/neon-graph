//! SVG filter definitions for scene effects.

use graph_core::geom::Size;
use graph_core::style::Glow;

use crate::writer::{attr_num, attr_str};

/// Writes a glow filter: a blurred, slightly transparent copy of the source
/// merged under the crisp original.
///
/// The filter region uses user-space units covering the whole canvas, so
/// perfectly horizontal or vertical lines (zero-height bounding boxes) still
/// glow.
pub fn glow(out: &mut String, id: &str, glow: &Glow, canvas: Size, precision: usize) {
    out.push_str("<filter");
    attr_str(out, "id", id);
    attr_str(out, "filterUnits", "userSpaceOnUse");
    attr_num(out, "x", 0.0, precision);
    attr_num(out, "y", 0.0, precision);
    attr_num(out, "width", canvas.width, precision);
    attr_num(out, "height", canvas.height, precision);
    out.push('>');

    out.push_str("<feGaussianBlur in=\"SourceGraphic\"");
    attr_num(out, "stdDeviation", glow.radius.max(0.0), precision);
    out.push_str(" result=\"blur\"/>");

    out.push_str("<feComponentTransfer in=\"blur\" result=\"halo\"><feFuncA type=\"linear\"");
    attr_num(
        out,
        "slope",
        glow.intensity.clamp(0.0, 1.0),
        precision.max(2),
    );
    out.push_str("/></feComponentTransfer>");

    out.push_str(
        "<feMerge><feMergeNode in=\"halo\"/><feMergeNode in=\"SourceGraphic\"/></feMerge></filter>",
    );
}
