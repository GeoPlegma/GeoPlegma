use crate::canvas::Canvas;
use crate::models::bary::BaryI;
use crate::models::bary::BaryIHex;
use crate::models::cart::{CPoint, CTriangle};
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

    pub fn dot(&mut self, p: CPoint, color: &str, dot_size: f64, c: &Canvas) {
        // Hardcoded ~3px dot, independent of viewBox scale
        let (x, y) = c.map(p);
        let _ = write!(
            self.buf,
            r#"<path d="M{x},{y} L{x},{y}" stroke="{color}" stroke-linecap="round"
            stroke-width="{dot_size}" vector-effect="non-scaling-stroke"/>"#
        );
    }

    pub fn dot_bary(&mut self, b: BaryI, tri: &CTriangle, color: &str, size_px: f64, c: &Canvas) {
        let p = b.to_cpoint_on(tri);
        self.dot(p, color, size_px, c);
    }

    /// Draw a closed triangle outline at `tri` with given line width and color
    pub fn tri(&mut self, tri: &CTriangle, line_width: f64, color: &str, c: &Canvas) {
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
    pub fn hex(
        &mut self,
        hex: &BaryIHex,
        tri: &CTriangle,
        line_width: f64,
        color: &str,
        fill: Option<&str>,
        c: &Canvas,
    ) {
        let pts = hex.to_cpoints_on(tri).map(|p| c.map(p));
        let d = format!(
            "M{},{} L{},{} L{},{} L{},{} L{},{} L{},{} Z",
            pts[0].0,
            pts[0].1,
            pts[1].0,
            pts[1].1,
            pts[2].0,
            pts[2].1,
            pts[3].0,
            pts[3].1,
            pts[4].0,
            pts[4].1,
            pts[5].0,
            pts[5].1
        );
        match fill {
            Some(f) => {
                let _ = write!(
                    self.buf,
                    r#"<path d="{d}" stroke="{color}" stroke-width="{line_width}" fill="{f}"/>"#
                );
            }
            None => {
                let _ = write!(
                    self.buf,
                    r#"<path d="{d}" stroke="{color}" stroke-width="{line_width}" fill="none"/>"#
                );
            }
        }
    }
}
