use geoplegma::api::DggrsApiConfig;

pub(crate) const CONFIG: DggrsApiConfig = DggrsApiConfig {
    region: true,
    children: false,
    center: false,
    neighbors: false,
    densify: false,
    area_sqm: false,
    vertex_count: false,
};
