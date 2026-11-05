//! A simple two-series line chart with Korean text.
//!
//! Run: `cargo run -p graph --example basic_line`

use graph::prelude::*;

fn main() -> std::io::Result<()> {
    let months: Vec<f64> = (1..=12).map(f64::from).collect();
    let sales = [
        1200., 1350., 1280., 1610., 1720., 1690., 1850., 2010., 1940., 2150., 2300., 2480.,
    ];
    let costs = [
        950., 1010., 990., 1120., 1180., 1230., 1300., 1350., 1390., 1420., 1510., 1600.,
    ];

    LineChart::new()
        .title("월별 매출 추이")
        .subtitle("2026년 · 단위: 만원")
        .x_label("월")
        .y_label("금액")
        .series("매출", &months, sales)
        .series_with("비용", &months, costs, |s| s.dash([6.0, 4.0]))
        .save_svg("basic_line.svg")?;

    println!("wrote basic_line.svg");
    Ok(())
}
