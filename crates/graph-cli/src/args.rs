//! Command line definitions.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use graph::prelude::*;

#[derive(Debug, Parser)]
#[command(
    name = "graph",
    version,
    about = "Render charts from CSV files (test tool for the graph library)"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Draw a line chart. First CSV column is x, every other column is a series.
    Line(Box<LineArgs>),
    /// List the built-in themes.
    Themes,
    /// Print a built-in theme as TOML, a starting point for --theme-file.
    Theme {
        /// Theme name (see `graph themes`).
        name: String,
    },
}

#[derive(Debug, Args)]
pub struct LineArgs {
    /// Input CSV file. The header row provides the series names.
    pub input: PathBuf,

    /// Output SVG file (`-` for stdout). Defaults to the input name with `.svg`.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    // ---- Text ---------------------------------------------------------------
    #[arg(long)]
    pub title: Option<String>,

    #[arg(long)]
    pub subtitle: Option<String>,

    /// X axis label. Defaults to the first CSV header.
    #[arg(long)]
    pub x_label: Option<String>,

    #[arg(long)]
    pub y_label: Option<String>,

    /// Text visibility preset.
    #[arg(long, value_enum, default_value_t = PresetArg::Full)]
    pub preset: PresetArg,

    /// Text elements to force visible (comma separated):
    /// title, subtitle, legend, x-label, y-label, x-tick-labels, y-tick-labels.
    #[arg(long, value_delimiter = ',')]
    pub show: Vec<TextElement>,

    /// Text elements to hide (comma separated, same names as --show).
    #[arg(long, value_delimiter = ',')]
    pub hide: Vec<TextElement>,

    /// Data only: no text, axes, ticks, grid or padding. --show still works.
    #[arg(long)]
    pub bare: bool,

    /// How series names are shown.
    #[arg(long, value_enum, default_value_t = LegendArg::Top)]
    pub legend: LegendArg,

    // ---- Size & style -------------------------------------------------------
    #[arg(long, default_value_t = 800.0)]
    pub width: f64,

    #[arg(long, default_value_t = 500.0)]
    pub height: f64,

    /// Built-in theme. Also the base that --theme-file extends by default.
    #[arg(long, default_value = "cyberpunk")]
    pub theme: String,

    /// TOML theme file. Missing keys fall back to the base theme
    /// (`extends = "<name>"` in the file, or --theme).
    #[arg(long)]
    pub theme_file: Option<PathBuf>,

    /// Show the grid (also in --bare mode).
    #[arg(long, conflicts_with = "no_grid")]
    pub grid: bool,

    /// Hide the grid.
    #[arg(long)]
    pub no_grid: bool,

    /// Disable the glow effect.
    #[arg(long)]
    pub no_glow: bool,

    /// Draw a dot at every data point.
    #[arg(long)]
    pub markers: bool,

    // ---- Viewport -----------------------------------------------------------
    /// Visible quadrant(s), e.g. `1` or `1,4` or `all`.
    #[arg(long)]
    pub quadrant: Option<Quadrants>,

    /// Origin position as fractions from the bottom-left, e.g. `0.25,0.25`.
    #[arg(long, value_parser = parse_pair, allow_hyphen_values = true)]
    pub origin: Option<(f64, f64)>,

    /// Fixed x range `MIN,MAX`. Use `--x-range=-2,10` for negative values.
    #[arg(long, value_parser = parse_pair, allow_hyphen_values = true)]
    pub x_range: Option<(f64, f64)>,

    /// Fixed y range `MIN,MAX`.
    #[arg(long, value_parser = parse_pair, allow_hyphen_values = true)]
    pub y_range: Option<(f64, f64)>,

    /// Where axis lines are drawn.
    #[arg(long, value_enum, default_value_t = AxisArg::Auto)]
    pub axis: AxisArg,

    // ---- Axes ---------------------------------------------------------------
    #[arg(long, value_enum, default_value_t = FormatArg::Thousands)]
    pub x_format: FormatArg,

    #[arg(long, value_enum, default_value_t = FormatArg::Thousands)]
    pub y_format: FormatArg,

    /// Approximate number of x tick intervals.
    #[arg(long)]
    pub x_ticks: Option<usize>,

    /// Approximate number of y tick intervals.
    #[arg(long)]
    pub y_ticks: Option<usize>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PresetArg {
    Full,
    Compact,
    Minimal,
}

impl From<PresetArg> for TextPreset {
    fn from(p: PresetArg) -> Self {
        match p {
            PresetArg::Full => TextPreset::Full,
            PresetArg::Compact => TextPreset::Compact,
            PresetArg::Minimal => TextPreset::Minimal,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LegendArg {
    Top,
    Direct,
}

impl From<LegendArg> for LegendMode {
    fn from(l: LegendArg) -> Self {
        match l {
            LegendArg::Top => LegendMode::Top,
            LegendArg::Direct => LegendMode::Direct,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AxisArg {
    Auto,
    Edge,
    Origin,
}

impl From<AxisArg> for AxisPosition {
    fn from(a: AxisArg) -> Self {
        match a {
            AxisArg::Auto => AxisPosition::Auto,
            AxisArg::Edge => AxisPosition::Edge,
            AxisArg::Origin => AxisPosition::Origin,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FormatArg {
    /// 12000
    Plain,
    /// 12,000
    Thousands,
    /// 1.2만
    Korean,
    /// 12k
    Si,
}

impl From<FormatArg> for NumberFormat {
    fn from(f: FormatArg) -> Self {
        match f {
            FormatArg::Plain => NumberFormat::Plain,
            FormatArg::Thousands => NumberFormat::Thousands,
            FormatArg::Korean => NumberFormat::Korean,
            FormatArg::Si => NumberFormat::Si,
        }
    }
}

/// Parses `A,B` into two numbers.
fn parse_pair(s: &str) -> Result<(f64, f64), String> {
    let (a, b) = s
        .split_once(',')
        .ok_or_else(|| format!("expected two comma-separated numbers, got {s:?}"))?;
    let num = |v: &str| {
        v.trim()
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .ok_or_else(|| format!("{v:?} is not a finite number"))
    };
    Ok((num(a)?, num(b)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs() {
        assert_eq!(parse_pair("-2, 10").unwrap(), (-2.0, 10.0));
        assert!(parse_pair("1").is_err());
        assert!(parse_pair("a,1").is_err());
    }

    #[test]
    fn cli_parses() {
        let cli = Cli::try_parse_from([
            "graph",
            "line",
            "d.csv",
            "--quadrant",
            "1,4",
            "--x-range=-2,10",
            "--hide",
            "legend,y-label",
        ])
        .unwrap();
        let Command::Line(args) = cli.command else {
            panic!()
        };
        assert_eq!(args.x_range, Some((-2.0, 10.0)));
        assert_eq!(args.hide, vec![TextElement::Legend, TextElement::YLabel]);
        assert!(args.quadrant.unwrap().contains(Quadrant::Q4));
    }
}
