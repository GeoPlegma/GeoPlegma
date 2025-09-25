use crate::models::bary::BaryI;

#[derive(Clone, Copy, Debug)]
pub enum CellKind {
    Hex,
    Pentagon,
} // pentagon shows up only on the sphere at 12 places

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub face: u8,      // which icosa face, if/when you go spherical
    pub center: BaryI, // integer bary center at this level (i+j+k = D)
    pub level: u8,     // refinement level (D = base^level)
    pub rot: u8,       // 0..5 if you want a per-level class-II/III rotation tag
    pub kind: CellKind,
}

impl Cell {
    /// Hex vertices determined by the “shared-edge” invariant (±½ along the 6 neighbor rays).
    /// This is the unique choice that ensures adjacent cells share edges exactly.
    pub fn vertices_cart_ccw(&self, v0: [f64; 2], v1: [f64; 2], v2: [f64; 2]) -> Vec<[f64; 2]> {
        // six half-step directions (unordered)
        const DIRS: [(i32, i32, i32); 6] = [
            (1, -1, 0),
            (-1, 1, 0),
            (0, 1, -1),
            (0, -1, 1),
            (-1, 0, 1),
            (1, 0, -1),
        ];

        // lift denominator to 2D to encode “±½ step” exactly in bary floats
        let d2 = 2.0 * self.center.denom as f64;
        let cxy = self.center.to_cart2(v0, v1, v2);

        let mut verts: Vec<[f64; 2]> = DIRS
            .iter()
            .map(|&(di, dj, dk)| {
                let bi = (2 * self.center.i as i32 + di) as f64 / d2;
                let bj = (2 * self.center.j as i32 + dj) as f64 / d2;
                let bk = (2 * self.center.k as i32 + dk) as f64 / d2;
                [
                    bi * v0[0] + bj * v1[0] + bk * v2[0],
                    bi * v0[1] + bj * v1[1] + bk * v2[1],
                ]
            })
            .collect();

        // sort CCW around actual center to make a proper polygon
        verts.sort_by(|a, b| {
            let aa = (a[1] - cxy[1]).atan2(a[0] - cxy[0]);
            let bb = (b[1] - cxy[1]).atan2(b[0] - cxy[0]);
            aa.partial_cmp(&bb).unwrap()
        });

        // ensure CCW
        let mut area2 = 0.0;
        for i in 0..verts.len() {
            let (x1, y1) = (verts[i][0], verts[i][1]);
            let (x2, y2) = (
                verts[(i + 1) % verts.len()][0],
                verts[(i + 1) % verts.len()][1],
            );
            area2 += x1 * y2 - x2 * y1;
        }
        if area2 < 0.0 {
            verts.reverse();
        }
        verts
    }

    /// Build the 6 hex vertices for this cell in **CCW order**, using a precomputed
    /// per-triangle CCW ordering of the 6 ±bary directions.
    ///
    /// `dir6_ccw` must be the six direction triples (±u1, ±u2, ±u3) ordered CCW
    /// for the current triangle. Compute it once with `dir6_ccw_for_triangle(...)`
    /// and reuse for all cells in that triangle.
    ///
    /// Vertices lie at the **midpoints** to the six nearest neighbors in those directions,
    /// which guarantees adjacent cells share the exact same edge.
    pub fn vertices_cart_ccw_with_dirs(
        &self,
        v0: [f64; 2],
        v1: [f64; 2],
        v2: [f64; 2],
        dir6_ccw: &[(i32, i32, i32); 6],
    ) -> [[f64; 2]; 6] {
        // (2i+di, 2j+dj, 2k+dk) / (2D) : the midpoint in bary coords
        let d2 = 2.0 * (self.center.denom as f64);

        let mut out = [[0.0; 2]; 6];
        for (idx, &(di, dj, dk)) in dir6_ccw.iter().enumerate() {
            let bi = (2 * self.center.i as i32 + di) as f64 / d2;
            let bj = (2 * self.center.j as i32 + dj) as f64 / d2;
            let bk = (2 * self.center.k as i32 + dk) as f64 / d2;

            out[idx] = [
                bi * v0[0] + bj * v1[0] + bk * v2[0],
                bi * v0[1] + bj * v1[1] + bk * v2[1],
            ];
        }
        out
    }
}
