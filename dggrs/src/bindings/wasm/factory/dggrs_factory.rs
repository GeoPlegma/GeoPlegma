use std::sync::Arc;

use wasm_bindgen::prelude::*;

use crate::{get, ports::dggrs::DggrsPort};

/// wasm-visible wrapper that owns the Arc<dyn DggrsPort>
#[wasm_bindgen]
pub struct JsDggrs {
    inner: Arc<dyn DggrsPort>,
}

#[wasm_bindgen]
pub fn get_wasm(tool: &str, dggrs: &str) -> Result<JsDggrs, JsValue> {
    match get(tool, dggrs) {
        Ok(handle) => Ok(JsDggrs { inner: handle }),
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}