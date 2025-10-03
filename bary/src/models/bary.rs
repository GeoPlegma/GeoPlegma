use crate::models::cart::{CPoint, CTriangle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaryI {
    pub i: u32,
    pub j: u32,
    pub k: u32,
    pub denom: u32,
}

impl BaryI {
    pub fn new(i: u32, j: u32, k: u32, denom: u32) -> Self {
        assert!(i + j + k == denom, "i+j+k must equal denom");
        Self { i, j, k, denom }
    }

    pub fn to_cpoint_on(&self, tri: &CTriangle) -> CPoint {
        assert!(self.denom != 0, "BaryI.denom must be > 0");
        let d = self.denom as f64;
        let w0 = self.i as f64 / d;
        let w1 = self.j as f64 / d;
        let w2 = self.k as f64 / d;

        CPoint::new(
            w0 * tri.v0.x + w1 * tri.v1.x + w2 * tri.v2.x,
            w0 * tri.v0.y + w1 * tri.v1.y + w2 * tri.v2.y,
        )
    }

    pub fn scale(&self, f: u32) -> Self {
        BaryI::new(self.i * f, self.j * f, self.k * f, self.denom * f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaryIHex(pub [BaryI; 6]);

impl BaryIHex {
    pub fn inscribed_hex() -> Self {
        Self([
            BaryI::new(2, 1, 0, 3),
            BaryI::new(1, 2, 0, 3),
            BaryI::new(0, 2, 1, 3),
            BaryI::new(0, 1, 2, 3),
            BaryI::new(1, 0, 2, 3),
            BaryI::new(2, 0, 1, 3),
        ])
    }

    pub fn inscribed_at(denom: u32) -> Self {
        assert!(denom % 3 == 0, "denom must be divisible by 3");
        let t = denom / 3;

        Self([
            // (t+2, t-1, t-1) and permutations
            BaryI::new(t + 2, t - 1, t - 1, denom),
            BaryI::new(t - 1, t + 2, t - 1, denom),
            BaryI::new(t - 1, t - 1, t + 2, denom),
            // (t+1, t+1, t-1) and permutations
            BaryI::new(t + 1, t + 1, t - 1, denom),
            BaryI::new(t + 1, t - 1, t + 1, denom),
            BaryI::new(t - 1, t + 1, t + 1, denom),
        ])
    }

    pub fn hex_from_center(center: BaryI, h: u32) -> Self {
        let d = center.denom;
        let (i, j, k) = (center.i as i32, center.j as i32, center.k as i32);
        let h = h as i32;

        Self([
            BaryI::new((i + h) as u32, j as u32, (k - h) as u32, d),
            BaryI::new(i as u32, (j + h) as u32, (k - h) as u32, d),
            BaryI::new((i - h) as u32, (j + h) as u32, k as u32, d),
            BaryI::new((i - h) as u32, j as u32, (k + h) as u32, d),
            BaryI::new(i as u32, (j - h) as u32, (k + h) as u32, d),
            BaryI::new((i + h) as u32, (j - h) as u32, k as u32, d),
        ])
    }

    pub fn at_denom(denom: u32) -> Self {
        assert!(denom % 3 == 0, "denom must be divisible by 3");
        if denom == 3 {
            return Self([
                BaryI::new(2, 1, 0, 3),
                BaryI::new(1, 2, 0, 3),
                BaryI::new(0, 2, 1, 3),
                BaryI::new(0, 1, 2, 3),
                BaryI::new(1, 0, 2, 3),
                BaryI::new(2, 0, 1, 3),
            ]);
        }

        let t = denom / 3;
        assert!(t >= 2, "for denom >= 9 we need t=denom/3 >= 2");

        // Three of type (t+2, t-1, t-1)
        let a = [
            BaryI::new(t + 2, t - 1, t - 1, denom),
            BaryI::new(t - 1, t + 2, t - 1, denom),
            BaryI::new(t - 1, t - 1, t + 2, denom),
        ];
        // Three of type (t+1, t+1, t-2)
        let b = [
            BaryI::new(t + 1, t + 1, t - 2, denom),
            BaryI::new(t + 1, t - 2, t + 1, denom),
            BaryI::new(t - 2, t + 1, t + 1, denom),
        ];

        Self([a[2], b[1], a[0], b[0], a[1], b[2]])
    }

    pub fn to_cpoints_on(&self, tri: &CTriangle) -> [CPoint; 6] {
        self.0.map(|b| b.to_cpoint_on(tri))
    }
}
