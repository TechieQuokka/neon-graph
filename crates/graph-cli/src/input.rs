//! CSV input: first column is x, every other column is a series.

use std::error::Error;
use std::path::Path;

/// Columns read from a CSV file.
#[derive(Debug)]
pub struct Table {
    pub x_name: String,
    pub xs: Vec<f64>,
    pub series: Vec<(String, Vec<f64>)>,
    /// `true` when the x column was not numeric and rows were numbered instead.
    pub x_is_index: bool,
}

pub fn read_csv(path: &Path) -> Result<Table, Box<dyn Error>> {
    let reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_path(path)
        .map_err(|e| format!("cannot open {}: {e}", path.display()))?;
    parse(reader)
}

fn parse<R: std::io::Read>(mut reader: csv::Reader<R>) -> Result<Table, Box<dyn Error>> {
    let headers: Vec<String> = reader
        .headers()?
        .iter()
        // Excel exports UTF-8 CSV with a byte order mark.
        .map(|h| h.trim_start_matches('\u{feff}').to_owned())
        .collect();
    if headers.len() < 2 {
        return Err("CSV needs at least two columns: x and one series".into());
    }

    let mut x_raw = Vec::new();
    let mut columns: Vec<Vec<f64>> = vec![Vec::new(); headers.len() - 1];
    for record in reader.records() {
        let record = record?;
        if record.iter().all(str::is_empty) {
            continue;
        }
        x_raw.push(record.get(0).unwrap_or("").to_owned());
        for (i, col) in columns.iter_mut().enumerate() {
            col.push(parse_number(record.get(i + 1).unwrap_or("")));
        }
    }

    let parsed: Vec<f64> = x_raw.iter().map(|s| parse_number(s)).collect();
    let x_is_index = parsed
        .iter()
        .zip(&x_raw)
        .any(|(v, raw)| v.is_nan() && !raw.is_empty());
    let xs = if x_is_index {
        (1..=x_raw.len()).map(|i| i as f64).collect()
    } else {
        parsed
    };

    let series = headers[1..]
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let name = if name.is_empty() {
                format!("series {}", i + 1)
            } else {
                name.clone()
            };
            (name, std::mem::take(&mut columns[i]))
        })
        .collect();

    Ok(Table {
        x_name: headers[0].clone(),
        xs,
        series,
        x_is_index,
    })
}

/// Parses a cell; thousands separators are allowed, empty or invalid -> NaN (gap).
fn parse_number(s: &str) -> f64 {
    let cleaned: String = s
        .chars()
        .filter(|c| !matches!(c, ',' | '_' | ' '))
        .collect();
    cleaned.parse().unwrap_or(f64::NAN)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(csv: &str) -> Table {
        parse(
            csv::ReaderBuilder::new()
                .flexible(true)
                .trim(csv::Trim::All)
                .from_reader(csv.as_bytes()),
        )
        .unwrap()
    }

    #[test]
    fn reads_korean_headers_with_bom() {
        let t = table("\u{feff}월,매출,비용\n1,100,80\n2,\"1,200\",\n");
        assert_eq!(t.x_name, "월");
        assert_eq!(t.series[0].0, "매출");
        assert_eq!(t.series[0].1, vec![100.0, 1200.0]);
        assert!(t.series[1].1[1].is_nan());
        assert!(!t.x_is_index);
    }

    #[test]
    fn non_numeric_x_becomes_index() {
        let t = table("월,값\n1월,5\n2월,6\n");
        assert!(t.x_is_index);
        assert_eq!(t.xs, vec![1.0, 2.0]);
    }
}
