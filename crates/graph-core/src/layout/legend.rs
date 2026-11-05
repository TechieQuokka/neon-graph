use crate::geom::{Point, Size};
use crate::style::{Spacing, TextStyle};
use crate::text::TextMeasurer;

/// Position of one legend entry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegendItem {
    /// Left end of the color swatch, vertically centered on the row.
    pub swatch: Point,
    /// Left end of the label, vertically centered on the row.
    pub label: Point,
}

/// A horizontal, wrapping legend.
#[derive(Debug, Clone, PartialEq)]
pub struct LegendLayout {
    pub items: Vec<LegendItem>,
    pub size: Size,
}

/// Lays out legend entries left-to-right starting at `origin` (top-left),
/// wrapping to a new row when `max_width` would be exceeded.
pub fn layout_legend(
    names: &[&str],
    origin: Point,
    max_width: f64,
    style: &TextStyle,
    spacing: &Spacing,
    measurer: &dyn TextMeasurer,
) -> LegendLayout {
    let row_h = style.line_height();
    let swatch_gap = 6.0;
    let mut items = Vec::with_capacity(names.len());
    let (mut x, mut row) = (0.0f64, 0usize);
    let mut used_width = 0.0f64;

    for name in names {
        let w = spacing.legend_swatch + swatch_gap + measurer.width(name, style);
        if x > 0.0 && x + w > max_width {
            x = 0.0;
            row += 1;
        }
        let cy = origin.y + row as f64 * (row_h + spacing.legend_row) + row_h / 2.0;
        items.push(LegendItem {
            swatch: Point::new(origin.x + x, cy),
            label: Point::new(origin.x + x + spacing.legend_swatch + swatch_gap, cy),
        });
        used_width = used_width.max(x + w);
        x += w + spacing.legend_item;
    }

    let rows = if names.is_empty() { 0 } else { row + 1 };
    let height = if rows == 0 {
        0.0
    } else {
        rows as f64 * row_h + (rows - 1) as f64 * spacing.legend_row
    };
    LegendLayout {
        items,
        size: Size::new(used_width, height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Color;
    use crate::text::EstimateMeasurer;

    #[test]
    fn wraps_rows() {
        let style = TextStyle::new(10.0, 400, Color::BLACK);
        let spacing = Spacing::default();
        let one_row = layout_legend(
            &["가", "나"],
            Point::default(),
            1000.0,
            &style,
            &spacing,
            &EstimateMeasurer,
        );
        assert_eq!(one_row.size.height, style.line_height());
        assert_eq!(one_row.items[0].swatch.y, one_row.items[1].swatch.y);

        let wrapped = layout_legend(
            &["가", "나"],
            Point::default(),
            40.0,
            &style,
            &spacing,
            &EstimateMeasurer,
        );
        assert!(wrapped.items[1].swatch.y > wrapped.items[0].swatch.y);
        assert_eq!(wrapped.items[1].swatch.x, 0.0);
    }
}
