use std::fmt;
use std::str::FromStr;

/// Every text element of a chart that can be shown or hidden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextElement {
    Title,
    Subtitle,
    Legend,
    XLabel,
    YLabel,
    XTickLabels,
    YTickLabels,
}

impl TextElement {
    pub const ALL: [TextElement; 7] = [
        TextElement::Title,
        TextElement::Subtitle,
        TextElement::Legend,
        TextElement::XLabel,
        TextElement::YLabel,
        TextElement::XTickLabels,
        TextElement::YTickLabels,
    ];

    /// Kebab-case name used by the CLI (`x-tick-labels`, ...).
    pub fn name(self) -> &'static str {
        match self {
            TextElement::Title => "title",
            TextElement::Subtitle => "subtitle",
            TextElement::Legend => "legend",
            TextElement::XLabel => "x-label",
            TextElement::YLabel => "y-label",
            TextElement::XTickLabels => "x-tick-labels",
            TextElement::YTickLabels => "y-tick-labels",
        }
    }
}

impl fmt::Display for TextElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Error returned when parsing a [`TextElement`] fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextElementParseError(String);

impl fmt::Display for TextElementParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<_> = TextElement::ALL.iter().map(|e| e.name()).collect();
        write!(
            f,
            "unknown text element {:?} (expected one of: {})",
            self.0,
            names.join(", ")
        )
    }
}

impl std::error::Error for TextElementParseError {}

impl FromStr for TextElement {
    type Err = TextElementParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let norm = s.trim().to_ascii_lowercase().replace('_', "-");
        TextElement::ALL
            .into_iter()
            .find(|e| e.name() == norm)
            .ok_or_else(|| TextElementParseError(s.to_owned()))
    }
}

/// Predefined text visibility combinations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextPreset {
    /// Everything visible. Reports and documents.
    #[default]
    Full,
    /// Title and tick labels only. Small charts.
    Compact,
    /// No text at all. Sparklines and thumbnails.
    Minimal,
}

impl TextPreset {
    pub fn visibility(self) -> TextVisibility {
        let all = |v| TextVisibility {
            title: v,
            subtitle: v,
            legend: v,
            x_label: v,
            y_label: v,
            x_tick_labels: v,
            y_tick_labels: v,
        };
        match self {
            TextPreset::Full => all(true),
            TextPreset::Minimal => all(false),
            TextPreset::Compact => TextVisibility {
                title: true,
                x_tick_labels: true,
                y_tick_labels: true,
                ..all(false)
            },
        }
    }
}

impl FromStr for TextPreset {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "full" => Ok(TextPreset::Full),
            "compact" => Ok(TextPreset::Compact),
            "minimal" => Ok(TextPreset::Minimal),
            _ => Err(format!(
                "unknown preset {s:?} (expected full, compact or minimal)"
            )),
        }
    }
}

/// Resolved visibility of every text element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextVisibility {
    pub title: bool,
    pub subtitle: bool,
    pub legend: bool,
    pub x_label: bool,
    pub y_label: bool,
    pub x_tick_labels: bool,
    pub y_tick_labels: bool,
}

impl TextVisibility {
    pub fn get(&self, element: TextElement) -> bool {
        match element {
            TextElement::Title => self.title,
            TextElement::Subtitle => self.subtitle,
            TextElement::Legend => self.legend,
            TextElement::XLabel => self.x_label,
            TextElement::YLabel => self.y_label,
            TextElement::XTickLabels => self.x_tick_labels,
            TextElement::YTickLabels => self.y_tick_labels,
        }
    }

    pub fn set(&mut self, element: TextElement, visible: bool) {
        let slot = match element {
            TextElement::Title => &mut self.title,
            TextElement::Subtitle => &mut self.subtitle,
            TextElement::Legend => &mut self.legend,
            TextElement::XLabel => &mut self.x_label,
            TextElement::YLabel => &mut self.y_label,
            TextElement::XTickLabels => &mut self.x_tick_labels,
            TextElement::YTickLabels => &mut self.y_tick_labels,
        };
        *slot = visible;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets() {
        let c = TextPreset::Compact.visibility();
        assert!(c.title && c.x_tick_labels && c.y_tick_labels);
        assert!(!c.legend && !c.x_label && !c.subtitle);
        assert!(
            TextElement::ALL
                .iter()
                .all(|e| !TextPreset::Minimal.visibility().get(*e))
        );
    }

    #[test]
    fn parse_elements() {
        for e in TextElement::ALL {
            assert_eq!(e.name().parse::<TextElement>().unwrap(), e);
        }
        assert_eq!(
            "Y_TICK_LABELS".parse::<TextElement>().unwrap(),
            TextElement::YTickLabels
        );
        assert!("foo".parse::<TextElement>().is_err());
    }
}
