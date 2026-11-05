//! Quadrant selection and origin placement.
//!
//! Run: `cargo run -p graph --example quadrants`

use graph::prelude::*;

fn main() -> std::io::Result<()> {
    let xs: Vec<f64> = (-40..=40).map(|i| f64::from(i) / 10.0).collect();
    let cubic: Vec<f64> = xs.iter().map(|x| x * x * x / 8.0).collect();
    let line: Vec<f64> = xs.iter().map(|x| 0.5 * x + 1.0).collect();

    let base = || {
        LineChart::new()
            .size(560.0, 400.0)
            .x_label("x")
            .y_label("y")
            .series("x³/8", &xs, &cubic)
            .series("x/2 + 1", &xs, &line)
    };

    // 1. All four quadrants, axes through the origin, equal weight.
    base()
        .title("4개 분면 (균등)")
        .origin(0.5, 0.5)
        .save_svg("quadrants_all.svg")?;

    // 2. First quadrant only.
    base()
        .title("1사분면만")
        .quadrants(Quadrant::Q1)
        .save_svg("quadrants_q1.svg")?;

    // 3. All quadrants, but Q1 gets most of the space.
    base()
        .title("1사분면 비중 크게 (원점 25%, 25%)")
        .origin(0.25, 0.25)
        .save_svg("quadrants_weighted.svg")?;

    // 4. Right half (Q1 + Q4).
    base()
        .title("1·4사분면")
        .quadrants([Quadrant::Q1, Quadrant::Q4])
        .save_svg("quadrants_right.svg")?;

    println!("wrote quadrants_*.svg");
    Ok(())
}
