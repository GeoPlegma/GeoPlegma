use geo::geometry::Point;
use geo_plegmata::dggrs;
fn main() {
    let configs = vec![
        // (
        //     String::from("DGGRID"),
        //     String::from("ISEA3H"),
        //     String::from("03a000000000000000"),
        // ),
        // (
        //     String::from("DGGRID"),
        //     String::from("IGEO7"),
        //     String::from("054710bfffffffffff"),
        // ),
        (
            String::from("H3O"),
            String::from("H3"),
            String::from("811fbffffffffff"),
        ),
    ];

    let bbox: Option<Vec<Vec<f64>>> = Some(vec![
        vec![-77.0, 39.0], // lower left
        vec![-76.0, 40.0], // upper right
    ]);

    let pnt = Point::new(10.9, 4.9);
    for (tool, dggs, zone_id) in configs {
        println!("=== DGGS Type: {} ===", dggs);

        let generator = dggrs::get(&tool, &dggs);

        println!("Global");
        let result = generator.zones_from_bbox(2, false, None);
        println!(
            "{:?} \nGenerated {} zones",
            result.zones,
            result.zones.len()
        );

        println!("Global with Bbox");
        let result = generator.zones_from_bbox(2, false, bbox.clone());
        println!(
            "{:?} \nGenerated {} zones",
            result.zones,
            result.zones.len()
        );

        println!("Point");
        let result = generator.zone_from_point(6, pnt, false);
        println!(
            "{:?} \nGenerated {} zones",
            result.zones,
            result.zones.len()
        );

        println!("Subzones of {}", zone_id);
        let result = generator.zones_from_parent(6, zone_id.clone(), false);
        println!(
            "{:?} \nGenerated {} zones",
            result.zones,
            result.zones.len()
        );

        println!("Single Zone {}", zone_id.clone());
        let result = generator.zone_from_id(zone_id.clone(), false);
        println!(
            "{:?} \nGenerated {} zones",
            result.zones,
            result.zones.len()
        );
    }
}
