//! SVG backend for the `graph` library.
//!
//! Turns a backend-agnostic [`Scene`] into a compact, standalone SVG document.
//! No external dependencies.
//!
//! ```
//! use graph_core::chart::{Chart, LineChart};
//! use graph_svg::to_svg;
//!
//! let scene = LineChart::new().series_y("값", [1.0, 3.0, 2.0]).build_scene();
//! let svg = to_svg(&scene);
//! assert!(svg.starts_with("<svg"));
//! ```

mod filter;
mod writer;

use std::collections::HashMap;
use std::convert::Infallible;

use graph_core::render::Renderer;
use graph_core::scene::{Baseline, ClipId, Effect, Group, Node, PathCmd, Scene, TextAnchor};
use graph_core::style::Glow;

use writer::{attr_num, attr_str, escape, num, paint};

/// Renders scenes to SVG text.
#[derive(Debug, Clone, Copy)]
pub struct SvgRenderer {
    /// Decimal places for coordinates.
    pub precision: usize,
}

impl Default for SvgRenderer {
    fn default() -> Self {
        Self { precision: 2 }
    }
}

impl SvgRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn precision(mut self, precision: usize) -> Self {
        self.precision = precision.min(6);
        self
    }

    /// Renders `scene` to an SVG string.
    pub fn render_to_string(&self, scene: &Scene) -> String {
        let mut ctx = Ctx {
            out: String::with_capacity(16 * 1024),
            p: self.precision,
            glows: HashMap::new(),
        };
        ctx.document(scene);
        ctx.out
    }
}

impl Renderer for SvgRenderer {
    type Output = String;
    type Error = Infallible;

    fn render(&self, scene: &Scene) -> Result<String, Infallible> {
        Ok(self.render_to_string(scene))
    }
}

/// Renders `scene` with the default settings.
pub fn to_svg(scene: &Scene) -> String {
    SvgRenderer::default().render_to_string(scene)
}

struct Ctx {
    out: String,
    p: usize,
    /// Glow parameters (as bit patterns) -> filter id.
    glows: HashMap<(u64, u64), String>,
}

impl Ctx {
    fn document(&mut self, scene: &Scene) {
        let (w, h) = (scene.size.width, scene.size.height);
        self.out
            .push_str("<svg xmlns=\"http://www.w3.org/2000/svg\"");
        attr_num(&mut self.out, "width", w, self.p);
        attr_num(&mut self.out, "height", h, self.p);
        self.out.push_str(" viewBox=\"0 0 ");
        num(&mut self.out, w, self.p);
        self.out.push(' ');
        num(&mut self.out, h, self.p);
        self.out.push_str("\">\n");

        self.defs(scene);

        if let Some(bg) = scene.background {
            self.out.push_str("<rect width=\"100%\" height=\"100%\"");
            paint(&mut self.out, "fill", bg, self.p);
            self.out.push_str("/>\n");
        }

        for child in &scene.root.children {
            self.node(child);
        }
        self.out.push_str("</svg>\n");
    }

    fn defs(&mut self, scene: &Scene) {
        let mut glows: Vec<Glow> = Vec::new();
        scene.walk(&mut |n| {
            if let Node::Group(Group {
                effect: Some(Effect::Glow(g)),
                children,
                ..
            }) = n
                && !children.is_empty()
                && !glows.contains(g)
            {
                glows.push(*g);
            }
        });
        let clips: Vec<_> = scene.clips().collect();
        if glows.is_empty() && clips.is_empty() {
            return;
        }

        self.out.push_str("<defs>");
        for (id, rect) in clips {
            self.out.push_str("<clipPath");
            attr_str(&mut self.out, "id", &clip_id(id));
            self.out.push_str("><rect");
            attr_num(&mut self.out, "x", rect.x, self.p);
            attr_num(&mut self.out, "y", rect.y, self.p);
            attr_num(&mut self.out, "width", rect.width, self.p);
            attr_num(&mut self.out, "height", rect.height, self.p);
            self.out.push_str("/></clipPath>");
        }
        for (i, g) in glows.iter().enumerate() {
            let id = format!("glow-{i}");
            filter::glow(&mut self.out, &id, g, scene.size, self.p);
            self.glows.insert(glow_key(g), id);
        }
        self.out.push_str("</defs>\n");
    }

