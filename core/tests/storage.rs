use cartographer_core::storage::{Doc, SCHEMA, Settings, Style};
use cartographer_core::units::TargetScale;
use serde_json::{Value, json};

#[test]
fn empty_data_gets_defaults_and_schema_1() {
    for input in [json!({}), Value::Null] {
        let doc = Doc::from_document_data(&input).unwrap();
        let v = doc.to_value();
        assert_eq!(v["schema"], 1);
        assert_eq!(v["settings"]["style"], "medium-contrast");
        assert_eq!(v["settings"]["target_print_scale"], "true");
        assert_eq!(v["settings"]["seed"], 1);
        assert_eq!(v["settings"]["north_deg"], 0.0);
        assert_eq!(doc.settings.style, Style::MediumContrast);
        assert_eq!(doc.settings.target_print_scale, TargetScale::True);
    }
    assert_eq!(SCHEMA, 1);
}

#[test]
fn unknown_keys_survive_a_round_trip() {
    let doc = Doc::from_document_data(&json!({"schema": 1, "roads": {"a": 1}, "settings": {"seed": 9}})).unwrap();
    let v = doc.to_value();
    assert_eq!(v["roads"], json!({"a": 1}));
    assert_eq!(v["settings"]["seed"], 9);
    assert_eq!(v["settings"]["style"], "medium-contrast");
}

#[test]
fn a_newer_schema_is_refused_not_overwritten() {
    let e = Doc::from_document_data(&json!({"schema": 2, "roads": {}})).err().unwrap();
    assert!(e.contains("newer Cartographer") && e.contains("schema 2"), "{e}");
}

#[test]
fn schema_zero_or_missing_is_migrated() {
    for input in [json!({"roads": {"a": 1}}), json!({"schema": 0, "roads": {"a": 1}})] {
        let v = Doc::from_document_data(&input).unwrap().to_value();
        assert_eq!(v["schema"], 1);
        assert_eq!(v["roads"], json!({"a": 1}));
    }
}

#[test]
fn settings_are_read_and_written() {
    let s = Settings::from_json(&json!({"target_print_scale": "1:5000", "style": "hand-drawn", "seed": 7, "north_deg": 10})).unwrap();
    assert_eq!((s.target_print_scale, s.style, s.seed, s.north_deg), (TargetScale::Ratio(5000), Style::HandDrawn, 7, 10.0));
    let back = s.to_json();
    assert_eq!(back["style"], "hand-drawn");
    assert_eq!(back["target_print_scale"], "1:5000");
    for id in ["black-white", "hand-drawn", "high-contrast", "medium-contrast"] {
        assert_eq!(Style::parse(id).unwrap().id(), id);
    }
}

#[test]
fn bad_settings_are_plain_errors() {
    let e = Settings::from_json(&json!({"style": "neon"})).unwrap_err();
    for id in ["black-white", "hand-drawn", "high-contrast", "medium-contrast"] {
        assert!(e.contains(id), "{e}");
    }
    let e = Settings::from_json(&json!({"north_deg": 400})).unwrap_err();
    assert!(e.contains("-360") && e.contains("360"), "{e}");
    assert!(Settings::from_json(&json!({"seed": -1})).unwrap_err().contains("seed"));
    assert!(Settings::from_json(&json!({"seed": 4294967296u64})).unwrap_err().contains("seed"));
    assert!(Settings::from_json(&json!({"target_print_scale": "1:0"})).unwrap_err().contains("1:N"));
    assert!(Doc::from_document_data(&json!({"settings": 5})).is_err());
}
