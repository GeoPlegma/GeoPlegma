// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Luís Moreira de Sousa, Técnico, ULisboa 
// (luis.moreira.de.sousa [at] tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use geo::Point;
use api::models::common::RefinementLevel;
use gp_dggrs::sys_api::DggrsSysApi;
use gp_dggrs::impls::ivea3h_bary::IVEA3HBary;

/// This is just an example and basic testing function if there is output or not
//pub fn main() -> Result<(), std::io::Error> {
pub fn main() {

    let p1 = Point::new(0.45, 0.22);
    let p2 = Point::new(0.21, 0.64);
    let p3 = Point::new(0.75, 0.11);

    let system = IVEA3HBary {};
    let mut level = RefinementLevel::new(3).unwrap();

    println!("== Resolution 3 ==");
    println!("Point 1 {} {}", p1.x(), p1.y());
    let zone1 = system.zone_from_point(level, p1);
    assert_eq!(zone1.0, 5);
    assert_eq!(zone1.1, 2);   

    println!("Point 2 {} {}", p2.x(), p2.y());
    let zone2 = system.zone_from_point(level, p2);
    assert_eq!(zone2.0, 2);
    assert_eq!(zone2.1, 5);   
    
    let level = RefinementLevel::new(4).unwrap();

    println!("\n== Resolution 4 ==");
    println!("Point 1 {} {}", p1.x(), p1.y());
    let zone3 = system.zone_from_point(level, p1);
    assert_eq!(zone3.0, 4);
    assert_eq!(zone3.1, 2);   

    println!("Point 2 {} {}", p2.x(), p2.y());
    let zone4 = system.zone_from_point(level, p2);
    assert_eq!(zone4.0, 2);
    assert_eq!(zone4.1, 6);   
    
    let level = RefinementLevel::new(5).unwrap();

    println!("\n== Resolution 5 ==");
    println!("Point 1 {} {}", p1.x(), p1.y());
    let zone3 = system.zone_from_point(level, p1);
    assert_eq!(zone3.0, 12);
    assert_eq!(zone3.1, 6);   

    println!("Point 2 {} {}", p2.x(), p2.y());
    let zone4 = system.zone_from_point(level, p2);
    assert_eq!(zone4.0, 5);
    assert_eq!(zone4.1, 17);   
    
    let level = RefinementLevel::new(6).unwrap();

    println!("\n== Resolution 6 ==");
    println!("Point 1 {} {}", p1.x(), p1.y());
    let zone3 = system.zone_from_point(level, p1);
    assert_eq!(zone3.0, 12);
    assert_eq!(zone3.1, 6);   

    println!("Point 2 {} {}", p2.x(), p2.y());
    let zone4 = system.zone_from_point(level, p2);
    assert_eq!(zone4.0, 6);
    assert_eq!(zone4.1, 17);   

    println!("Point 3 {} {}", p3.x(), p3.y());
    let zone5 = system.zone_from_point(level, p3);
    assert_eq!(zone5.0, 20);
    assert_eq!(zone5.1, 3);   
}
