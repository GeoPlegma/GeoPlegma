use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use zarrs::array::{Array, ArrayBuilder, ArrayBytes};
use zarrs::array::codec::api::BytesToBytesCodecTraits;
use zarrs::array::codec::{GzipCodec, ZstdCodec};
use zarrs::group::{Group, GroupBuilder};
use zarrs_filesystem::FilesystemStore;

use crate::error::EncodingError;
use crate::models::{Compression, DataType, DatasetMetadata};
use crate::storage::{LevelHandle, StorageBackend};

/// Handle to a Zarr array that represents a single resolution level.
pub struct ZarrLevel {
    /// Resolution level index.
    pub level: u32,
    /// Number of cells at this level.
    pub num_cells: u64,
}

impl LevelHandle for ZarrLevel {}

// ─────────────────────────── Zarr Backend ───────────────────────────

/// A [`StorageBackend`] implementation that persists DGGS data in the
/// [Zarr v3](https://zarr-specs.readthedocs.io/en/latest/v3/core/v3.0.html)
/// format.
pub struct ZarrBackend {
    /// Root path of the Zarr store on disk.
    _root: PathBuf,
    /// The underlying filesystem store.
    store: Arc<FilesystemStore>,
    /// Dataset metadata.
    metadata: DatasetMetadata,
    /// Created resolution levels (level → num_cells).
    level_map: BTreeMap<u32, u64>,
}

impl ZarrBackend {
    fn codecs_for_compression(
        compression: Option<&Compression>,
    ) -> Result<Option<Vec<Arc<dyn BytesToBytesCodecTraits>>>, EncodingError> {
        let Some(config) = compression else {
            return Ok(None);
        };

        match config {
            Compression::Gzip => {
                let codec: Arc<dyn BytesToBytesCodecTraits> = Arc::new(
                    GzipCodec::new(6).map_err(|e| EncodingError::Zarr(e.to_string()))?,
                );
                Ok(Some(vec![codec]))
            }
            Compression::Zstd => {
                let codec: Arc<dyn BytesToBytesCodecTraits> = Arc::new(ZstdCodec::new(3, false));
                Ok(Some(vec![codec]))
            }
        }
    }

    /// Convert our [`DataType`] to a zarrs data type string.
    fn to_zarr_dtype_str(dt: &DataType) -> &'static str {
        match dt {
            DataType::Float32 => "float32",
            DataType::Float64 => "float64",
            DataType::Int8 => "int8",
            DataType::Int16 => "int16",
            DataType::Int32 => "int32",
            DataType::Int64 => "int64",
            DataType::UInt8 => "uint8",
            DataType::UInt16 => "uint16",
            DataType::UInt32 => "uint32",
            DataType::UInt64 => "uint64",
        }
    }

    /// Determine the primary Zarr data type string from the attribute schema.
    ///
    /// For now we use the first attribute; multi-attribute datasets can be
    /// extended to use structured types or separate arrays.
    fn primary_dtype_str(&self) -> &'static str {
        self.metadata
            .attributes
            .first()
            .map(|a| Self::to_zarr_dtype_str(&a.dtype))
            .unwrap_or("uint64")
    }

    /// Build the Zarr array path for a level (e.g. `"/level_3"`).
    fn level_path(level: u32, band: u32) -> String {
        format!("/level_{level}/band_{band}")
    }

    /// Open a Zarr array for a given resolution level.
    fn open_array(&self, level: u32, band: u32) -> Result<Array<FilesystemStore>, EncodingError> {
        let path = Self::level_path(level, band);
        let array = Array::open(self.store.clone(), &path)
            .map_err(|e| EncodingError::Zarr(e.to_string()))?;
        Ok(array)
    }

    /// Persist the [`DatasetMetadata`] as JSON attributes on the root group.
    fn write_metadata(
        store: &Arc<FilesystemStore>,
        metadata: &DatasetMetadata,
    ) -> Result<(), EncodingError> {
        let attrs_json = serde_json::to_value(metadata)?;
        let mut attributes = serde_json::Map::new();
        attributes.insert("gp_encoding".to_string(), attrs_json);

        let group = GroupBuilder::new()
            .attributes(attributes)
            .build(store.clone(), "/")
            .map_err(|e| EncodingError::Zarr(e.to_string()))?;

        group
            .store_metadata()
            .map_err(|e| EncodingError::Zarr(e.to_string()))?;

        Ok(())
    }

    /// Read [`DatasetMetadata`] from the root group attributes.
    fn read_metadata(store: &Arc<FilesystemStore>) -> Result<DatasetMetadata, EncodingError> {
        let group =
            Group::open(store.clone(), "/").map_err(|e| EncodingError::Zarr(e.to_string()))?;

        let attrs = group.attributes();

        let meta_value = attrs.get("gp_encoding").ok_or_else(|| {
            EncodingError::Zarr("missing 'gp_encoding' attribute in root group".into())
        })?;

        let metadata: DatasetMetadata = serde_json::from_value(meta_value.clone())?;
        Ok(metadata)
    }
}

