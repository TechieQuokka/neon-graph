use super::Color;

/// An ordered list of series colors. Indexing cycles through the list.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Palette(pub Vec<Color>);

impl Palette {
    pub fn new(colors: impl IntoIterator<Item = Color>) -> Self {
        Self(colors.into_iter().collect())
    }

    /// Color for the `i`-th series. Falls back to black for an empty palette.
    pub fn get(&self, i: usize) -> Color {
        if self.0.is_empty() {
            Color::BLACK
        } else {
            self.0[i % self.0.len()]
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<C: Into<Color>> FromIterator<C> for Palette {
    fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
        Self(iter.into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycles() {
        let p = Palette::new([Color::BLACK, Color::WHITE]);
        assert_eq!(p.get(0), Color::BLACK);
        assert_eq!(p.get(3), Color::WHITE);
        assert_eq!(Palette::new([]).get(5), Color::BLACK);
    }
}
