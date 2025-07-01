use crate::models::common::{Zone, ZoneID, Zones};
use dggal::{DGGRS, DGGRSZone};

pub fn ids_to_zones(d: DGGRS, ids: Vec<DGGRSZone>) -> Zones {
    let zones = ids
        .into_iter()
        .map(|id| {
            let count_edges = d.countZoneEdges(id);
            Zone {
                id: ZoneID { id: id.to_string() },
                region,
                vertex_count,
                center: center_point,
                children: children_opt,
                neighbors: neighbors_opt,
            }
        })
        .collect();

    Zones { zones }
}
