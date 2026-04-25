mod common;
pub mod error;
pub mod geotiff_convert;
pub mod models;
pub mod query;
pub mod storage;
pub mod value;
pub mod zarr;

pub use geoplegma::api::DggrsApi;
pub use geoplegma::types::{RefinementLevel, RelativeDepth, Zone, ZoneId, Zones};

pub use geotiff_convert::convert_geotiff_file_to_backend;
pub use models::{AttributeSchema, Compression, DataType, DatasetMetadata};
pub use query::{
    H3VisualizationCell, export_h3_level_as_visualization_json,
    query_value_by_cell_index, query_value_for_point, write_h3_level_as_visualization_json,
};
pub use storage::StorageBackend;
pub use value::{decode_value_to_json, format_value};
pub use zarr::ZarrBackend;
