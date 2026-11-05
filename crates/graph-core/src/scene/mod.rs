//! Backend-agnostic description of what to draw.
//!
//! Charts produce a [`Scene`]; renderers (SVG, PNG, ...) consume it. Neither
//! side knows about the other, so `N` chart kinds and `M` backends need only
//! `N + M` implementations.

mod path;

pub use path::{Path, PathBuilder, PathCmd};

use crate::geom::{Point, Rect, Size};
use crate::style::{Color, Glow, Stroke, TextStyle};

/// Horizontal alignment of text relative to its anchor point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAnchor {
    #[default]
    Start,
    Middle,
    End,
}

/// Vertical alignment of text relative to its anchor point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Baseline {
    /// Anchor is the top of the text box.
    #[default]
    Top,
    /// Anchor is the vertical center of the text box.
    Middle,
    /// Anchor is the alphabetic baseline.
    Alphabetic,
    /// Anchor is the bottom of the text box.
    Bottom,
}

/// Identifier of a clip rectangle registered in a [`Scene`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClipId(pub usize);

/// Post-processing effect applied to a group.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Effect {
    Glow(Glow),
}

/// A drawable element.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Group(Group),
    Rect(RectNode),
    Path(PathNode),
    Circle(CircleNode),
    Text(TextNode),
}

/// A container with optional clipping, effect and opacity.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Group {
    /// Semantic role, e.g. `"grid"` or `"series"`. Backends may expose it
    /// (SVG writes it as a `class`) to make output easy to inspect and style.
    pub role: Option<&'static str>,
    pub clip: Option<ClipId>,
    pub effect: Option<Effect>,
    pub opacity: Option<f64>,
    pub children: Vec<Node>,
}

impl Group {
    pub fn new(role: &'static str) -> Self {
        Self {
            role: Some(role),
            ..Default::default()
        }
    }

    pub fn push(&mut self, node: impl Into<Node>) {
        self.children.push(node.into());
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RectNode {
    pub rect: Rect,
    pub fill: Option<Color>,
    pub stroke: Option<Stroke>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathNode {
    pub path: Path,
    pub stroke: Option<Stroke>,
    pub fill: Option<Color>,
    pub opacity: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CircleNode {
    pub center: Point,
    pub radius: f64,
    pub fill: Option<Color>,
    pub stroke: Option<Stroke>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextNode {
    pub text: String,
    pub pos: Point,
    pub style: TextStyle,
    pub anchor: TextAnchor,
    pub baseline: Baseline,
    /// Use tabular (fixed-width) digits, for numbers that line up.
    pub tabular_nums: bool,
}

impl TextNode {
    pub fn new(text: impl Into<String>, pos: Point, style: TextStyle) -> Self {
        Self {
            text: text.into(),
            pos,
            style,
            anchor: TextAnchor::Start,
            baseline: Baseline::Top,
            tabular_nums: false,
        }
    }

    pub fn anchor(mut self, anchor: TextAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn baseline(mut self, baseline: Baseline) -> Self {
        self.baseline = baseline;
        self
    }

    pub fn tabular(mut self) -> Self {
        self.tabular_nums = true;
        self
    }
}

macro_rules! impl_into_node {
    ($($ty:ident => $variant:ident),*) => {
        $(impl From<$ty> for Node {
            fn from(n: $ty) -> Self {
                Node::$variant(n)
            }
        })*
    };
}

impl_into_node!(Group => Group, RectNode => Rect, PathNode => Path, CircleNode => Circle, TextNode => Text);

/// Everything needed to draw one chart.
#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub size: Size,
    pub background: Option<Color>,
    clips: Vec<Rect>,
    pub root: Group,
}

impl Scene {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            background: None,
            clips: Vec::new(),
            root: Group::new("chart"),
        }
    }

    /// Registers a clip rectangle and returns its id.
    pub fn add_clip(&mut self, rect: Rect) -> ClipId {
        self.clips.push(rect);
        ClipId(self.clips.len() - 1)
    }

    pub fn clips(&self) -> impl Iterator<Item = (ClipId, Rect)> + '_ {
        self.clips.iter().enumerate().map(|(i, r)| (ClipId(i), *r))
    }

    pub fn push(&mut self, node: impl Into<Node>) {
        self.root.push(node);
    }

    /// Calls `f` for every node, depth-first.
    pub fn walk(&self, f: &mut impl FnMut(&Node)) {
        fn go(node: &Node, f: &mut impl FnMut(&Node)) {
            f(node);
            if let Node::Group(g) = node {
                for c in &g.children {
                    go(c, f);
                }
            }
        }
        for c in &self.root.children {
            go(c, f);
        }
    }

    /// All text strings in drawing order. Handy for tests.
    pub fn texts(&self) -> Vec<&str> {
        let mut out = Vec::new();
        fn go<'a>(g: &'a Group, out: &mut Vec<&'a str>) {
            for c in &g.children {
                match c {
                    Node::Text(t) => out.push(t.text.as_str()),
                    Node::Group(g) => go(g, out),
                    _ => {}
                }
            }
        }
        go(&self.root, &mut out);
        out
    }
}
