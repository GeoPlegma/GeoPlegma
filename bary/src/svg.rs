use crate::canvas::Canvas;
use crate::models::bary::BaryI;
use crate::models::cart::{cPoint, cTriangle};
use std::fmt::Write;

pub struct Svg {
    buf: String,
}

impl Svg {
    /// viewBox sets the *data* coordinate system; width/height are display size.
    pub fn new_viewbox(xmin: f64, ymin: f64, w: f64, h: f64, px_w: u32, px_h: u32) -> Self {
        let mut buf = String::new();
        let _ = write!(
            buf,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{px_w}" height="{px_h}" viewBox="{xmin} {ymin} {w} {h}">"#,
        );
        Self { buf }
    }

    pub fn dot(&mut self, p: cPoint, color: &str, dot_size: f64, c: &Canvas) {
        // Hardcoded ~3px dot, independent of viewBox scale
        let (x, y) = c.map(p);
        let _ = write!(
            self.buf,
            r#"<path d="M{x},{y} L{x},{y}" stroke="{color}" stroke-linecap="round"
            stroke-width="{dot_size}" vector-effect="non-scaling-stroke"/>"#
        );
    }

    pub fn dot_bary(&mut self, b: BaryI, tri: &cTriangle, color: &str, size_px: f64, c: &Canvas) {
        let p = b.to_cpoint_on(tri);
        self.dot(p, color, size_px, c);
    }

    /// Draw a closed triangle outline at `tri` with given line width and color
    pub fn tri(&mut self, tri: &cTriangle, line_width: f64, color: &str, c: &Canvas) {
        let (a, b, c1) = (c.map(tri.v0), c.map(tri.v1), c.map(tri.v2));
        let d = format!("M{},{} L{},{} L{},{} Z", a.0, a.1, b.0, b.1, c1.0, c1.1);
        let _ = write!(
            self.buf,
            r#"<path d="{d}" stroke="{color}" stroke-width="{line_width}" strike-linecap="round" fill="none"/>"#
        );
    }

    pub fn finish(mut self) -> String {
        self.buf.push_str("</svg>");
        self.buf
    }
}
