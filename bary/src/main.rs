use plotters::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Bary {
    pub b0: f64,
    pub b1: f64,
    pub b2: f64,
}

impl Bary {
    pub fn to_cartesian2(self, v0: [f64; 2], v1: [f64; 2], v2: [f64; 2]) -> [f64; 2] {
        [
            self.b0 * v0[0] + self.b1 * v1[0] + self.b2 * v2[0],
            self.b0 * v0[1] + self.b1 * v1[1] + self.b2 * v2[1],
        ]
    }

    /// Add a barycentric delta and re-normalize to keep sum == 1.
    /// Caller should keep steps small enough so components stay >= 0
    /// (or clamp afterwards if desired).
    pub fn add(self, dx: f64, dy: f64, dz: f64) -> Self {
        let nb0 = self.b0 + dx;
        let nb1 = self.b1 + dy;
        let nb2 = self.b2 + dz;
        let s = nb0 + nb1 + nb2;
        Bary {
            b0: nb0 / s,
            b1: nb1 / s,
            b2: nb2 / s,
        }
    }

    /// Six vertices of a hex built by *barycentric steps* of size `step`,
    /// returned in *world/cartesian* coords via (v0,v1,v2).
    ///
    /// Directions follow the natural bary axes (+/-):
    ///  u1=(+1,-1,0), u2=(0,+1,-1), u3=(-1,0,+1)
    pub fn hex_vertices_bary_affine(
        self,
        v0: [f64; 2],
        v1: [f64; 2],
        v2: [f64; 2],
        step: f64,
    ) -> Vec<[f64; 2]> {
        let safe_step = step.min(self.max_step_inside());

        // unit directions in bary space
        const DIRS: [(f64, f64, f64); 3] = [(1.0, -1.0, 0.0), (0.0, 1.0, -1.0), (-1.0, 0.0, 1.0)];

        let mut verts: Vec<[f64; 2]> = DIRS
            .iter()
            .flat_map(|&(dx, dy, dz)| {
                [(dx, dy, dz), (-dx, -dy, -dz)] // positive + negative direction
            })
            .map(|(dx, dy, dz)| {
                self.add(dx * safe_step, dy * safe_step, dz * safe_step)
                    .to_cartesian2(v0, v1, v2)
            })
            .collect();

        // sort CCW around the actual center (affine-safe)
        let c = self.to_cartesian2(v0, v1, v2);
        verts.sort_by(|a, b| {
            let aa = (a[1] - c[1]).atan2(a[0] - c[0]);
            let bb = (b[1] - c[1]).atan2(b[0] - c[0]);
            aa.partial_cmp(&bb).unwrap_or(std::cmp::Ordering::Equal)
        });

        verts
    }

    pub fn max_step_inside(self) -> f64 {
        self.b0.min(self.b1).min(self.b2)
    }
}

pub struct VertexLattice {
    n: u32,
    i: u32,
    j: u32,
}

impl VertexLattice {
    pub fn new(n: u32) -> Self {
        Self { n, i: 0, j: 0 }
    }
}

impl Iterator for VertexLattice {
    type Item = Bary;
    fn next(&mut self) -> Option<Self::Item> {
        while self.i <= self.n {
            if self.j <= self.n - self.i {
                let k = self.n - self.i - self.j;
                let n = self.n as f64;
                let out = Bary {
                    b0: self.i as f64 / n,
                    b1: self.j as f64 / n,
                    b2: k as f64 / n,
                };
                self.j += 1;
                return Some(out);
            } else {
                self.i += 1;
                self.j = 0;
            }
        }
        None
    }
}

pub struct UpCentroidLattice {
    n: u32,
    i: u32,
    j: u32,
}
impl UpCentroidLattice {
    /// centroids with +1/3 offset: i+j+k = n-1
    pub fn new(n: u32) -> Self {
        assert!(n >= 1);
        Self { n, i: 0, j: 0 }
    }
}
impl Iterator for UpCentroidLattice {
    type Item = Bary;
    fn next(&mut self) -> Option<Self::Item> {
        let m = self.n - 1;
        while self.i <= m {
            if self.j <= m - self.i {
                let k = m - self.i - self.j;
                let n = self.n as f64;
                let off = 1.0 / 3.0;
                let out = Bary {
                    b0: (self.i as f64 + off) / n,
                    b1: (self.j as f64 + off) / n,
                    b2: (k as f64 + off) / n,
                };
                self.j += 1;
                return Some(out);
            } else {
                self.i += 1;
                self.j = 0;
            }
        }
        None
    }
}