    fn node(&mut self, node: &Node) {
        let p = self.p;
        match node {
            Node::Group(g) => self.group(g),
            Node::Rect(r) => {
                let o = &mut self.out;
                o.push_str("<rect");
                attr_num(o, "x", r.rect.x, p);
                attr_num(o, "y", r.rect.y, p);
                attr_num(o, "width", r.rect.width, p);
                attr_num(o, "height", r.rect.height, p);
                match r.fill {
                    Some(c) => paint(o, "fill", c, p),
                    None => attr_str(o, "fill", "none"),
                }
                if let Some(s) = &r.stroke {
                    writer::stroke(o, s, p);
                }
                o.push_str("/>\n");
            }
            Node::Circle(c) => {
                let o = &mut self.out;
                o.push_str("<circle");
                attr_num(o, "cx", c.center.x, p);
                attr_num(o, "cy", c.center.y, p);
                attr_num(o, "r", c.radius, p);
                match c.fill {
                    Some(col) => paint(o, "fill", col, p),
                    None => attr_str(o, "fill", "none"),
                }
                if let Some(s) = &c.stroke {
                    writer::stroke(o, s, p);
                }
                o.push_str("/>\n");
            }
            Node::Path(path) => {
                if path.path.is_empty() {
                    return;
                }
                let o = &mut self.out;
                o.push_str("<path d=\"");
                for (i, cmd) in path.path.commands().iter().enumerate() {
                    if i > 0 {
                        o.push(' ');
                    }
                    match cmd {
                        PathCmd::MoveTo(pt) | PathCmd::LineTo(pt) => {
                            o.push(if matches!(cmd, PathCmd::MoveTo(_)) {
                                'M'
                            } else {
                                'L'
                            });
                            num(o, pt.x, p);
                            o.push(' ');
                            num(o, pt.y, p);
                        }
                        PathCmd::Close => o.push('Z'),
                    }
                }
                o.push('"');
                match path.fill {
                    Some(c) => paint(o, "fill", c, p),
                    None => attr_str(o, "fill", "none"),
                }
                if let Some(s) = &path.stroke {
                    writer::stroke(o, s, p);
                }
                if let Some(op) = path.opacity {
                    attr_num(o, "opacity", op, p.max(2));
                }
                o.push_str("/>\n");
            }
            Node::Text(t) => {
                let o = &mut self.out;
                let size = t.style.size;
                // Convert our box-relative baselines into alphabetic baselines;
                // `dominant-baseline` is not supported consistently by viewers.
                let dy = match t.baseline {
                    Baseline::Top => size * 0.9,
                    Baseline::Middle => size * 0.35,
                    Baseline::Alphabetic => 0.0,
                    Baseline::Bottom => -size * 0.3,
                };
                o.push_str("<text");
                attr_num(o, "x", t.pos.x, p);
                attr_num(o, "y", t.pos.y + dy, p);
                match t.anchor {
                    TextAnchor::Start => {}
                    TextAnchor::Middle => attr_str(o, "text-anchor", "middle"),
                    TextAnchor::End => attr_str(o, "text-anchor", "end"),
                }
                attr_str(o, "font-family", &t.style.family);
                attr_num(o, "font-size", size, p);
                if t.style.weight != 400 {
                    attr_num(o, "font-weight", t.style.weight as f64, 0);
                }
                paint(o, "fill", t.style.color, p);
                if t.tabular_nums {
                    attr_str(o, "font-variant-numeric", "tabular-nums");
                }
                o.push('>');
                escape(o, &t.text);
                o.push_str("</text>\n");
            }
        }
    }

    fn group(&mut self, g: &Group) {
        if g.children.is_empty() {
            return;
        }
        let p = self.p;
        self.out.push_str("<g");
        if let Some(role) = g.role {
            attr_str(&mut self.out, "class", role);
        }
        if let Some(clip) = g.clip {
            attr_str(
                &mut self.out,
                "clip-path",
                &format!("url(#{})", clip_id(clip)),
            );
        }
        if let Some(Effect::Glow(glow)) = &g.effect
            && let Some(id) = self.glows.get(&glow_key(glow))
        {
            attr_str(&mut self.out, "filter", &format!("url(#{id})"));
        }
        if let Some(op) = g.opacity {
            attr_num(&mut self.out, "opacity", op, p.max(2));
        }
        self.out.push_str(">\n");
        for c in &g.children {
            self.node(c);
        }
        self.out.push_str("</g>\n");
    }
}

fn clip_id(id: ClipId) -> String {
    format!("clip-{}", id.0)
}

fn glow_key(g: &Glow) -> (u64, u64) {
    (g.radius.to_bits(), g.intensity.to_bits())
}
