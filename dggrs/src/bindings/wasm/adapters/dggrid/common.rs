use std::{
    io::{BufReader, Lines},
    path::{Path, PathBuf},
};

use js_sys::Array;
use serde_wasm_bindgen::to_value;
use tracing::span::Id;
use wasm_bindgen::prelude::*;

use crate::{
    adapters::dggrid::common::{
        assign_field, bbox_to_aigen, dggrid_cleanup, dggrid_execute, dggrid_metafile, dggrid_parse, dggrid_setup, parse_aigen, parse_children, parse_neighbors, print_file, read_file, read_lines
    },
    bindings::wasm::models::common::JsZones,
    wasm_fields_clone,
};

#[wasm_bindgen]
#[derive(Clone)]
pub struct IdArray {
    id: Option<String>,
    arr: Option<Vec<String>>,
}
#[wasm_bindgen]
impl IdArray {}
// this is strictly for wasm, since the String type in Rust isn't implicitly copyable
wasm_fields_clone!(IdArray,
    (get_id, set_id,  id, "id", Option<String>),
    (get_arr, set_arr, arr, "arr", Option<Vec<String>>),
);

// #[wasm_bindgen]
// pub struct IdArrays {
//     ids: Vec<Option<String>>,
//     vec_array: Vec<Option<Vec<String>>>,
// }

// wasm_fields_clone!(IdArrays,
//     (get_id_arrays, set_id_arrays,  id_arrays, "id_arrays", Vec<IdArray>),
// );

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
) -> Result<(), JsValue> {
    let path_m = PathBuf::from(metafile);
    let path_c = PathBuf::from(cell_output_file_name);
    let path_ch = PathBuf::from(children_output_file_name);
    let path_n = PathBuf::from(neighbor_output_file_name);

    dggrid_metafile(&path_m, &depth, &path_c, &path_ch, &path_n, densify)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn dggrid_execute_wasm(dggrid_path: String, meta_path: String) {
    let dggrid_path = PathBuf::from(dggrid_path);
    let meta_path = PathBuf::from(meta_path);
    dggrid_execute(&dggrid_path, &meta_path);
}

#[wasm_bindgen]
pub fn dggrid_parse_wasm(
    aigen_path: String,
    children_path: String,
    neighbor_path: String,
    depth: u8,
) -> Result<JsZones, JsValue> {
    let aigen_path = PathBuf::from(aigen_path);
    let children_path = PathBuf::from(children_path);
    let neighbor_path = PathBuf::from(neighbor_path);
    let zones = dggrid_parse(&aigen_path, &children_path, &neighbor_path, &depth)
        .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

    let export = zones.to_export();
    Ok(export)
}

#[wasm_bindgen]
pub fn parse_aigen_wasm(data: String, depth: u8) -> Result<JsZones, JsValue> {
    let zones = parse_aigen(&data, &depth).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
    let export = zones.to_export();
    Ok(export)
}

#[wasm_bindgen]
pub fn dggrid_cleanup_wasm(
    meta_path: String,
    aigen_path: String,
    children_path: String,
    neighbor_path: String,
    bbox_path: String,
) {
    dggrid_cleanup(
        &PathBuf::from(meta_path),
        &PathBuf::from(aigen_path),
        &PathBuf::from(children_path),
        &PathBuf::from(neighbor_path),
        &PathBuf::from(bbox_path),
    );
}

// @TODO review
#[wasm_bindgen]
pub fn parse_children_wasm(data: String, depth: u8) -> Result<Array, JsValue> {
    let vec = parse_children(&data, &depth).map_err(|e| JsValue::from_str(&format!("{}", e)))?;

    let mut js_array = Array::new();
    for item in &vec {
        js_array.push(&JsValue::from(item.id.clone()));
        // Convert Rust IdArray to JsValue
        js_array.push(&JsValue::from(item.arr.clone()));
    }
    Ok(js_array)
}

#[wasm_bindgen]
pub fn parse_neighbors_wasm(data: String, depth: u8) -> Result<Array, JsValue> {
    let vec = parse_neighbors(&data, &depth).map_err(|e| JsValue::from_str(&format!("{}", e)))?;

    let mut js_array = Array::new();
    for item in &vec {
        js_array.push(&JsValue::from(item.id.clone()));
        // Convert Rust IdArray to JsValue
        js_array.push(&JsValue::from(item.arr.clone()));
    }
    Ok(js_array)
}

// #[wasm_bindgen]
// pub fn assign_field_wasm(data: String, depth: u8) {
//     assign_field(zones, data, field);
// }

#[wasm_bindgen]
pub fn print_file_wasm(file: String) {
    print_file(PathBuf::from(file));
}

#[wasm_bindgen]
pub fn read_file_wasm(file: String) -> Result<String, JsValue> {
    let path = Path::new(&file);
    read_file(&path).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn read_lines_wasm(filename: String) -> Result<Array, JsValue> {
    let path = Path::new(&filename);
    let lines = read_lines(path).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let js_array = Array::new();
    for line_result in lines {
        match line_result {
            Ok(line) => {
                let _ = js_array.push(&JsValue::from_str(&line));
            }
            Err(e) => return Err(JsValue::from_str(&e.to_string())),
        }
    }

    Ok(js_array)
}


// #[wasm_bindgen]
// pub fn bbox_to_aigen_wasm(bbox: Array, bboxfile: String) -> Result<()> {

//     bbox_to_aigen(bbox, &PathBuf::from(bboxfile))

// }