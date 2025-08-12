use std::path::PathBuf;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct DggridAdapter {
    executable: String,
    workdir: String,
}

#[wasm_bindgen]
impl DggridAdapter {
    pub fn new(executable: String, workdir: String) -> Self {
        Self {
            executable,
            workdir,
        }
    }
}

impl Default for DggridAdapter {
    fn default() -> Self {
        Self {
            executable: "dggrid".to_owned(),
            workdir: "/dev/shm".to_owned(),
        }
    }
}
