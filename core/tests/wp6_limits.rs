use cartographer_core::builder::build_text;
use serde_json::{Value, json};

fn path(id: i64, name: &str, pts: &[[f64; 2]], closed: bool) -> Value {
    json!({"id": id, "name": name, "path": {"subpaths": [{"anchors": pts.iter().map(|p| json!({"p": p})).collect::<Vec<_>>(), "closed": closed}]}})
}

fn names(text: &str) -> Vec<String> {
    serde_json::from_str::<Value>(text).unwrap()["objects"].as_array().unwrap().iter().filter_map(|o| o["name"].as_str().map(String::from)).collect()
}

#[test]
fn the_point_note_starts_near_the_measured_limit() {
    let pts: Vec<[f64; 2]> = (0..3000).map(|i| [f64::from(i) * 0.5, 300.0]).collect();
    let input = json!({"objects": [path(1, "road.a", &pts, false)]});
    let p = json!({"_context": {"documentData": {"schema": 1, "segments": {"a": {"cross_section": "lane back | lane fwd"}}}, "artboard": [0, 0, 800, 800]}});
    let text = build_text(&input, &p).unwrap();
    let notes = serde_json::from_str::<Value>(&text).unwrap()["documentData"]["notes"].clone();
    assert!(!notes.to_string().contains("points"), "3,000 points is far from the limit (about 19,000 measured): {notes}");
}
