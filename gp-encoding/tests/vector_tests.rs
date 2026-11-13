use geoplegma::types::{DggrsUid, RefinementLevel};
use gp_encoding::vector::{decode, encode};
use serde_json::json;
use std::fs;

#[test]
fn test_decode() {
    let val = json!({
        "dggrs": "H3",
        "type": "Feature",
        "geometry": {
            "type": "Point",
            "coordinates": "871e80531ffffff"
        },
        "properties": {
            "name": "Colosseum"
        }
    });

    let val = decode(val).unwrap();

    let coordinates = val["geometry"]["coordinates"].as_array().unwrap();
    assert_eq!(coordinates.len(), 2);
    assert!(coordinates.iter().all(|coordinate| coordinate.is_number()));
}

#[test]
fn test_encode_end_to_end() {
    let geojson_content = json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": [12.4924, 41.8902]
                },
                "properties": {
                    "name": "Colosseum",
                    "ranking": 1
                }
            }
        ]
    });

    let temp_dir = tempfile::tempdir().unwrap();
    let input_path = temp_dir.path().join("input.geojson");
    let output_path = temp_dir.path().join("output.json");

    fs::write(&input_path, serde_json::to_string(&geojson_content).unwrap()).unwrap();

    encode(
        &input_path,
        &output_path,
        DggrsUid::H3,
        RefinementLevel::from(7u8),
    )
    .unwrap();

    assert!(output_path.exists());

    let encoded_str = fs::read_to_string(&output_path).unwrap();
    let encoded_json: serde_json::Value = serde_json::from_str(&encoded_str).unwrap();

    assert_eq!(encoded_json["dggrs"], "H3");
    assert_eq!(encoded_json["refinement_level"], 7);
    assert_eq!(encoded_json["type"], "FeatureCollection");
    
    let features = encoded_json["features"].as_array().unwrap();
    assert_eq!(features.len(), 1);
    
    let feat = &features[0];
    assert_eq!(feat["properties"]["name"], "Colosseum");
    assert_eq!(feat["properties"]["ranking"], 1);
    
    let cell_id = feat["geometry"]["coordinates"].as_str().unwrap();
    assert!(!cell_id.is_empty());
}
