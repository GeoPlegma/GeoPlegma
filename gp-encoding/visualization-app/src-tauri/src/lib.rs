#[tauri::command]
fn get_h3_data(store: String, level: u32) -> Result<Vec<gp_encoding::query::H3VisualizationCell>, String> {
    use gp_encoding::{ZarrBackend, StorageBackend};
    let backend = ZarrBackend::open(std::path::Path::new(&store)).map_err(|e| e.to_string())?;
    gp_encoding::query::export_h3_level_as_visualization_json(&backend, level).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_h3_levels(store: String) -> Result<Vec<u32>, String> {
    use gp_encoding::{ZarrBackend, StorageBackend};
    let backend = ZarrBackend::open(std::path::Path::new(&store)).map_err(|e| e.to_string())?;
    Ok(backend.levels())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_h3_data, get_h3_levels])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
