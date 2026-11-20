mod common;
use serde_json::{Value, json};
use cartographer_core::rail_builder as builder;
use cartographer_core::rail_standards::*;
use cartographer_core::tracks::{TrackSpec, min_radius, parse_length};

/// A filter input with one named line (document points, y down).
fn input(name: &str, pts: &[[f64; 2]]) -> Value {
    let anchors: Vec<Value> = pts.iter().map(|p| json!({"p": p})).collect();
    json!({"objects": [{"id": 1, "name": name, "path": {"subpaths": [{"anchors": anchors, "closed": false}]}}]})
}

fn params(spec: Value) -> Value {
    json!({"_context": {"documentData": {"tracks": {"a": spec}}}})
}

fn object<'a>(out: &'a Value, name: &str) -> &'a Value {
    out["objects"].as_array().unwrap().iter().find(|o| o["name"] == name).unwrap_or_else(|| panic!("no object {name}"))
}

/// Anchor points of subpath `i`, as document points.
fn sub(o: &Value, i: usize) -> Vec<[f64; 2]> {
    o["path"]["subpaths"][i]["anchors"].as_array().unwrap().iter().map(|a| [a["p"][0].as_f64().unwrap(), a["p"][1].as_f64().unwrap()]).collect()
}

const LEN: f64 = 100.0 * PT_PER_MM; // a 100 mm line

#[test]
fn one_track_rails_sit_half_a_gauge_either_side() {
    let out = builder::build(&input("track.a", &[[0.0, 0.0], [LEN, 0.0]]), &params(json!({"tracks": 1}))).unwrap();
    let rails = object(&out, "gen.track.a.t1.rails");
    let (l, r) = (sub(rails, 0), sub(rails, 1));
    let mm = |p: f64| p / PT_PER_MM;
    // Travelling +x on screen: right of travel is down the page (y grows). Left rail first.
    assert!((mm(l[0][1]) + GAUGE / 2.0).abs() < 1e-3 && (mm(r[0][1]) - GAUGE / 2.0).abs() < 1e-3, "{l:?} {r:?}");
    assert!((mm(r[1][0]) - 100.0).abs() < 1e-3);
}

#[test]
fn ties_one_per_pitch_across_the_track() {
    let out = builder::build(&input("track.a", &[[0.0, 0.0], [LEN, 0.0]]), &params(json!({"tracks": 1}))).unwrap();
    let ties = common::tie_segments(object(&out, "gen.track.a.t1.ties"));
    assert_eq!(ties.len(), 32); // centres 1.55, 4.65, ... 97.65 mm
    let first = ties[0];
    assert!(((first[1][1] - first[0][1]).abs() / PT_PER_MM - TIE_LENGTH).abs() < 1e-3);
    assert!((first[0][0] / PT_PER_MM - TIE_PITCH / 2.0).abs() < 1e-3);
    let last = ties[31];
    assert!((last[0][0] / PT_PER_MM - (TIE_PITCH / 2.0 + 31.0 * TIE_PITCH)).abs() < 1e-3);
}

#[test]
fn two_tracks_are_one_spacing_apart() {
    let out = builder::build(&input("track.a", &[[0.0, 0.0], [LEN, 0.0]]), &params(json!({"tracks": 2}))).unwrap();
    let (a, b) = (sub(object(&out, "gen.track.a.t1.rails"), 0), sub(object(&out, "gen.track.a.t2.rails"), 0));
    assert!(((b[0][1] - a[0][1]) / PT_PER_MM - TRACK_CENTERS).abs() < 1e-3);
    // Centred on the drawn line.
    let mid = (a[0][1] + b[0][1] + sub(object(&out, "gen.track.a.t1.rails"), 1)[0][1] + sub(object(&out, "gen.track.a.t2.rails"), 1)[0][1]) / 4.0;
    assert!(mid.abs() < 1e-3);
}

