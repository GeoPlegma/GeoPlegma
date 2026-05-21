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

pub(crate) const ID_ONLY_CONFIG: DggrsApiConfig = DggrsApiConfig {
    region: false,
    children: false,
    center: false,
    neighbors: false,
    densify: false,
    area_sqm: false,
    vertex_count: false,
};
pub(crate) const CENTER_CONFIG: DggrsApiConfig = DggrsApiConfig {
    region: false,
    children: false,
    center: true,
    neighbors: false,
    densify: false,
    area_sqm: false,
    vertex_count: false,
};
