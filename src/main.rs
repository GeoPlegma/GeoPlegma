extern crate ecrt;
use ecrt::{Application, tokenizeWith};
extern crate dggal;
use dggal::{DGGAL, DGGRS, GeoExtent, GeoPoint, wholeWorld};
use geo::geometry::Point;
use geo_plegmata::dggrs;
use std;
use std::env;

use std::collections::HashMap;
use std::f64::consts::PI;

fn main() {
    let args: Vec<String> = env::args().collect();
    let my_app = Application::new(&args);
    let dggal = DGGAL::new(&my_app);
    let dggrs: DGGRS = DGGRS::new(&dggal, "IVEA3H").expect("Unknown DGGRS");

    let max_depth = dggrs.getMaxDepth();
    println!("The maximum depth of this dggrs is: \n{:?}\n\n", max_depth);

    let pnt: GeoPoint = GeoPoint {
        lat: 52.3,
        lon: 12.3,
    };

    let zone = dggrs.getZoneFromWGS84Centroid(8, &pnt);
    println!("The Zone ID for the given point is: \n{:?}\n\n", zone);

    let zone_level = dggrs.getZoneLevel(zone);
    println!("The level of that Zone is: \n{:?}\n\n", zone_level);

    let mut nb_types: [i32; 6] = [0; 6];
    let neighbors = dggrs.getZoneNeighbors(zone, &mut nb_types);
    println!("The neighbors of that zone: \n{:?}\n\n", neighbors);

    if zone_level + 1 > max_depth {
        println!(
            "Zone level: {:?} is already max depth {:?}",
            zone_level, max_depth
        );
    } else {
        let kids = dggrs.getSubZones(zone, 1);
        println!("The children of that zone: \n{:?}\n\n", kids);
    }

    let vertices: Vec<GeoPoint> = dggrs.getZoneWGS84Vertices(zone);
    print!("These are the vertices of the zone: \n{:?}\n\n", vertices);

    let rvertices: Vec<GeoPoint> = dggrs.getZoneRefinedWGS84Vertices(zone, 0);
    print!(
        "These are the refined vertices of the zone: \n{:?}\n\n",
        rvertices
    );

    let ll: GeoPoint = GeoPoint {
        lat: 14.5,
        lon: 14.5,
    };
    let ur: GeoPoint = GeoPoint {
        lat: 20.3,
        lon: 20.3,
    };

    let bbox = GeoExtent { ll, ur };
    println!("The extent of the bbox: \n{:?}\n\n", bbox);
    // let mut options = HashMap::<&str, &str>::new();

    // let mut exit_code: i32 = 0;

    // if parse_bbox(&options, &mut bbox) {
    //     exit_code = 1
    // }
    // println!("{:?}", bbox);
    println!("The extent of the whole world: \n{:?}\n\n", wholeWorld);

    let zones = dggrs.listZones(2, &wholeWorld);
    println!(
        "The length of the array of zone IDs for the whole world: \n{:?}\n\n",
        zones.len()
    );

    let subzones = dggrs.getSubZones(zone, 5);
    println!(
        "The length of the array of zone IDs of a parent zone: \n{:?}\n\n",
        subzones.len()
    );

    use std::time::Instant;
    let t0 = Instant::now();
    use rayon::prelude::*;
    let ga: Vec<_> = subzones
        .iter() // WARN: par_iter does not work because the underlying ecere/dggal C FFI is not threat safe.
        .map(|zone: &u64| {
            let z: u64 = *zone;
            //println!("{:?}", z);
            //let my_app2 = Application::new(&args);
            //let dggal2 = DGGAL::new(&my_app2);
            //let dggrs2: DGGRS = DGGRS::new(&dggal2, "IVEA9R").expect("Unknown DGGRS");
            dggrs.getZoneWGS84Vertices(z)
        })
        .collect();
    println!("getZoneWGS84Verticies() took {:.2?}", t0.elapsed());

    //println!("{:?}", ga);

    println!("here");

    let configs = vec![
        (
            String::from("DGGAL"),
            String::from("IVEA3H"),
            String::from("1297036692682743552"),
        ),
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
        // (
        //     String::from("H3O"),
        //     String::from("H3"),
        //     String::from("811fbffffffffff"),
        // ),
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
        let result = generator.zones_from_bbox(2, false, None); // NOTE: no bbox = global
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

fn parse_bbox(options: &HashMap<&str, &str>, bbox: &mut GeoExtent) -> bool {
    let mut result = true;
    let bbox_option = options.get(&"bbox");
    if bbox_option != None {
        let s = bbox_option.unwrap();
        // NOTE: tokenizeWith() will eventually be moved to ecrt crate
        let tokens: Vec<String> = tokenizeWith::<4>(s, ",", false);
        result = false;
        if tokens.len() == 4 {
            let a = tokens[0].parse::<f64>();
            let b = tokens[1].parse::<f64>();
            let c = tokens[2].parse::<f64>();
            let d = tokens[3].parse::<f64>();
            if a.is_ok() && b.is_ok() && c.is_ok() && d.is_ok() {
                let af = a.unwrap();
                let bf = b.unwrap();
                let cf = c.unwrap();
                let df = d.unwrap();
                if af < 90.0 && af > -90.0 {
                    bbox.ll = GeoPoint {
                        lat: af * PI / 180.0,
                        lon: bf * PI / 180.0,
                    };
                    bbox.ur = GeoPoint {
                        lat: cf * PI / 180.0,
                        lon: df * PI / 180.0,
                    };
                    result = true;
                } else {
                    result = false;
                }
            } else {
                result = false;
            }
        }
        if result == false {
            println!("Invalid bounding box specified");
        }
    }
    return result;
}
