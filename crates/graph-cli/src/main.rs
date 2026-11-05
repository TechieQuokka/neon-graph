//! `graph` – a thin CLI over the graph library, for quick visual testing.

mod args;
mod input;
mod theme;

use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use graph::prelude::*;

use args::{Cli, Command, LineArgs};

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    match cli.command {
        Command::Line(args) => line(*args),
        Command::Themes => {
            for name in Theme::BUILTIN {
                println!("{name}");
            }
            Ok(())
        }
        Command::Theme { name } => {
            print!("{}", theme::to_toml(&theme::builtin(&name)?)?);
            Ok(())
        }
    }
}

fn line(args: LineArgs) -> Result<(), Box<dyn Error>> {
    let table = input::read_csv(&args.input)?;
    if table.x_is_index {
        eprintln!(
            "note: x column {:?} is not numeric; using row numbers 1..={}",
            table.x_name,
            table.xs.len()
        );
    }

    let theme = match &args.theme_file {
        Some(path) => theme::load(path, &args.theme)?,
        None => theme::builtin(&args.theme)?,
    };

    let x_label = args.x_label.clone().unwrap_or_else(|| table.x_name.clone());
    let mut chart = LineChart::new()
        .size(args.width, args.height)
        .theme(theme)
        .text_preset(args.preset.into())
        .legend_mode(args.legend.into())
        .axis_position(args.axis.into())
        .x_axis(|a| {
            let a = a.label(x_label).format(args.x_format.into());
            match args.x_ticks {
                Some(n) => a.max_ticks(n),
                None => a,
            }
        })
        .y_axis(|a| {
            let a = a.format(args.y_format.into());
            match args.y_ticks {
                Some(n) => a.max_ticks(n),
                None => a,
            }
        });

    if let Some(t) = &args.title {
        chart = chart.title(t);
    }
    if let Some(t) = &args.subtitle {
        chart = chart.subtitle(t);
    }
    if let Some(t) = &args.y_label {
        chart = chart.y_label(t);
    }
    if let Some(q) = args.quadrant {
        chart = chart.quadrants(q);
    }
    if let Some((x, y)) = args.origin {
        chart = chart.origin(x, y);
    }
    if let Some((a, b)) = args.x_range {
        chart = chart.x_range(a, b);
    }
    if let Some((a, b)) = args.y_range {
        chart = chart.y_range(a, b);
    }
    if args.bare {
        chart = chart.bare();
    }
    if args.grid {
        chart = chart.grid(true);
    }
    if args.no_grid {
        chart = chart.grid(false);
    }
    if args.no_glow {
        chart = chart.glow(false);
    }
    if args.markers {
        chart = chart.theme_with(|t| t.series.markers = true);
    }
    for el in &args.show {
        chart = chart.show(*el, true);
    }
    for el in &args.hide {
        chart = chart.show(*el, false);
    }

    let points: usize = table.series.iter().map(|(_, ys)| ys.len()).sum();
    let series_count = table.series.len();
    for (name, ys) in table.series {
        chart = chart.series(name, &table.xs, ys);
    }

    let svg = chart.to_svg();
    match output_path(&args) {
        None => std::io::stdout().write_all(svg.as_bytes())?,
        Some(path) => {
            std::fs::write(&path, &svg)
                .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
            eprintln!(
                "wrote {} ({series_count} series, {points} points, {} bytes)",
                path.display(),
                svg.len()
            );
        }
    }
    Ok(())
}

/// `None` means stdout.
fn output_path(args: &LineArgs) -> Option<PathBuf> {
    match &args.output {
        Some(p) if p.as_os_str() == "-" => None,
        Some(p) => Some(p.clone()),
        None => Some(args.input.with_extension("svg")),
    }
}
