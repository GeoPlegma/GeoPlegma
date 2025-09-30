mod models;
use models::bary::{BaryI, BaryIHex};
use models::cart::{CPoint, CTriangle};
mod svg;
use svg::Svg;
mod canvas;
use canvas::Canvas;
mod colors;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p0 = CPoint::new(0.0, 0.0);
    let p1 = CPoint::new(1.0, 0.0);
    let p2 = CPoint::new(0.5, (3.0f64).sqrt() / 2.0);

    let base_triangle = CTriangle::new(p0, p1, p2);

    let mut svg = Svg::new_viewbox(0.0, -1.0, 1.0, 1.1, 1000, 1000);
    let canvas = Canvas::y_up();

    svg.tri(&base_triangle, 0.005, colors::BASE, &canvas);
    // svg.dot(p0, colors::SAPPHIRE, 12.0, &canvas);
    // svg.dot(p1, colors::TEAL, 12.0, &canvas);
    // svg.dot(p2, colors::FLAMINGO, 12.0, &canvas);

    let b0 = BaryI::new(1, 0, 0, 1);
    svg.dot_bary(b0, &base_triangle, colors::RED, 12.0, &canvas);

    let b1 = BaryI::new(0, 1, 0, 1);
    svg.dot_bary(b1, &base_triangle, colors::RED, 12.0, &canvas);

    let b2 = BaryI::new(0, 0, 1, 1);
    svg.dot_bary(b2, &base_triangle, colors::RED, 12.0, &canvas);

    let bx = BaryI::new(1, 5, 3, 9);
    svg.dot_bary(bx, &base_triangle, colors::PEACH, 12.0, &canvas);

    let h0 = BaryI::new(0, 1, 2, 3);
    svg.dot_bary(h0, &base_triangle, colors::MANTLE, 12.0, &canvas);

    let h1 = BaryI::new(0, 2, 1, 3);
    svg.dot_bary(h1, &base_triangle, colors::MANTLE, 12.0, &canvas);

    let h2 = BaryI::new(1, 0, 2, 3);
    svg.dot_bary(h2, &base_triangle, colors::MANTLE, 12.0, &canvas);

    let h3 = BaryI::new(2, 0, 1, 3);
    svg.dot_bary(h3, &base_triangle, colors::MANTLE, 12.0, &canvas);

    let h4 = BaryI::new(1, 2, 0, 3);
    svg.dot_bary(h4, &base_triangle, colors::MANTLE, 12.0, &canvas);

    let h5 = BaryI::new(2, 1, 0, 3);
    svg.dot_bary(h5, &base_triangle, colors::MANTLE, 12.0, &canvas);

    let hex = BaryIHex::inscribed_hex();
    svg.hex(&hex, &base_triangle, 0.004, colors::OVERLAY1, None, &canvas);

    let chd = BaryIHex::hex_from_center(
        BaryI {
            i: 1,
            j: 1,
            k: 1,
            denom: 3,
        },
        3,
    );
    svg.hex(&chd, &base_triangle, 0.004, colors::PINK, None, &canvas);

    std::fs::write("tri.svg", svg.finish())?;
    Ok(())
}
