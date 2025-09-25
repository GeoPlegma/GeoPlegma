#[derive(Clone, Copy, Debug)]
pub enum Aperture {
    A3,
    A4,
    A7,
}

impl Aperture {
    /// Radix for the denominator growth per level.
    pub fn base(self) -> u32 {
        match self {
            Aperture::A3 => 3,
            Aperture::A4 => 4,
            Aperture::A7 => 7,
        }
    }
    pub fn denom_for_level(self, level: u32) -> u32 {
        self.base().pow(level)
    }
}
