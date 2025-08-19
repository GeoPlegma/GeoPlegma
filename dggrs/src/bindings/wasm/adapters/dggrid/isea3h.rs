use crate::{
    adapters::dggrid::isea3h::{
        extract_res_from_cellid, extract_res_from_z3, extract_res_from_z7,
    },
    bindings::wasm::adapters::dggrid::dggrid::DggridAdapter,
};
use std::path::PathBuf;
use wasm_bindgen::prelude::*;
pub const CLIP_CELL_DENSIFICATION: u8 = 50; // DGGRID option

#[wasm_bindgen]
pub struct Isea3hImplWasm {
    adapter: DggridAdapter,
}

#[wasm_bindgen]
impl Isea3hImplWasm {
    // Optional: allow custom paths too
    pub fn new(executable: String, workdir: String) -> Self {
        Self {
            adapter: DggridAdapter::new(executable, workdir),
        }
    }
}

// impl Default for Isea3hImpl {
//     fn default() -> Self {
//         Self {
//             adapter: DggridAdapter::default(),
//         }
//     }
// }

// #[wasm_bindgen]
// pub fn isea3h_metafile_wasm(meta_path: String) -> Result<(), JsValue> {
//     isea3h_metafile(&PathBuf::from(meta_path))
//         .map_err(|e| JsValue::from_str(&format!("Metafile error: {e}")));
//     Ok(())
// }