pub struct DownCentroidLattice {
    n: u32,
    i: u32,
    j: u32,
}
impl DownCentroidLattice {
    /// centroids with +2/3 offset: i+j+k = n-2
    pub fn new(n: u32) -> Self {
        assert!(n >= 2);
        Self { n, i: 0, j: 0 }
    }
}
impl Iterator for DownCentroidLattice {
    type Item = Bary;
    fn next(&mut self) -> Option<Self::Item> {
        let m = self.n - 2;
        while self.i <= m {
            if self.j <= m - self.i {
                let k = m - self.i - self.j;
                let n = self.n as f64;
                let off = 2.0 / 3.0;
                let out = Bary {
                    b0: (self.i as f64 + off) / n,
                    b1: (self.j as f64 + off) / n,
                    b2: (k as f64 + off) / n,
                };
                self.j += 1;
                return Some(out);
            } else {
                self.i += 1;
                self.j = 0;
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug)]
pub enum HexMode {
    /// Regular hex in *world/cartesian* space (no affine distortion).
    /// `radius` = center→vertex distance in world units.
    WorldRegular { radius: f64, pointy: bool },

    /// Hex defined by *barycentric* steps (aligns to bary axes; affinely distorted unless the
    /// triangle is equilateral). `step` is dimensionless (e.g., 1.0/n).
    BaryAffine { step: f64 },
}

pub fn hex_vertices_world_regular_from_bary(
    center_bary: Bary,
    v0: [f64; 2],
    v1: [f64; 2],
    v2: [f64; 2],
    radius: f64,
    pointy: bool, // true = pointy-top, false = flat-top
) -> Vec<[f64; 2]> {
    use std::f64::consts::PI;
    let c = center_bary.to_cartesian2(v0, v1, v2);
    let start = if pointy { PI / 6.0 } else { 0.0 };
    (0..6)
        .map(|i| {
            let ang = start + (i as f64) * (PI / 3.0);
            [c[0] + radius * ang.cos(), c[1] + radius * ang.sin()]
        })
        .collect()
}

pub fn hex_from_bary(
    center_bary: Bary,
    v0: [f64; 2],
    v1: [f64; 2],
    v2: [f64; 2],
    mode: HexMode,
) -> Vec<[f64; 2]> {
    match mode {
        HexMode::WorldRegular { radius, pointy } => {
            hex_vertices_world_regular_from_bary(center_bary, v0, v1, v2, radius, pointy)
        }
        HexMode::BaryAffine { step } => center_bary.hex_vertices_bary_affine(v0, v1, v2, step),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("tri_grid.png", (1100, 1100)).into_drawing_area();
    root.fill(&WHITE)?;

    let v0 = [0.0, 0.0];
    let v1 = [1.0, 0.0];
    let v2 = [0.5, (3.0f64).sqrt() / 2.0];

    // scale up for plotting
    let scale = 1000.0;
    let offset = (50.0, 1050.0);

    let to_screen = |p: [f64; 2]| -> (i32, i32) {
        (
            (p[0] * scale + offset.0) as i32,
            (offset.1 - p[1] * scale) as i32,
        )
    };

    // base triangle
    let tri_pts = vec![to_screen(v0), to_screen(v1), to_screen(v2)];
    let mut outline = tri_pts.clone(); // tri_pts: Vec<(i32, i32)>
    outline.push(to_screen(v0)); // close the loop
    root.draw(&PathElement::new(
        outline,
        ShapeStyle::from(&RGBColor(120, 120, 120)).stroke_width(2),
    ))?;

    let subdivision = 4;

    // Choose how to build hexes:
    // let mode = HexMode::WorldRegular { radius: 0.020, pointy: true };
    let mode = HexMode::BaryAffine {
        step: 1.0 / (subdivision as f64 * 3.0),
    };

    // helper to draw one hex (verts in world space)
    let mut draw_hex = |verts: Vec<[f64; 2]>| -> Result<(), Box<dyn std::error::Error>> {
        let mut poly_px: Vec<(i32, i32)> = verts.iter().map(|&p| to_screen(p)).collect();
        if poly_px.len() != 6 {
            return Ok(());
        } // guard (shouldn't happen)
        // outline (close loop)
        poly_px.push(poly_px[0]);
        root.draw(&PathElement::new(
            poly_px,
            ShapeStyle::from(&RGBColor(80, 80, 80)).stroke_width(1),
        ))?;
        Ok(())
    };

    // tiny epsilon to skip edge centers (hex would collapse)
    let eps = 1e-9;

    // 1) hex at every vertex-lattice point
    for b in VertexLattice::new(subdivision) {
        // skip if on/near edge (min bary too small)
        if b.b0.min(b.b1).min(b.b2) <= eps {
            continue;
        }
        let verts = hex_from_bary(b, v0, v1, v2, mode);
        draw_hex(verts)?;
    }

    // 2) hex at every "up" centroid
    for b in UpCentroidLattice::new(subdivision) {
        if b.b0.min(b.b1).min(b.b2) <= eps {
            continue;
        }
        let verts = hex_from_bary(b, v0, v1, v2, mode);
        draw_hex(verts)?;
    }

    // 3) hex at every "down" centroid
    for b in DownCentroidLattice::new(subdivision) {
        if b.b0.min(b.b1).min(b.b2) <= eps {
            continue;
        }
        let verts = hex_from_bary(b, v0, v1, v2, mode);
        draw_hex(verts)?;
    }

    for b in VertexLattice::new(subdivision) {
        let p = b.to_cartesian2(v0, v1, v2);
        root.draw(&Circle::new(to_screen(p), 3, RED.filled()))?;
    }

    for b in UpCentroidLattice::new(subdivision) {
        let p = b.to_cartesian2(v0, v1, v2);
        root.draw(&Circle::new(to_screen(p), 3, BLUE.filled()))?;
    }

    for b in DownCentroidLattice::new(subdivision) {
        let p = b.to_cartesian2(v0, v1, v2);
        root.draw(&Circle::new(to_screen(p), 3, GREEN.filled()))?;
    }

    Ok(())
}
