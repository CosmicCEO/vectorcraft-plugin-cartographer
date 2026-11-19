use cartographer_core::builder::{build_text, set_cross_section_text, set_joint_text, set_tracks_text, set_yard_text};
use serde_json::{Value, json};

const TWO: &str = "lane back | lane fwd";

fn line(id: i64, name: &str, pts: &[[f64; 2]], closed: bool) -> Value {
    json!({"id": id, "name": name, "path": {"subpaths": [{"anchors": pts.iter().map(|p| json!({"p": p})).collect::<Vec<_>>(), "closed": closed}]}})
}

/// A track line 200 m of ground (1,250 mm of model) long.
fn track(id: i64, name: &str) -> Value {
    line(id, name, &[[0.0, 0.0], [200.0, 0.0]], false)
}

fn params(document_data: Value, description: &str) -> Value {
    json!({"description": description, "_context": {"documentData": document_data, "artboard": [0, 0, 600, 800]}})
}

fn parsed(text: Result<String, String>) -> Value {
    serde_json::from_str(&text.unwrap()).unwrap()
}

fn names(v: &Value) -> Vec<String> {
    v["objects"].as_array().unwrap().iter().filter_map(|o| o["name"].as_str().map(String::from)).collect()
}

#[test]
fn every_rail_command_and_a_track_build_refuse_a_newer_schema() {
    let input = json!({"objects": [track(1, "track.a")]});
    let p = params(json!({"schema": 9}), r#"{"tracks": 1}"#);
    let commands: [(&str, fn(&Value, &Value) -> Result<String, String>); 4] = [("set_tracks", set_tracks_text), ("set_yard", set_yard_text), ("set_joint", set_joint_text), ("build", build_text)];
    for (name, f) in commands {
        let e = f(&input, &p).unwrap_err();
        assert!(e.contains("newer Cartographer"), "{name}: {e}");
    }
}

#[test]
fn settings_road_data_and_unknown_keys_survive_a_rail_command() {
    let data = json!({"schema": 1, "settings": {"seed": 7}, "segments": {"a": {"cross_section": TWO}}, "extra": {"k": 1}});
    let out = parsed(set_tracks_text(&json!({"objects": [track(1, "track.b")]}), &params(data, r#"{"tracks": 2}"#)));
    let kept = &out["documentData"];
    assert_eq!(kept["settings"]["seed"], 7);
    assert_eq!(kept["segments"]["a"]["cross_section"], TWO);
    assert_eq!(kept["extra"]["k"], 1);
    assert_eq!(kept["tracks"]["b"]["tracks"], 2);
    assert_eq!(kept["schema"], 1, "the schema is written even when the document had none");
}

#[test]
fn roads_and_tracks_build_together() {
    let input = json!({"objects": [line(1, "road.a", &[[0.0, 300.0], [400.0, 300.0]], false), track(2, "track.b")]});
    let data = json!({
        "schema": 1,
        "segments": {"a": {"cross_section": TWO}},
        "wets": {"x1": {"road": "zz", "water": "w", "treatment": "bridge"}},
        "joints": {"j1": {"kind": "crossing", "lines": ["b", "zz"]}},
    });
    let out = parsed(build_text(&input, &params(data, "")));
    let n = names(&out);
    let last_road = n.iter().rposition(|x| x.starts_with("gen.road.")).expect("road objects");
    let first_track = n.iter().position(|x| x.starts_with("gen.track.")).expect("track objects");
    assert!(last_road < first_track, "track objects come after road objects: {n:?}");
    let notes: Vec<String> = out["documentData"]["notes"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
    assert!(notes.iter().any(|x| x.contains("crossing x1")), "a road note: {notes:?}");
    assert!(notes.iter().any(|x| x.contains("Joint j1")), "a track note: {notes:?}");
    // Both lines keep their highlight, and every kept object is still there.
    let kept: Vec<i64> = out["objects"].as_array().unwrap().iter().filter_map(|o| if o.get("name").is_none() { o["id"].as_i64() } else { None }).collect();
    assert!(kept.contains(&1) && kept.contains(&2));
}

#[test]
fn nothing_to_build_names_both_kinds() {
    let input = json!({"objects": [line(1, "something", &[[0.0, 0.0], [5.0, 5.0]], false)]});
    let e = build_text(&input, &params(json!({}), "")).unwrap_err();
    assert!(e.contains("Nothing to build") && e.contains("road.<id>") && e.contains("track.<id>"), "{e}");
}

#[test]
fn a_one_point_track_and_a_closed_track_make_no_empty_object() {
    let input = json!({"objects": [line(1, "track.a", &[[10.0, 10.0]], false), line(2, "track.b", &[[0.0, 0.0], [100.0, 0.0], [100.0, 100.0]], true)]});
    match build_text(&input, &params(json!({}), "")) {
        Ok(text) => {
            let out: Value = serde_json::from_str(&text).unwrap();
            for o in out["objects"].as_array().unwrap() {
                if o["name"].as_str().is_some_and(|n| n.starts_with("gen.")) {
                    let subs = o["path"]["subpaths"].as_array().unwrap();
                    assert!(subs.iter().any(|s| s["anchors"].as_array().unwrap().len() >= 2), "empty object {}", o["name"]);
                }
            }
        }
        Err(e) => assert!(!e.trim().is_empty(), "a plain message"),
    }
}

#[test]
fn a_yard_on_a_line_that_is_too_short_is_refused_or_drawn_as_mains_with_a_note() {
    let short = json!({"objects": [line(1, "track.t1", &[[0.0, 0.0], [1.0, 0.0]], false)]});
    let e = set_yard_text(&short, &params(json!({}), r#"{"left": 8, "ends": "merge"}"#)).unwrap_err();
    assert!(e.contains("needs") && e.contains("longer"), "{e}");
    let built = parsed(build_text(&short, &params(json!({"yards": {"t1": {"mains": 2, "left": 8, "ends": "merge"}}}), "")));
    let notes = built["documentData"]["notes"].as_array().unwrap();
    assert!(notes.iter().any(|n| n.as_str().unwrap().contains("mains")), "{notes:?}");
}

#[test]
fn a_road_line_is_never_turned_into_a_track_and_a_track_line_never_into_a_road() {
    let road = json!({"objects": [line(1, "road.a", &[[0.0, 0.0], [100.0, 0.0]], false)]});
    let e = set_tracks_text(&road, &params(json!({"segments": {"a": {"cross_section": TWO}}}), r#"{"tracks": 2}"#)).unwrap_err();
    assert!(e.contains("Select the line"), "{e}");
    let water = json!({"objects": [line(1, "water.w", &[[0.0, 0.0], [100.0, 0.0]], false)]});
    assert!(set_tracks_text(&water, &params(json!({}), r#"{"tracks": 2}"#)).is_err(), "a water line is not a plain line");
    let tr = json!({"objects": [track(1, "track.a")]});
    let mut p = params(json!({}), "");
    p["cross_section"] = json!(TWO);
    let out = set_cross_section_text(&tr, &p);
    assert!(out.is_err(), "a track line is not a plain line for Describe Road: {out:?}");
}

#[test]
fn a_big_track_build_gets_the_point_note() {
    let pts: Vec<[f64; 2]> = (0..12_500).map(|i| [f64::from(i) * 0.5, 0.0]).collect();
    let input = json!({"objects": [line(1, "track.a", &pts, false)]});
    let out = parsed(build_text(&input, &params(json!({}), "")));
    let notes: Vec<&str> = out["documentData"]["notes"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
    assert!(notes.iter().any(|n| n.contains("12500") && n.contains("split")), "{notes:?}");
}

#[test]
fn a_track_only_build_keeps_the_road_data_and_the_custom_library() {
    let data = json!({
        "schema": 1,
        "segments": {"a": {"cross_section": TWO, "anchor": "left", "extra": 1}},
        "library": {"elements": {"mine": {"width": 2.0}}},
    });
    let out = parsed(build_text(&json!({"objects": [track(1, "track.b")]}), &params(data, "")));
    let kept = &out["documentData"];
    assert_eq!(kept["segments"]["a"]["cross_section"], TWO);
    assert_eq!(kept["segments"]["a"]["extra"], 1);
    assert_eq!(kept["library"]["elements"]["mine"]["width"], 2.0);
    assert!(names(&out).iter().any(|n| n.starts_with("gen.track.")), "{:?}", names(&out));
}

#[test]
fn a_one_point_track_line_says_it_drew_nothing() {
    let out = parsed(build_text(&json!({"objects": [line(1, "track.a", &[[10.0, 10.0]], false)]}), &params(json!({}), "")));
    assert!(!names(&out).iter().any(|n| n.starts_with("gen.")), "{:?}", names(&out));
    let notes: Vec<&str> = out["documentData"]["notes"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
    assert!(notes.iter().any(|n| n.contains("Track a") && n.contains("two points")), "{notes:?}");
}
