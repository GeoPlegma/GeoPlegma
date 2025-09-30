#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CPoint {
    pub x: f64,
    pub y: f64,
}

impl CPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CTriangle {
    pub v0: CPoint,
    pub v1: CPoint,
    pub v2: CPoint,
}

impl CTriangle {
    pub fn new(p0: CPoint, p1: CPoint, p2: CPoint) -> Self {
        Self {
            v0: p0,
            v1: p1,
            v2: p2,
        }
    }

    pub fn path(&self) -> CPath {
        CPath([self.v0, self.v1, self.v2, self.v0])
    }
}

pub struct CPath([CPoint; 4]);

impl CPath {
    pub fn to_plot(&self, scale: u32, pad: u32, img_h: u32) -> [(i32, i32); 4] {
        self.0.map(|p| {
            let x = (p.x * scale as f64).round() as u32 + pad;
            let y = img_h - pad - (p.y * scale as f64).round() as u32;
            (x as i32, y as i32)
        })
    }
}
