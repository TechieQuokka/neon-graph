//! Theme customization at every level: theme, chart, axis and series.
//!
//! Run: `cargo run -p graph --example custom_theme`

use graph::prelude::*;

fn main() -> std::io::Result<()> {
    let xs: Vec<f64> = (0..=60).map(f64::from).collect();
    let a: Vec<f64> = xs
        .iter()
        .map(|x| 30_000.0 + 12_000.0 * (x / 8.0).sin() + x * 900.0)
        .collect();
    let b: Vec<f64> = xs
        .iter()
        .map(|x| 25_000.0 + 8_000.0 * (x / 5.0).cos() + x * 600.0)
        .collect();

    // Cyberpunk with glow, Korean number units and direct labels.
    LineChart::new()
        .theme(Theme::cyberpunk())
        .title("서버 트래픽")
        .subtitle("최근 60분 · 요청 수")
        .x_label("분")
        .legend_mode(LegendMode::Direct)
        .y_axis(|a| a.format(NumberFormat::Korean))
        .series("서울", &xs, &a)
        .series("부산", &xs, &b)
        .save_svg("theme_cyberpunk.svg")?;

    // Light theme, tweaked at every level.
    LineChart::new()
        .theme_with(|t| {
            t.grid_x.visible = true;
            t.series.width = 2.5;
            t.title.color = Color::hex("#0b3d91");
        })
        .background(Color::hex("#fbfaf7"))
        .palette([Color::hex("#0b3d91"), Color::hex("#e4572e")])
        .title("커스텀 라이트 테마")
        .x_axis(|a| a.label("분").line_color(Color::hex("#0b3d91")))
        .y_axis(|a| a.format(NumberFormat::Si).max_ticks(4))
        .series("서울", &xs, &a)
        .series("부산", &xs, &b)
        .series_style("부산", |s| {
            s.dash([2.0, 3.0]).markers(true).marker_radius(2.0)
        })
        .save_svg("theme_custom_light.svg")?;

    // Sparkline: data only, tiny canvas.
    LineChart::new()
        .size(240.0, 60.0)
        .bare()
        .series("서울", &xs, &a)
        .save_svg("sparkline.svg")?;

    println!("wrote theme_cyberpunk.svg, theme_custom_light.svg, sparkline.svg");
    Ok(())
}
