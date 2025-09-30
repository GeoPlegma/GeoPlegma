#[derive(Clone, Copy, Debug, PartialEq)]
pub struct cPoint {
    pub x: f64,
    pub y: f64,
}

impl cPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct cTriangle {
    pub v0: cPoint,
    pub v1: cPoint,
    pub v2: cPoint,
}

impl cTriangle {
    pub fn new(p0: cPoint, p1: cPoint, p2: cPoint) -> Self {
        Self {
            v0: p0,
            v1: p1,
            v2: p2,
        }
    }

    pub fn path(&self) -> cPath {
        cPath([self.v0, self.v1, self.v2, self.v0])
    }
}

pub struct cPath([cPoint; 4]);

impl cPath {
    pub fn to_plot(&self, scale: u32, pad: u32, img_h: u32) -> [(i32, i32); 4] {
        self.0.map(|p| {
            let x = (p.x * scale as f64).round() as u32 + pad;
            let y = img_h - pad - (p.y * scale as f64).round() as u32;
            (x as i32, y as i32)
        })
    }
}
