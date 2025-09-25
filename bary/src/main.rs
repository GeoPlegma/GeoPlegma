use plotters::prelude::*;
mod models;
use models::aperture::Aperture;
use models::bary::{BaryI, hex_vertices_world_regular_from_baryi};

// ======== tiny utils just for debugging ========

fn angle(p: [f64; 2], c: [f64; 2]) -> f64 {
    (p[1] - c[1]).atan2(p[0] - c[0])
}

fn poly_area2(verts: &[[f64; 2]]) -> f64 {
    let mut a = 0.0;
    for i in 0..verts.len() {
        let (x1, y1) = (verts[i][0], verts[i][1]);
        let (x2, y2) = (
            verts[(i + 1) % verts.len()][0],
            verts[(i + 1) % verts.len()][1],
        );
        a += x1 * y2 - x2 * y1;
    }
    a
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // pick aperture & level
    let ap = Aperture::A3;
    let level = 1;
    let denom = ap.denom_for_level(level);
    let frequency = 1;

    println!("=== DEBUG ===");
    println!(
        "aperture = {:?}, level = {}, denom = {}",
        ap, level, frequency
    );

    let root = BitMapBackend::new("tri_grid.png", (1100, 1100)).into_drawing_area();
    root.fill(&WHITE)?;

    // world triangle (equilateral unit-ish)
    let v0 = [0.0, 0.0];
    let v1 = [1.0, 0.0];
    let v2 = [0.5, (3.0f64).sqrt() / 2.0];

    // orientation
    let tri_area2 = (v1[0] - v0[0]) * (v2[1] - v0[1]) - (v2[0] - v0[0]) * (v1[1] - v0[1]);
    println!(
        "triangle signed area*2 = {} ({}-winding)",
        tri_area2,
        if tri_area2 > 0.0 { "CCW" } else { "CW" }
    );

    // screen mapping
    let scale = 1000.0;
    let offset = (50.0, 1050.0);
    let to_screen = |p: [f64; 2]| -> (i32, i32) {
        (
            (p[0] * scale + offset.0) as i32,
            (offset.1 - p[1] * scale) as i32,
        )
    };

    // base triangle outline
    let tri_pts = vec![to_screen(v0), to_screen(v1), to_screen(v2)];
    let mut outline = tri_pts.clone();
    outline.push(to_screen(v0));
    root.draw(&PathElement::new(
        outline,
        ShapeStyle::from(&RGBColor(120, 120, 120)).stroke_width(2),
    ))?;

    // draw centers + labels (as before)
    for i in 0..=frequency {
        for j in 0..=(frequency - i) {
            let k = frequency - i - j;
            let b = BaryI {
                i,
                j,
                k,
                denom: frequency,
            };
            let p = b.to_cart2(v0, v1, v2);
            let (x, y) = to_screen(p);
            root.draw(&Circle::new((x, y), 2, RED.filled()))?;
            // label only a subset to keep the image readable at higher levels
            let label = format!("({}, {}, {})", b.i, b.j, b.k);
            root.draw(&Text::new(
                label,
                (x + 8, y - 8),
                ("sans-serif", 16).into_font().color(&RED),
            ))?;
        }
    }

    let c = BaryI {
        i: 0,
        j: 1,
        k: 1,
        denom: frequency,
    };
    let p = c.to_cart2(v0, v1, v2);
    let (x, y) = to_screen(p);
    root.draw(&Circle::new((x, y), 2, GREEN.filled()))?;

    root.present()?;
    println!("wrote tri_grid.png");
    Ok(())
}
