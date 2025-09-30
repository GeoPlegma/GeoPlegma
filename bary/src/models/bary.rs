use crate::models::aperture::Aperture;
use crate::models::cart::{cPoint, cTriangle};
use std::cmp::Ordering;

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

    pub fn center_for(ap: Aperture, level: u32) -> Self {
        let d = ap.denom_for_level(level);
        assert!(d % 3 == 0, "center needs denom divisible by 3");
        let t = d / 3;
        Self {
            i: t,
            j: t,
            k: t,
            denom: d,
        }
    }
    #[inline]
    pub fn to_cpoint_on(&self, tri: &cTriangle) -> cPoint {
        assert!(self.denom != 0, "BaryI.denom must be > 0");
        let d = self.denom as f64;
        let w0 = self.i as f64 / d;
        let w1 = self.j as f64 / d;
        let w2 = self.k as f64 / d;

        cPoint::new(
            w0 * tri.v0.x + w1 * tri.v1.x + w2 * tri.v2.x,
            w0 * tri.v0.y + w1 * tri.v1.y + w2 * tri.v2.y,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaryIHex(pub [BaryI; 6]);

impl BaryIHex {
    pub fn inscribed_with_denom(denom: u32) -> Self {
        assert!(denom % 3 == 0, "denom must be divisible by 3");
        let t = denom / 3;
        let mk = |i, j, k| BaryI::new(i, j, k, denom);
        Self([
            mk(2 * t, 1 * t, 0),
            mk(1 * t, 2 * t, 0),
            mk(0, 2 * t, 1 * t),
            mk(0, 1 * t, 2 * t),
            mk(1 * t, 0, 2 * t),
            mk(2 * t, 0, 1 * t),
        ])
    }

    #[inline]
    pub fn to_cpoints_on(&self, tri: &cTriangle) -> [cPoint; 6] {
        self.0.map(|b| b.to_cpoint_on(tri))
    }
}
