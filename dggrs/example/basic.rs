// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.
use dggrs::error;
use dggrs::factory;
use dggrs::models::common::{DggrsUid, RefinementLevel, RelativeDepth};
use geo::{Point, Rect};

/// This is just an example and basic testing function if there is output or not
pub fn main() -> Result<(), error::port::GeoPlegmaError> {
    let dt = vec![
        DggrsUid::ISEA3HDGGRID,
        DggrsUid::IGEO7,
        DggrsUid::H3,
        DggrsUid::ISEA3HDGGAL,
        DggrsUid::IVEA3H,
        DggrsUid::IVEA9R,
        DggrsUid::IVEA3H,
        DggrsUid::RTEA9R,
        DggrsUid::RTEA3H,
    ];

    let points = vec![
        Point::new(19.96, 5.34),
        Point::new(9.06, 52.98),
        Point::new(-29.11, -15.28),
    ];

    let refinment = vec![
        RefinementLevel::new(3)?,
        RefinementLevel::new(4)?,
        RefinementLevel::new(5)?,
    ];

    let rd = RelativeDepth::new(3)?;
    let bbox = Rect::new(Point::new(-10.0, -10.0), Point::new(10.0, 10.0));

    for p in points {
        for rf in &refinment {
            for did in &dt {
                println!(
                    "=== DGGRS: {} TOOL: {} POINT: {:?} RF: {:?}===",
                    &did.spec().name,
                    &did.spec().tool,
                    &p,
                    &rf
                );
                let d = factory::dggrs_factory::get(*did).unwrap();
                let r = d.zone_from_point(*rf, p, None)?;
                println!(
                    "{:?} \nzone from point generated {} zones",
                    r.zones,
                    r.zones.len()
                );

                let zone = &r.zones[0].id;
                let r = d.zones_from_parent(rd, zone.clone(), None)?;
                println!(
                    "{:?} \nzones from parent generated {} zones",
                    r.zones,
                    r.zones.len()
                );

                let r = d.zone_from_id(zone.clone(), None)?;
                println!(
                    "{:?} \nzone from id generated {} zones",
                    r.zones,
                    r.zones.len()
                );

                let r = d.zones_from_bbox(*rf, Some(bbox), None)?;
                println!(
                    "{:?} \nzones from bbox generated {} zones",
                    r.zones,
                    r.zones.len()
                );

                let global_rf = RefinementLevel::new(1)?;
                let r = d.zones_from_bbox(global_rf, None, None)?;
                println!(
                    "{:?} \nzones from NO bbox generated {} zones",
                    r.zones,
                    r.zones.len()
                );
            }
        }
    }
    Ok(())
}
