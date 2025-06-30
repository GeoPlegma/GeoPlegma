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
    let dggrs: DGGRS = DGGRS::new(&dggal, "IVEA9R").expect("Unknown DGGRS");

    let pnt: GeoPoint = GeoPoint {
        lat: 52.3,
        lon: 12.3,
    };

    let zone = dggrs.getZoneFromWGS84Centroid(12, &pnt);
    println!("{:?}", zone);

    let kids = dggrs.getSubZones(zone, 1);

    println!("{:?}", kids);

    let ll: GeoPoint = GeoPoint {
        lat: 14.5,
        lon: 14.5,
    };
    let ur: GeoPoint = GeoPoint {
        lat: 20.3,
        lon: 20.3,
    };

    let mut bbox = GeoExtent { ll, ur };
    println!("{:?}", bbox);
    let mut options = HashMap::<&str, &str>::new();

    let mut exit_code: i32 = 0;

    if parse_bbox(&options, &mut bbox) {
        exit_code = 1
    }
    println!("{:?}", bbox);
    println!("{:?}", wholeWorld);

    let zones = dggrs.listZones(2, &bbox);

    // println!("{:?}", zones);
    println!("{:?}", zones.len());

    println!("here");

    std::process::exit(0);

    let configs = vec![
        (
            String::from("DGGRID"),
            String::from("ISEA3H"),
            String::from("03a000000000000000"),
        ),
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
