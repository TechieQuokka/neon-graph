//! End-to-end checks of the generated SVG.

use graph::prelude::*;

fn sample() -> LineChart {
    LineChart::new()
        .title("월별 <매출> & 비용")
        .x_label("월")
        .y_label("금액")
        .series("매출", [1.0, 2.0, 3.0], [100.0, 200.0, 150.0])
}

#[test]
fn document_is_well_formed_enough() {
    let svg = sample().to_svg();
    assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
    assert!(svg.trim_end().ends_with("</svg>"));
    assert_eq!(svg.matches("<g").count(), svg.matches("</g>").count());
    assert!(svg.contains("viewBox=\"0 0 800 500\""));
}

#[test]
fn korean_text_is_kept_and_markup_escaped() {
    let svg = sample().to_svg();
    assert!(svg.contains("월별 &lt;매출&gt; &amp; 비용"));
    assert!(svg.contains(">금액</text>"));
    assert!(svg.contains("Noto Sans KR"));
}

#[test]
fn glow_filter_only_when_enabled() {
    let plain = sample().to_svg();
    assert!(!plain.contains("<filter"));

    let neon = sample().theme(Theme::cyberpunk()).to_svg();
    assert!(neon.contains("<filter id=\"glow-0\""));
    assert!(neon.contains("filter=\"url(#glow-0)\""));

    let off = sample().theme(Theme::cyberpunk()).glow(false).to_svg();
    assert!(!off.contains("<filter"));
}

#[test]
fn data_is_clipped_to_plot() {
    let svg = sample().to_svg();
    assert!(svg.contains("<clipPath id=\"clip-0\""));
    assert!(svg.contains("class=\"data\" clip-path=\"url(#clip-0)\""));
}

#[test]
fn rendering_is_deterministic() {
    assert_eq!(sample().to_svg(), sample().to_svg());
}

#[test]
fn large_series_stays_small() {
    let xs: Vec<f64> = (0..1_000_000).map(|i| i as f64).collect();
    let ys: Vec<f64> = xs.iter().map(|x| (x / 5000.0).sin()).collect();
    let svg = LineChart::new().series("big", &xs, &ys).to_svg();
    assert!(svg.len() < 200_000, "svg too large: {} bytes", svg.len());
}