#[test]
fn oblique_line_keeps_the_gauge_square_to_the_line() {
    let out = builder::build(&input("track.a", &[[0.0, 0.0], [LEN, LEN]]), &params(json!({"tracks": 1}))).unwrap();
    let rails = object(&out, "gen.track.a.t1.rails");
    let (l, r) = (sub(rails, 0)[0], sub(rails, 1)[0]);
    assert!(((l[0] - r[0]).hypot(l[1] - r[1]) / PT_PER_MM - GAUGE).abs() < 1e-3);
    assert!(((l[0] - r[0]) + (l[1] - r[1])).abs() < 1e-6, "rail pair must be square to a 45 degree line"); // perpendicular to (1,1)
}

#[test]
fn spec_words_and_errors() {
    let s = TrackSpec::from_json(&json!({"tracks": 3, "spacing": "14ft"})).unwrap();
    assert_eq!(s.count, 3);
    assert!((s.spacing - 14.0 * FOOT).abs() < 1e-9);
    assert!((parse_length(&json!("1in"), "x").unwrap() - 25.4).abs() < 1e-9);
    assert!((parse_length(&json!(30), "x").unwrap() - 30.0).abs() < 1e-9);
    for bad in [json!({"tracks": 0}), json!({"tracks": 13}), json!({"spacing": "wide"}), json!({"spacing": 5}), json!({"colour": "red"}), json!(3)] {
        assert!(TrackSpec::from_json(&bad).is_err(), "{bad}");
    }
}

#[test]
fn tight_curve_warns_and_wide_curve_does_not() {
    let arc = |r: f64| -> Vec<[f64; 2]> { (0..=40).map(|i| { let a = i as f64 * std::f64::consts::PI / 2.0 / 40.0; [r * a.sin() * PT_PER_MM, r * (1.0 - a.cos()) * PT_PER_MM] }).collect() };
    let notes = |r: f64, spec: Value| builder::build(&input("track.a", &arc(r)), &params(spec)).unwrap()["documentData"]["notes"].as_array().unwrap().len();
    assert_eq!(notes(150.0, json!({"tracks": 1})), 1);
    assert_eq!(notes(300.0, json!({"tracks": 1})), 0);
    // 300 mm drawn radius with 4 tracks: the inner track is 300 - 1.5 * 24.8 = 262.8 mm: fine; 3 tracks at 200 mm: inner 150: warns
    assert_eq!(notes(300.0, json!({"tracks": 4})), 0);
    assert_eq!(notes(200.0, json!({"tracks": 3})), 1);
    let pts: Vec<[f64; 2]> = arc(250.0).iter().map(|p| [p[0] / PT_PER_MM, p[1] / PT_PER_MM]).collect();
    assert!((min_radius(&pts).unwrap() - 250.0).abs() < 0.5);
}

#[test]
fn set_tracks_names_a_plain_line_and_stores_the_description() {
    let p = json!({"description": "{\"tracks\": 2}", "_context": {"command": "set_tracks", "documentData": {}}});
    let out: Value = serde_json::from_str(&builder::set_tracks_text(&input("line", &[[0.0, 0.0], [LEN, 0.0]]), &p).unwrap()).unwrap();
    assert_eq!(out["objects"][0]["name"], "track.t1");
    assert_eq!(out["documentData"]["tracks"]["t1"]["tracks"], 2);
    assert!(builder::set_tracks_text(&json!({"objects": []}), &p).is_err());
}

#[test]
fn nothing_to_build_says_so() {
    assert!(builder::build(&input("line", &[[0.0, 0.0], [10.0, 0.0]]), &json!({})).unwrap_err().contains("track."));
}

#[test]
fn spacing_can_name_an_nmra_era() {
    for (word, feet) in [("classic", 13.0), ("Old-Time", 12.0), ("modern", 14.0), ("early modern", 14.0)] {
        let s = TrackSpec::from_json(&json!({"spacing": word})).unwrap();
        assert!((s.spacing - feet * FOOT).abs() < 1e-9, "{word}");
    }
    assert!(TrackSpec::from_json(&json!({"spacing": "baroque"})).unwrap_err().contains("era"));
}
