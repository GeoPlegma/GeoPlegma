use std::{path::PathBuf};

use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use crate::{adapters::dggrid::common::{dggrid_metafile, dggrid_setup}, wasm_fields_clone};

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct IdArray {
    id: Option<String>,
    arr: Option<Vec<String>>,
}

// this is strictly for wasm, since the String type in Rust isn't implicitly copyable
wasm_fields_clone!(IdArray,
    (get_id, set_id,  id, "id", Option<String>),
    (get_arr, set_arr, arr, "arr", Option<Vec<String>>),
);

#[wasm_bindgen]
pub fn dggrid_setup_wasm(workdir: String) -> Result<JsValue, JsValue> {
    let path = PathBuf::from(workdir);
    let output = dggrid_setup(&path);
    to_value(&output).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn dggrid_metafile_wasm(
    metafile: String,
    depth: u8,
    cell_output_file_name: String,
    children_output_file_name: String,
    neighbor_output_file_name: String,
    densify: bool,
) ->  Result<(), JsValue> {
    let path_m = PathBuf::from(metafile);
    let path_c = PathBuf::from(cell_output_file_name);
    let path_ch = PathBuf::from(children_output_file_name);
    let path_n = PathBuf::from(neighbor_output_file_name);

    dggrid_metafile(
        &path_m,
        &depth,
        &path_c,
        &path_ch,
        &path_n,
        densify,
    ).map_err(|e| JsValue::from_str(&e.to_string()))
}
