use crate::models::aperture::Aperture;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaryI {
    pub i: u32,
    pub j: u32,
    pub k: u32,
    pub denom: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct BaryF {
    pub b0: f64,
    pub b1: f64,
    pub b2: f64,
}

impl From<BaryI> for BaryF {
    fn from(b: BaryI) -> Self {
        let d = b.denom as f64;
        Self {
            b0: b.i as f64 / d,
            b1: b.j as f64 / d,
            b2: b.k as f64 / d,
        }
    }
}

/// Snap a float bary to the nearest integer bary for a given denom.
impl BaryI {
    /// Construct from components; asserts i+j+k == denom (derived).
    pub fn new(i: u32, j: u32, k: u32) -> Self {
        let denom = i + j + k;
        Self { i, j, k, denom }
    }

    /// Center at given denominator (requires denom % 3 == 0).
    pub fn center(denom: u32) -> Self {
        assert!(denom % 3 == 0);
        let t = denom / 3;
        Self {
            i: t,
            j: t,
            k: t,
            denom,
        }
    }

    #[inline]
    pub fn min_coord(&self) -> u32 {
        self.i.min(self.j).min(self.k)
    }

    /// Add an integer delta that preserves sum==denom. Returns None if any coord would go < 0.
    pub fn add_checked(&self, di: i32, dj: i32, dk: i32) -> Option<Self> {
        debug_assert!(di + dj + dk == 0, "bary deltas must sum to 0");
        let ni = self.i as i32 + di;
        let nj = self.j as i32 + dj;
        let nk = self.k as i32 + dk;
        if ni >= 0 && nj >= 0 && nk >= 0 {
            Some(Self {
                i: ni as u32,
                j: nj as u32,
                k: nk as u32,
                denom: self.denom,
            })
        } else {
            None
        }
    }

    /// Maximum integer step t so all 6 hex vertices (±u1, ±u2, ±u3) remain inside.
    pub fn max_step_inside_i(&self) -> u32 {
        self.min_coord()
    }

    /// Convert to float bary (sum == 1.0).
    pub fn to_baryf(&self) -> BaryF {
        let d = self.denom as f64;
        BaryF {
            b0: self.i as f64 / d,
            b1: self.j as f64 / d,
            b2: self.k as f64 / d,
        }
    }

    /// Convert to 2D Cartesian via triangle vertices (b0*V0 + b1*V1 + b2*V2).
    pub fn to_cart2(&self, v0: [f64; 2], v1: [f64; 2], v2: [f64; 2]) -> [f64; 2] {
        self.to_baryf().to_cart2(v0, v1, v2)
    }

    /// Six vertices of a hex built by *barycentric integer* steps of size `t` (≤ min_coord).
    /// Returns vertices in world/cartesian using v0, v1, v2. Vertices are CCW-sorted.
    pub fn hex_vertices_bary_affine_cart2(
        &self,
        v0: [f64; 2],
        v1: [f64; 2],
        v2: [f64; 2],
        mut t: u32,
    ) -> Vec<[f64; 2]> {
        t = t.min(self.max_step_inside_i());
        const U: [(i32, i32, i32); 3] = [(1, -1, 0), (0, 1, -1), (-1, 0, 1)];
        let mut verts = Vec::with_capacity(6);
        for &(di, dj, dk) in &U {
            if let Some(p) = self.add_checked(di * (t as i32), dj * (t as i32), dk * (t as i32)) {
                verts.push(p.to_cart2(v0, v1, v2));
            }
            if let Some(p) = self.add_checked(-di * (t as i32), -dj * (t as i32), -dk * (t as i32))
            {
                verts.push(p.to_cart2(v0, v1, v2));
            }
        }
        // CCW sort around center
        let c = self.to_cart2(v0, v1, v2);
        verts.sort_by(|a, b| {
            let aa = (a[1] - c[1]).atan2(a[0] - c[0]);
            let bb = (b[1] - c[1]).atan2(b[0] - c[0]);
            aa.partial_cmp(&bb).unwrap_or(Ordering::Equal)
        });
        verts
    }

    pub fn snap_from_float(b: BaryF, denom: u32) -> Self {
        // project, round, and fix sum to denom
        let mut i = (b.b0 * denom as f64).round() as i64;
        let mut j = (b.b1 * denom as f64).round() as i64;
        let mut k = denom as i64 - i - j;
        // clamp if needed
        if i < 0 {
            k += i;
            i = 0;
        }
        if j < 0 {
            k += j;
            j = 0;
        }
        if k < 0 {
            let dk = -k; // borrow from the max of i/j
            if i >= j {
                i -= dk;
            } else {
                j -= dk;
            }
            k = 0;
        }
        Self {
            i: i as u32,
            j: j as u32,
            k: k as u32,
            denom,
        }
    }

    /// Center {1/3,1/3,1/3} at a given aperture+level.
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

    /// Helper: map a slice of integer-bary vertices to 2D cartesian.
    pub fn verts_to_cart2(
        verts: &[BaryI],
        v0: [f64; 2],
        v1: [f64; 2],
        v2: [f64; 2],
    ) -> Vec<[f64; 2]> {
        verts.iter().map(|b| b.to_cart2(v0, v1, v2)).collect()
    }

    pub fn hex_cart_ccw(
        &self,
        v0: [f64; 2],
        v1: [f64; 2],
        v2: [f64; 2],
        mut t: u32,
    ) -> Option<Vec<[f64; 2]>> {
        t = t.min(self.max_step_inside_i());
        if t == 0 {
            return None;
        }

        // ±u directions (any order is fine; we’ll sort after mapping)
        const DIRS: [(i32, i32, i32); 6] = [
            (1, -1, 0),
            (-1, 1, 0),
            (0, 1, -1),
            (0, -1, 1),
            (-1, 0, 1),
            (1, 0, -1),
        ];

        let ti = t as i32;
        let c = self.to_cart2(v0, v1, v2);
        let mut verts: Vec<[f64; 2]> = Vec::with_capacity(6);

        for (di, dj, dk) in DIRS {
            let p = self.add_checked(di * ti, dj * ti, dk * ti).unwrap();
            verts.push(p.to_cart2(v0, v1, v2));
        }

        // sort by angle around center
        verts.sort_by(|a, b| {
            let aa = (a[1] - c[1]).atan2(a[0] - c[0]);
            let bb = (b[1] - c[1]).atan2(b[0] - c[0]);
            aa.partial_cmp(&bb).unwrap()
        });

        // ensure CCW (shoelace sign)
        let mut area2 = 0.0;
        for i in 0..6 {
            let (x1, y1) = (verts[i][0], verts[i][1]);
            let (x2, y2) = (verts[(i + 1) % 6][0], verts[(i + 1) % 6][1]);
            area2 += x1 * y2 - x2 * y1;
        }
        if area2 < 0.0 {
            verts.reverse();
        }

        Some(verts)
    }

    /// Hex vertices as integer-bary, following a **given** CCW dir order.
    pub fn hex_vertices_baryi_with_dirs(
        &self,
        mut t: u32,
        dir6_ccw: &[(i32, i32, i32); 6],
    ) -> Option<[BaryI; 6]> {
        t = t.min(self.max_step_inside_i());
        if t == 0 {
            return None;
        }
        let ti = t as i32;
        let mut out: [BaryI; 6] = [*self; 6];
        for (idx, &(di, dj, dk)) in dir6_ccw.iter().enumerate() {
            out[idx] = self
                .add_checked(di * ti, dj * ti, dk * ti)
                .expect("clamped t fits");
        }
        Some(out)
    }

    /// Hex cell for this center, sized by level (Voronoi: ±½ step to neighbors).
    /// Works for A7; vertices are in CCW order after a center-angle sort.
    pub fn cell_hex_cart_ccw(
        &self,
        v0: [f64; 2],
        v1: [f64; 2],
        v2: [f64; 2],
    ) -> Option<Vec<[f64; 2]>> {
        // Center on the border has a clipped cell; you can choose to skip or clip.
        if self.min_coord() == 0 {
            return None;
        }

        // Six half-step directions (±u). We’ll sort after mapping.
        const DIRS: [(i32, i32, i32); 6] = [
            (1, -1, 0),
            (-1, 1, 0),
            (0, 1, -1),
            (0, -1, 1),
            (-1, 0, 1),
            (1, 0, -1),
        ];

        let c = self.to_cart2(v0, v1, v2);
        let denom2 = 2.0 * (self.denom as f64);
        let mut verts = Vec::with_capacity(6);

        for (di, dj, dk) in DIRS {
            // (2i+di, 2j+dj, 2k+dk) / (2D)
            let bi = (2 * self.i as i32 + di) as f64 / denom2;
            let bj = (2 * self.j as i32 + dj) as f64 / denom2;
            let bk = (2 * self.k as i32 + dk) as f64 / denom2;
            // map to world
            let p = [
                bi * v0[0] + bj * v1[0] + bk * v2[0],
                bi * v0[1] + bj * v1[1] + bk * v2[1],
            ];
            verts.push(p);
        }

        // angle-sort around the true center to guarantee proper polygon winding
        verts.sort_by(|a, b| {
            let aa = (a[1] - c[1]).atan2(a[0] - c[0]);
            let bb = (b[1] - c[1]).atan2(b[0] - c[0]);
            aa.partial_cmp(&bb).unwrap()
        });
        // ensure CCW
        let mut area2 = 0.0;
        for i in 0..6 {
            let (x1, y1) = (verts[i][0], verts[i][1]);
            let (x2, y2) = (verts[(i + 1) % 6][0], verts[(i + 1) % 6][1]);
            area2 += x1 * y2 - x2 * y1;
        }
        if area2 < 0.0 {
            verts.reverse();
        }

        Some(verts)
    }
}

impl BaryF {
    pub fn to_cart2(self, v0: [f64; 2], v1: [f64; 2], v2: [f64; 2]) -> [f64; 2] {
        [
            self.b0 * v0[0] + self.b1 * v1[0] + self.b2 * v2[0],
            self.b0 * v0[1] + self.b1 * v1[1] + self.b2 * v2[1],
        ]
    }
}

/// Regular hex in *world/cartesian* space around an integer-bary center.
/// `radius` = center→vertex distance (world units). Vertices are emitted CCW.
pub fn hex_vertices_world_regular_from_baryi(
    center: BaryI,
    v0: [f64; 2],
    v1: [f64; 2],
    v2: [f64; 2],
    radius: f64,
    pointy: bool,
) -> Vec<[f64; 2]> {
    use std::f64::consts::PI;
    let c = center.to_cart2(v0, v1, v2);
    let start = if pointy { PI / 6.0 } else { 0.0 };
    (0..6)
        .map(|i| {
            let ang = start + (i as f64) * (PI / 3.0);
            [c[0] + radius * ang.cos(), c[1] + radius * ang.sin()]
        })
        .collect()
}
