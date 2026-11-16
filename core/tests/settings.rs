use cartographer_core::builder::{set_settings_text, settings_summary};
use cartographer_core::schema::settings_schema;
use cartographer_core::storage::{Settings, Style};
use cartographer_core::units::TargetScale;
use serde_json::{Value, json};

fn input() -> Value {
    let line = |id: i64| json!({"id": id, "name": format!("line{id}"), "path": {"subpaths": [{"anchors": [{"p": [0, 0]}, {"p": [10, 10]}], "closed": false}]}});
    json!({"objects": [line(1), line(2)]})
}

fn params(description: &str, data: Value) -> Value {
    json!({"description": description, "_context": {"command": "set_settings", "documentData": data}})
}

fn run(description: &str, data: Value) -> Result<Value, String> {
    set_settings_text(&input(), &params(description, data)).map(|t| serde_json::from_str(&t).unwrap())
}

#[test]
fn set_settings_keeps_objects_and_stores_data() {
    let out = run(r#"{"style":"hand-drawn","target_print_scale":"1:5000","seed":7,"north_deg":10}"#, json!({})).unwrap();
    let ids: Vec<i64> = out["objects"].as_array().unwrap().iter().map(|o| o["id"].as_i64().unwrap()).collect();
    assert_eq!(ids, vec![1, 2]);
    assert_eq!(out["documentData"]["schema"], 1);
    assert_eq!(out["documentData"]["settings"]["style"], "hand-drawn");
    assert_eq!(out["documentData"]["settings"]["target_print_scale"], "1:5000");
    assert_eq!(out["documentData"]["settings"]["seed"], 7);
}

#[test]
fn a_partial_change_keeps_the_other_settings() {
    let out = run(r#"{"seed":3}"#, json!({"schema": 1, "settings": {"style": "high-contrast", "north_deg": 45}})).unwrap();
    assert_eq!(out["documentData"]["settings"]["style"], "high-contrast");
    assert_eq!(out["documentData"]["settings"]["north_deg"], 45.0);
    assert_eq!(out["documentData"]["settings"]["seed"], 3);
}

#[test]
fn unknown_data_is_kept() {
    let out = run(r#"{}"#, json!({"roads": {"a": 1}})).unwrap();
    assert_eq!(out["documentData"]["roads"], json!({"a": 1}));
}

#[test]
fn a_newer_schema_declines() {
    let e = run(r#"{"seed":2}"#, json!({"schema": 9})).unwrap_err();
    assert!(e.contains("newer Cartographer"), "{e}");
}

#[test]
fn bad_description_is_plain() {
    assert!(run("{nope", json!({})).unwrap_err().contains("not valid JSON"));
    let e = run(r#"{"style":"neon"}"#, json!({})).unwrap_err();
    assert!(e.contains("black-white") && e.contains("medium-contrast"), "{e}");
    assert!(run(r#"{"target_print_scale":"1:0"}"#, json!({})).unwrap_err().contains("1:N"));
}

#[test]
fn an_empty_description_changes_nothing() {
    let out = run("", json!({})).unwrap();
    assert_eq!(out["documentData"]["settings"]["style"], "medium-contrast");
}

#[test]
fn summary_text() {
    let s = Settings { target_print_scale: TargetScale::Ratio(5000), style: Style::HandDrawn, seed: 7, north_deg: 10.0 };
    assert_eq!(settings_summary(&s), "style hand-drawn, target print scale 1:5000, seed 7, north 10 degrees");
}

#[test]
fn schema_lists_four_fields_with_four_styles() {
    let schema = settings_schema();
    let fields = schema["fields"].as_array().unwrap();
    assert_eq!(fields.len(), 4);
    let style = fields.iter().find(|f| f["key"] == "style").unwrap();
    let options: Vec<&str> = style["options"].as_array().unwrap().iter().map(|o| o.as_str().unwrap()).collect();
    assert_eq!(options, vec!["black-white", "hand-drawn", "high-contrast", "medium-contrast"]);
    assert_eq!(schema["kind"], "settings");
}

#[test]
fn a_misspelt_key_is_refused_and_names_the_known_ones() {
    let e = run(r#"{"stlye":"hand-drawn"}"#, json!({})).unwrap_err();
    assert!(e.contains("stlye") && e.contains("style") && e.contains("seed"), "{e}");
    assert!(Settings::check_keys(&json!({"stlye": 1})).unwrap_err().contains("stlye"));
    assert!(Settings::check_keys(&json!({"style": "x", "seed": 1})).is_ok());
}

#[test]
fn null_keeps_the_stored_value() {
    let out = run(r#"{"style":null,"seed":5}"#, json!({"schema": 1, "settings": {"style": "high-contrast"}})).unwrap();
    assert_eq!(out["documentData"]["settings"]["style"], "high-contrast");
    assert_eq!(out["documentData"]["settings"]["seed"], 5);
}

#[test]
fn a_scale_that_is_not_text_says_so() {
    let e = Settings::from_json(&json!({"target_print_scale": true})).unwrap_err();
    assert!(e.contains("text"), "{e}");
}

#[test]
fn unknown_settings_keys_survive_a_command() {
    let out = run(r#"{"seed":2}"#, json!({"schema": 1, "settings": {"sheet": {"size": "A0"}}})).unwrap();
    assert_eq!(out["documentData"]["settings"]["sheet"], json!({"size": "A0"}));
}
