pub struct PolyhedronConstants;

impl PolyhedronConstants {
    /// Golden ratio for the polyhedron
    pub fn golden_ratio() -> f64 {
        (1.0 + 5.0_f64.sqrt()) / 2.0
    }
}
