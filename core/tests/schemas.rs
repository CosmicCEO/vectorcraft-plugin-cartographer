use cartographer_core::schema::{joint_schema, tracks_schema, yard_schema};
use cartographer_core::tracks::TrackSpec;
use cartographer_core::yard::YardSpec;
use serde_json::{Value, json};

fn field<'a>(schema: &'a Value, key: &str) -> &'a Value {
    schema["fields"].as_array().unwrap().iter().find(|f| f["key"] == key).unwrap_or_else(|| panic!("no field {key}"))
}

fn keys(schema: &Value) -> Vec<&str> {
    schema["fields"].as_array().unwrap().iter().map(|f| f["key"].as_str().unwrap()).collect()
}

#[test]
fn the_tracks_schema_lists_four_fields_with_the_limits() {
    let s = tracks_schema();
    assert_eq!(s["kind"], "tracks");
    assert_eq!(keys(&s), vec!["tracks", "spacing", "ballast", "ties"]);
    let t = field(&s, "tracks");
    assert_eq!((t["type"].as_str(), t["min"].as_i64(), t["max"].as_i64()), (Some("integer"), Some(1), Some(12)));
    assert_eq!(field(&s, "spacing")["type"], "number");
    for k in ["ballast", "ties"] {
        assert_eq!(field(&s, k)["options"], json!(["on", "off"]), "{k}");
    }
}

#[test]
fn the_yard_schema_lists_its_fields_and_leaves_crossovers_to_words() {
    let s = yard_schema();
    assert_eq!(s["kind"], "yard");
    assert_eq!(keys(&s), vec!["mains", "left", "right", "ends", "frog", "spacing", "at", "length", "ballast", "ties"]);
    assert_eq!((field(&s, "mains")["min"].as_i64(), field(&s, "mains")["max"].as_i64()), (Some(1), Some(4)));
    assert_eq!((field(&s, "left")["min"].as_i64(), field(&s, "left")["max"].as_i64()), (Some(0), Some(16)));
    assert_eq!((field(&s, "frog")["min"].as_i64(), field(&s, "frog")["max"].as_i64()), (Some(4), Some(20)));
    assert_eq!(field(&s, "ends")["options"], json!(["merge", "stub"]));
    assert!(!keys(&s).contains(&"crossovers"));
}

#[test]
fn the_joint_schema_lists_kind_main_and_frog() {
    let s = joint_schema();
    assert_eq!(s["kind"], "joint");
    assert_eq!(keys(&s), vec!["kind", "main", "frog"]);
    assert_eq!(field(&s, "kind")["options"], json!(["crossing", "branch", "join"]));
    assert_eq!((field(&s, "frog")["min"].as_i64(), field(&s, "frog")["max"].as_i64()), (Some(4), Some(20)));
}

#[test]
fn every_default_is_what_the_parser_defaults_to() {
    let t = TrackSpec::default().to_json();
    let s = tracks_schema();
    assert_eq!(field(&s, "tracks")["default"], t["tracks"]);
    assert_eq!(field(&s, "spacing")["default"], cartographer_core::rail_units::model_to_record(&t)["spacing"], "the card is in metres");
    let y = yard_schema();
    let d = YardSpec::default();
    assert_eq!(field(&y, "mains")["default"], d.mains);
    assert_eq!(field(&y, "left")["default"], d.left);
    assert_eq!(field(&y, "frog")["default"], d.frog);
    assert_eq!(field(&y, "ends")["default"], if d.merge { "merge" } else { "stub" });
}

#[test]
fn every_field_has_a_label_a_type_and_help() {
    for s in [tracks_schema(), yard_schema(), joint_schema()] {
        for f in s["fields"].as_array().unwrap() {
            for k in ["key", "label", "type", "help"] {
                assert!(f[k].as_str().is_some_and(|t| !t.is_empty()), "{} {k}: {f}", s["kind"]);
            }
        }
    }
}
