use serde_json::Value;
use cartographer_core::rail_standards::*;

fn db() -> Value {
    serde_json::from_str(include_str!("../../library/standards-rail.json")).unwrap()
}

fn entry<'a>(d: &'a Value, id: &str) -> &'a Value {
    d["entries"].as_array().unwrap().iter().find(|e| e["id"] == id).unwrap_or_else(|| panic!("no entry {id}"))
}

fn val(e: &Value, name: &str) -> f64 {
    e["values"].as_array().unwrap().iter().find(|v| v["name"] == name).unwrap()["value"].as_f64().unwrap()
}

#[test]
fn constants_match_the_database() {
    let d = db();
    assert_eq!(SCALE, entry(&d, "scale")["values"][0]["value"].as_f64().unwrap());
    assert_eq!(GAUGE, val(entry(&d, "gauge"), "gauge_mm"));
    assert_eq!(RAIL_HEAD, val(entry(&d, "rail"), "head_mm"));
    let tie = entry(&d, "tie");
    assert_eq!((TIE_LENGTH, TIE_WIDTH, TIE_PITCH), (val(tie, "length_mm"), val(tie, "width_mm"), val(tie, "pitch_mm")));
    let smallest = entry(&d, "radius-frog")["values"][0]["radius_mm"].as_f64().unwrap();
    assert_eq!(MIN_RADIUS, smallest);
}

#[test]
fn thirteen_feet_is_24_77_mm() {
    assert!((TRACK_CENTERS - 24.765).abs() < 1e-9);
    assert_eq!(val(entry(&db(), "tangent-centers"), "classic_ft"), 13.0);
}

#[test]
fn verified_entries_cite_their_source() {
    let d = db();
    for e in d["entries"].as_array().unwrap() {
        if e["status"] == "verified" {
            assert!(d["sources"][e["source"].as_str().unwrap()].is_object(), "{}", e["id"]);
            assert!(e["section"].is_string() && e["quote"].is_string(), "{}", e["id"]);
        } else {
            assert_eq!(e["status"], "unverified", "{}", e["id"]);
        }
    }
}
