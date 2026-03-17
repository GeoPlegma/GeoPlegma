use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    /// OGC DGGS Reference System identifier (e.g. `"H3"`, `"ISEA3H"`).
    pub dggrs: String, // (switch to "name")

    /// Spatial extent covered by this dataset.
    pub extent: GridExtent,

    /// Schema of the stored attributes.
    pub attributes: Vec<AttributeSchema>,

    /// Chunk size: number of cells per chunk along the linearized SFC axis.
    // pub chunk_size: u64,

    /// Resolution levels stored in this dataset.
    pub levels: Vec<u32>,

    /// Compression method used for the cell attribute data.
    pub compression: String,
}

/// The spatial bounds or specific cell subset covered by a dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GridExtent {
    /// The dataset covers the entire globe.
    Global,

    /// The dataset covers a geographic bounding box.
    BoundingBox {
        min_lon: f64,
        min_lat: f64,
        max_lon: f64,
        max_lat: f64,
    },
}

/// Description of a single data attribute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeSchema {
    /// Human-readable name (e.g. `"elevation"`, `"population"`).
    pub name: String,

    /// Data type of the attribute values.
    pub dtype: DataType,

    /// Optional fill / no-data value.
    pub fill_value: Option<String>,
}

/// Supported data types for cell attribute values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Float32,
    Float64,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
}

impl DataType {
    pub const fn byte_size(&self) -> usize {
        match self {
            DataType::Int8 | DataType::UInt8 => 1,
            DataType::Int16 | DataType::UInt16 => 2,
            DataType::Float32 | DataType::Int32 | DataType::UInt32 => 4,
            DataType::Float64 | DataType::Int64 | DataType::UInt64 => 8,
        }
    }
}