impl StorageBackend for ZarrBackend {
    type Level = ZarrLevel;

    fn create(path: &Path, metadata: DatasetMetadata) -> Result<Self, EncodingError> {
        // Ensure the directory exists.
        std::fs::create_dir_all(path)?;

        let store =
            Arc::new(FilesystemStore::new(path).map_err(|e| EncodingError::Zarr(e.to_string()))?);

        // Write root group + metadata.
        Self::write_metadata(&store, &metadata)?;

        Ok(Self {
            _root: path.to_path_buf(),
            store,
            metadata,
            level_map: BTreeMap::new(),
        })
    }

    fn open(path: &Path) -> Result<Self, EncodingError> {
        let store =
            Arc::new(FilesystemStore::new(path).map_err(|e| EncodingError::Zarr(e.to_string()))?);

        let metadata = Self::read_metadata(&store)?;

        // Discover existing level arrays.
        let mut level_map = BTreeMap::new();
        for &lvl in &metadata.levels {
            for band in 0..metadata.attributes.len() as u32 {
                let array_path = Self::level_path(lvl, band);
                if let Ok(array) = Array::open(store.clone(), &array_path) {
                    let shape = array.shape();
                    let num_cells = shape.first().copied().unwrap_or(0);
                    level_map.insert(lvl, num_cells);
                }
            }
        }

        Ok(Self {
            _root: path.to_path_buf(),
            store,
            metadata,
            level_map,
        })
    }

    fn metadata(&self) -> &DatasetMetadata {
        &self.metadata
    }

    fn create_level(
        &mut self,
        level: u32,
        band: u32,
        num_cells: u64,
    ) -> Result<ZarrLevel, EncodingError> {
        let path = Self::level_path(level, band);
        let dtype_str = self.primary_dtype_str();
        let chunk_size = self.metadata.chunk_size;

        let mut array_builder = ArrayBuilder::new(
            vec![num_cells],  // array shape
            vec![chunk_size], // regular chunk shape
            dtype_str,        // data type as string
            0u64,             // fill value
        );

        if let Some(codecs) = Self::codecs_for_compression(self.metadata.compression.as_ref())? {
            array_builder.bytes_to_bytes_codecs(codecs);
        }

        let array = array_builder
            .build(self.store.clone(), &path)
            .map_err(|e| EncodingError::Zarr(e.to_string()))?;

        array
            .store_metadata()
            .map_err(|e| EncodingError::Zarr(e.to_string()))?;

        self.level_map.insert(level, num_cells);

        Ok(ZarrLevel { level, num_cells })
    }

    fn levels(&self) -> Vec<u32> {
        self.level_map.keys().copied().collect()
    }

    fn band_count(&self) -> u32 {
        self.metadata.attributes.len() as u32
    }

    fn write_chunk(
        &self,
        level: u32,
        band: u32,
        chunk_index: u64,
        data: &[u8],
    ) -> Result<(), EncodingError> {
        let array = self.open_array(level, band)?;

        let array_bytes = ArrayBytes::new_flen(data.to_vec());
        array
            .store_chunk(&[chunk_index], array_bytes)
            .map_err(|e| EncodingError::Zarr(e.to_string()))?;

        Ok(())
    }

    fn read_chunk(
        &self,
        level: u32,
        band: u32,
        chunk_index: u64,
    ) -> Result<Vec<u8>, EncodingError> {
        let array = self.open_array(level, band)?;

        let bytes: ArrayBytes<'static> = array
            .retrieve_chunk(&[chunk_index])
            .map_err(|e| EncodingError::Zarr(e.to_string()))?;

        let fixed = bytes
            .into_fixed()
            .map_err(|e| EncodingError::Zarr(e.to_string()))?;

        Ok(fixed.into_owned())
    }

    fn num_chunks(&self, level: u32) -> Result<u64, EncodingError> {
        let num_cells = *self
            .level_map
            .get(&level)
            .ok_or_else(|| EncodingError::Zarr(format!("level {level} not found")))?;
        let chunk_size = self.metadata.chunk_size;
        Ok(num_cells.div_ceil(chunk_size))
    }
}
