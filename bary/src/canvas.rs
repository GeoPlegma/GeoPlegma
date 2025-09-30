use crate::models::cart::cPoint;

pub struct Canvas {
    pub y_up: bool,
}
impl Canvas {
    pub fn y_up() -> Self {
        Self { y_up: true }
    }
    #[inline]
    pub fn map(&self, p: cPoint) -> (f64, f64) {
        let y = if self.y_up { -p.y } else { p.y };
        (p.x, y)
    }
}
