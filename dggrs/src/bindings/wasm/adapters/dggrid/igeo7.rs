use crate::{
    adapters::dggrid::igeo7::{
        extract_res_from_cellid, extract_res_from_z3, extract_res_from_z7, igeo7_metafile,
    },
    bindings::wasm::adapters::dggrid::dggrid::DggridAdapter,
};
use std::path::PathBuf;
use wasm_bindgen::prelude::*;
pub const CLIP_CELL_DENSIFICATION: u8 = 50; // DGGRID option

#[wasm_bindgen]
pub struct Igeo7ImplWasm {
    adapter: DggridAdapter,
}

#[wasm_bindgen]
impl Igeo7ImplWasm {
    // Optional: allow custom paths too
    pub fn new(executable: String, workdir: String) -> Self {
        Self {
            adapter: DggridAdapter::new(executable, workdir),
        }
    }
}

#[wasm_bindgen]
pub fn igeo7_metafile_wasm(meta_path: String) -> Result<(), JsValue> {
    igeo7_metafile(&PathBuf::from(meta_path))
        .map_err(|e| JsValue::from_str(&format!("Metafile error: {e}")));
    Ok(())
}
