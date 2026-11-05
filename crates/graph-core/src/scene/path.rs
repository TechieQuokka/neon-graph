use crate::geom::Point;

/// A single path command in pixel space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathCmd {
    MoveTo(Point),
    LineTo(Point),
    Close,
}

/// A sequence of path commands (polylines and polygons).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Path {
    cmds: Vec<PathCmd>,
}

impl Path {
    pub fn builder() -> PathBuilder {
        PathBuilder::default()
    }

    /// A straight segment from `a` to `b`.
    pub fn line(a: Point, b: Point) -> Self {
        Self {
            cmds: vec![PathCmd::MoveTo(a), PathCmd::LineTo(b)],
        }
    }

    pub fn commands(&self) -> &[PathCmd] {
        &self.cmds
    }

    pub fn is_empty(&self) -> bool {
        self.cmds.is_empty()
    }
}

/// Incrementally builds a [`Path`].
#[derive(Debug, Clone, Default)]
pub struct PathBuilder {
    cmds: Vec<PathCmd>,
}

impl PathBuilder {
    pub fn with_capacity(n: usize) -> Self {
        Self {
            cmds: Vec::with_capacity(n),
        }
    }

    pub fn move_to(&mut self, p: Point) -> &mut Self {
        self.cmds.push(PathCmd::MoveTo(p));
        self
    }

    pub fn line_to(&mut self, p: Point) -> &mut Self {
        self.cmds.push(PathCmd::LineTo(p));
        self
    }

    pub fn close(&mut self) -> &mut Self {
        self.cmds.push(PathCmd::Close);
        self
    }

    pub fn build(self) -> Path {
        Path { cmds: self.cmds }
    }
}
