//! Theme files: partial TOML layered on top of a built-in theme.

use std::error::Error;
use std::path::Path;

use graph::prelude::Theme;
use toml::{Table, Value};

/// Resolves a built-in theme by name.
pub fn builtin(name: &str) -> Result<Theme, Box<dyn Error>> {
    Theme::by_name(name).ok_or_else(|| {
        format!(
            "unknown theme {name:?} (available: {})",
            Theme::BUILTIN.join(", ")
        )
        .into()
    })
}

/// Loads a theme file. Keys missing from the file are taken from the theme
/// named by its `extends` key, or from `default_base`.
pub fn load(path: &Path, default_base: &str) -> Result<Theme, Box<dyn Error>> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read theme file {}: {e}", path.display()))?;
    from_toml(&text, default_base)
        .map_err(|e| format!("invalid theme file {}: {e}", path.display()).into())
}

pub fn from_toml(text: &str, default_base: &str) -> Result<Theme, Box<dyn Error>> {
    let mut overrides: Table = text.parse()?;
    let base_name = match overrides.remove("extends") {
        Some(Value::String(s)) => s,
        Some(_) => return Err("`extends` must be a string".into()),
        None => default_base.to_owned(),
    };
    let base = builtin(&base_name)?;

    let Value::Table(mut merged) = Value::try_from(&base)? else {
        return Err("theme did not serialize to a table".into());
    };
    merge(&mut merged, overrides);
    Ok(Value::Table(merged).try_into()?)
}

/// Serializes a theme to TOML.
pub fn to_toml(theme: &Theme) -> Result<String, Box<dyn Error>> {
    Ok(toml::to_string_pretty(theme)?)
}

/// Deep-merges `src` into `dst`; tables merge recursively, everything else
/// (including arrays such as the palette) is replaced.
fn merge(dst: &mut Table, src: Table) {
    for (key, value) in src {
        match (dst.get_mut(&key), value) {
            (Some(Value::Table(d)), Value::Table(s)) => merge(d, s),
            (_, v) => {
                dst.insert(key, v);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph::prelude::Color;

    #[test]
    fn partial_file_extends_base() {
        let t = from_toml(
            r##"
            extends = "dark"
            background = "#000000"
            [series]
            width = 3.5
            "##,
            "light",
        )
        .unwrap();
        assert_eq!(t.background, Color::BLACK);
        assert_eq!(t.series.width, 3.5);
        // Untouched values come from the dark theme.
        assert_eq!(t.title, Theme::dark().title);
    }

    #[test]
    fn roundtrip_all_builtins() {
        for name in Theme::BUILTIN {
            let theme = builtin(name).unwrap();
            let text = to_toml(&theme).unwrap();
            assert_eq!(from_toml(&text, "light").unwrap(), theme, "{name}");
        }
    }

    #[test]
    fn errors_are_reported() {
        assert!(from_toml("extends = \"nope\"", "light").is_err());
        assert!(from_toml("background = \"#zzz\"", "light").is_err());
    }
}
