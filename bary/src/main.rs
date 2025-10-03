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
    //
    let center = BaryI::new(1, 1, 1, 3);
    svg.dot_bary(center, &base_triangle, colors::GREEN, 10.0, &canvas);

    let b0 = BaryI::new(1, 0, 0, 1);
    svg.dot_bary(b0, &base_triangle, colors::BASE, 12.0, &canvas);

    let b1 = BaryI::new(0, 1, 0, 1);
    svg.dot_bary(b1, &base_triangle, colors::BASE, 12.0, &canvas);

    let b2 = BaryI::new(0, 0, 1, 1);
    svg.dot_bary(b2, &base_triangle, colors::BASE, 12.0, &canvas);

    let bx = BaryI::new(1, 5, 3, 9);
    svg.dot_bary(bx, &base_triangle, colors::PEACH, 12.0, &canvas);

    let g0 = BaryI::new(0, 1, 2, 3);
    svg.dot_bary(g0, &base_triangle, colors::GREEN, 12.0, &canvas);

    let g1 = BaryI::new(0, 2, 1, 3);
    svg.dot_bary(g1, &base_triangle, colors::GREEN, 12.0, &canvas);

    let g2 = BaryI::new(1, 0, 2, 3);
    svg.dot_bary(g2, &base_triangle, colors::GREEN, 12.0, &canvas);

    let g3 = BaryI::new(2, 0, 1, 3);
    svg.dot_bary(g3, &base_triangle, colors::GREEN, 12.0, &canvas);

    let g4 = BaryI::new(1, 2, 0, 3);
    svg.dot_bary(g4, &base_triangle, colors::GREEN, 12.0, &canvas);

    let g5 = BaryI::new(2, 1, 0, 3);
    svg.dot_bary(g5, &base_triangle, colors::GREEN, 12.0, &canvas);

    let g6 = BaryI::new(1, 1, 7, 9);
    svg.dot_bary(g6, &base_triangle, colors::GREEN, 12.0, &canvas);

    let g7 = BaryI::new(1, 7, 1, 9);
    svg.dot_bary(g7, &base_triangle, colors::GREEN, 12.0, &canvas);

    let g8 = BaryI::new(7, 1, 1, 9);
    svg.dot_bary(g8, &base_triangle, colors::GREEN, 12.0, &canvas);

    let hex = BaryIHex::inscribed_hex();
    svg.hex(&hex, &base_triangle, 0.004, colors::OVERLAY1, None, &canvas);

    // Constructing the next level manually
    let c0 = BaryI::new(2, 2, 5, 9);
    svg.dot_bary(c0, &base_triangle, colors::MAUVE, 8.0, &canvas);

    let c1 = BaryI::new(2, 5, 2, 9);
    svg.dot_bary(c1, &base_triangle, colors::MAUVE, 8.0, &canvas);

    let c2 = BaryI::new(5, 2, 2, 9);
    svg.dot_bary(c2, &base_triangle, colors::MAUVE, 8.0, &canvas);

    let c3 = BaryI::new(4, 4, 1, 9);
    svg.dot_bary(c3, &base_triangle, colors::MAUVE, 8.0, &canvas);

    let c4 = BaryI::new(4, 1, 4, 9);
    svg.dot_bary(c4, &base_triangle, colors::MAUVE, 8.0, &canvas);

    let c5 = BaryI::new(1, 4, 4, 9);
    svg.dot_bary(c5, &base_triangle, colors::MAUVE, 8.0, &canvas);

    // by denominator
    let hex27 = BaryIHex::at_denom(9);
    svg.hex(&hex27, &base_triangle, 0.004, colors::MAUVE, None, &canvas);

    std::fs::write("tri.svg", svg.finish())?;
    Ok(())
}
