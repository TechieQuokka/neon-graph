//! Number formatting for tick labels.

use super::ticks::decimals_for_step;

/// How tick values are printed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum NumberFormat {
    /// `12000` -> `12000`
    Plain,
    /// `12000` -> `12,000`
    #[default]
    Thousands,
    /// Korean myriad units: `12000` -> `1.2만`, `300000000` -> `3억`
    Korean,
    /// SI suffixes: `12000` -> `12k`, `3500000` -> `3.5M`
    Si,
}

impl NumberFormat {
    /// Formats every tick value of an axis consistently.
    ///
    /// `step` is the distance between ticks; it decides the precision so that
    /// neighbouring labels never collapse into the same text.
    pub fn format_ticks(self, values: &[f64], step: f64) -> Vec<String> {
        let max_abs = values.iter().fold(0.0f64, |m, v| m.max(v.abs()));
        match self {
            NumberFormat::Plain => {
                let d = decimals_for_step(step);
                values.iter().map(|v| fixed(*v, d)).collect()
            }
            NumberFormat::Thousands => {
                let d = decimals_for_step(step);
                values
                    .iter()
                    .map(|v| group_thousands(&fixed(*v, d)))
                    .collect()
            }
            NumberFormat::Korean => with_unit(values, step, max_abs, &KOREAN_UNITS, ""),
            NumberFormat::Si => with_unit(values, step, max_abs, &SI_UNITS, ""),
        }
    }

    /// Formats a single value (precision is inferred from the value itself).
    pub fn format_value(self, value: f64) -> String {
        let step = if value == 0.0 {
            1.0
        } else {
            10f64.powf(value.abs().log10().floor() - 2.0)
        };
        self.format_ticks(&[value], step).pop().unwrap_or_default()
    }
}

const KOREAN_UNITS: [(f64, &str); 3] = [(1e12, "조"), (1e8, "억"), (1e4, "만")];
const SI_UNITS: [(f64, &str); 4] = [(1e12, "T"), (1e9, "G"), (1e6, "M"), (1e3, "k")];

fn with_unit(
    values: &[f64],
    step: f64,
    max_abs: f64,
    units: &[(f64, &str)],
    none: &str,
) -> Vec<String> {
    let (div, suffix) = units
        .iter()
        .find(|(div, _)| max_abs >= *div)
        .map(|(d, s)| (*d, *s))
        .unwrap_or((1.0, none));
    let d = decimals_for_step(step / div);
    values
        .iter()
        .map(|v| {
            if *v == 0.0 {
                "0".to_owned()
            } else {
                format!("{}{suffix}", group_thousands(&fixed(v / div, d)))
            }
        })
        .collect()
}

/// Fixed-point formatting without `-0`.
fn fixed(value: f64, decimals: usize) -> String {
    let s = format!("{value:.decimals$}");
    if s.starts_with('-') && s[1..].chars().all(|c| c == '0' || c == '.') {
        s[1..].to_owned()
    } else {
        s
    }
}

/// Inserts `,` every three digits of the integer part.
fn group_thousands(s: &str) -> String {
    let (sign, rest) = s.strip_prefix('-').map_or(("", s), |r| ("-", r));
    let (int, frac) = rest
        .split_once('.')
        .map_or((rest, None), |(i, f)| (i, Some(f)));

    let mut out = String::with_capacity(s.len() + int.len() / 3);
    out.push_str(sign);
    for (i, ch) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    if let Some(f) = frac {
        out.push('.');
        out.push_str(f);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands() {
        let f = NumberFormat::Thousands;
        assert_eq!(
            f.format_ticks(&[0.0, 5000.0, 12000.0, -1234567.0], 1000.0),
            ["0", "5,000", "12,000", "-1,234,567"]
        );
        assert_eq!(f.format_ticks(&[0.5, 1.0], 0.5), ["0.5", "1.0"]);
    }

    #[test]
    fn plain_has_no_negative_zero() {
        assert_eq!(NumberFormat::Plain.format_ticks(&[-0.0001], 1.0), ["0"]);
    }

    #[test]
    fn korean_units() {
        let f = NumberFormat::Korean;
        assert_eq!(
            f.format_ticks(&[0.0, 10000.0, 12000.0, 20000.0], 2000.0),
            ["0", "1.0만", "1.2만", "2.0만"]
        );
        assert_eq!(
            f.format_ticks(&[100_000_000.0, 300_000_000.0], 100_000_000.0),
            ["1억", "3억"]
        );
        assert_eq!(f.format_ticks(&[10.0, 20.0], 10.0), ["10", "20"]);
    }

    #[test]
    fn si_units() {
        assert_eq!(
            NumberFormat::Si.format_ticks(&[0.0, 2_500_000.0, 5_000_000.0], 2_500_000.0),
            ["0", "2.5M", "5.0M"]
        );
    }
}
