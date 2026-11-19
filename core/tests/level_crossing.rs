use cartographer_core::builder::{build_text, set_level_crossing_text};
use serde_json::{Value, json};

const TWO: &str = "lane back | lane fwd";

fn line(id: i64, name: &str, pts: &[[f64; 2]]) -> Value {
    json!({"id": id, "name": name, "path": {"subpaths": [{"anchors": pts.iter().map(|p| json!({"p": p})).collect::<Vec<_>>(), "closed": false}]}})
}

/// A road along y = 100 and a single track crossing it at x = 300.
fn input() -> Value {
    json!({"objects": [line(1, "road.a", &[[0.0, 100.0], [600.0, 100.0]]), line(2, "track.t", &[[300.0, 0.0], [300.0, 200.0]])]})
}

fn data(extra: Value) -> Value {
    let mut d = json!({"schema": 1, "segments": {"a": {"cross_section": TWO}}, "tracks": {"t": {"tracks": 1}}});
    for (k, v) in extra.as_object().unwrap() {
        d[k] = v.clone();
    }
    d
}

fn params(data: Value, description: &str) -> Value {
    json!({"description": description, "_context": {"documentData": data, "artboard": [0, 0, 800, 800]}})
}

fn objects(text: &str) -> Vec<Value> {
    serde_json::from_str::<Value>(text).unwrap()["objects"].as_array().unwrap().clone()
}

fn named<'a>(objs: &'a [Value], prefix: &str) -> Vec<&'a Value> {
    objs.iter().filter(|o| o["name"].as_str().is_some_and(|n| n.starts_with(prefix))).collect()
}

fn points(o: &Value) -> Vec<[f64; 2]> {
    o["path"]["subpaths"].as_array().unwrap().iter().flat_map(|s| s["anchors"].as_array().unwrap().iter().map(|a| [a["p"][0].as_f64().unwrap(), a["p"][1].as_f64().unwrap()])).collect()
}

#[test]
fn the_command_stores_a_level_crossing_for_a_road_and_a_track_that_cross() {
    let text = set_level_crossing_text(&input(), &params(data(json!({})), r#"{"treatment": "gated"}"#)).unwrap();
    let v: Value = serde_json::from_str(&text).unwrap();
    let rec = &v["documentData"]["level_crossings"]["lc1"];
    assert_eq!(rec["road"], "a");
    assert_eq!(rec["track"], "t");
    assert_eq!(rec["treatment"], "gated");
    // Every selected object is kept.
    assert_eq!(v["objects"].as_array().unwrap().len(), 2);
}

#[test]
fn the_command_refuses_lines_that_do_not_cross_and_a_wrong_selection() {
    let apart = json!({"objects": [line(1, "road.a", &[[0.0, 100.0], [600.0, 100.0]]), line(2, "track.t", &[[300.0, 150.0], [300.0, 400.0]])]});
    let e = set_level_crossing_text(&apart, &params(data(json!({})), "")).unwrap_err();
    assert!(e.contains("do not cross"), "{e}");
    let two_roads = json!({"objects": [line(1, "road.a", &[[0.0, 100.0], [600.0, 100.0]]), line(2, "road.b", &[[300.0, 0.0], [300.0, 200.0]])]});
    let e = set_level_crossing_text(&two_roads, &params(data(json!({})), "")).unwrap_err();
    assert!(e.contains("one road") && e.contains("one track"), "{e}");
    let bad = set_level_crossing_text(&input(), &params(data(json!({})), r#"{"treatment": "drawbridge"}"#)).unwrap_err();
    assert!(bad.contains("level") && bad.contains("gated"), "{bad}");
}

#[test]
fn a_level_crossing_draws_a_panel_cuts_the_ties_under_the_road_and_keeps_the_rails() {
    let plain = objects(&build_text(&input(), &params(data(json!({})), "")).unwrap());
    let lc = objects(&build_text(&input(), &params(data(json!({"level_crossings": {"lc1": {"road": "a", "track": "t", "treatment": "level"}}})), "")).unwrap());
    assert_eq!(named(&lc, "gen.track.t.level.lc1.panel").len(), 1, "a crossing panel");
    // Ties: none under the road (the road is 7.2 m wide about y = 100), some elsewhere.
    let ties: Vec<[f64; 2]> = named(&lc, "gen.track.t.").into_iter().filter(|o| o["name"].as_str().unwrap().contains("ties")).flat_map(points).collect();
    assert!(!ties.is_empty());
    assert!(ties.iter().all(|p| (p[1] - 100.0).abs() > 3.6), "a tie under the road: {:?}", ties.iter().find(|p| (p[1] - 100.0).abs() <= 3.6));
    let plain_ties = named(&plain, "gen.track.t.").into_iter().filter(|o| o["name"].as_str().unwrap().contains("ties")).flat_map(points).count();
    assert!(ties.len() < plain_ties);
    // The rails are drawn across exactly as without the crossing.
    let rails = |objs: &[Value]| named(objs, "gen.track.t.").into_iter().filter(|o| o["name"].as_str().unwrap().contains("rails")).map(points).collect::<Vec<_>>();
    assert_eq!(rails(&lc), rails(&plain));
    // The panel lies between the ballast and the rails: after every roadbed object and before the rails.
    let names: Vec<&str> = lc.iter().filter_map(|o| o["name"].as_str()).collect();
    let panel = names.iter().position(|n| *n == "gen.track.t.level.lc1.panel").unwrap();
    let rail = names.iter().position(|n| n.contains("rails")).unwrap();
    let ballast = names.iter().position(|n| n.contains("ballast")).unwrap();
    assert!(ballast < panel && panel < rail, "{names:?}");
    assert!(named(&lc, "gen.track.t.level.lc1.gate").is_empty(), "no gates on a plain level crossing");
}

#[test]
fn a_gated_crossing_adds_crossbucks_and_gates_on_both_approaches() {
    let lc = objects(&build_text(&input(), &params(data(json!({"level_crossings": {"lc1": {"road": "a", "track": "t", "treatment": "gated"}}})), "")).unwrap());
    assert_eq!(named(&lc, "gen.track.t.level.lc1.crossbuck").len(), 2);
    assert_eq!(named(&lc, "gen.track.t.level.lc1.gate").len(), 2);
}

#[test]
fn a_level_crossing_whose_lines_are_gone_or_apart_is_left_out_with_a_note() {
    let only_road = json!({"objects": [line(1, "road.a", &[[0.0, 100.0], [600.0, 100.0]])]});
    let v: Value = serde_json::from_str(&build_text(&only_road, &params(data(json!({"level_crossings": {"lc1": {"road": "a", "track": "t", "treatment": "level"}}})), "")).unwrap()).unwrap();
    let notes = v["documentData"]["notes"].to_string();
    assert!(notes.contains("lc1"), "{notes}");
}

#[test]
fn the_level_crossing_schema_offers_the_treatments() {
    let s = cartographer_core::schema::level_crossing_schema();
    assert_eq!(s["kind"], "level_crossing");
    let f = &s["fields"][0];
    assert_eq!(f["key"], "treatment");
    assert_eq!(f["options"], json!(["level", "gated"]));
    assert_eq!(f["default"], "level");
}

#[test]
fn the_panel_covers_a_whole_yard_not_only_its_mains() {
    let yard = json!({"yards": {"t": {"mains": 1, "left": 2, "right": 2}}, "level_crossings": {"lc1": {"road": "a", "track": "t", "treatment": "level"}}});
    let mut d = data(yard);
    d.as_object_mut().unwrap().remove("tracks");
    let objs = objects(&build_text(&input(), &params(d, "")).unwrap());
    let panel = named(&objs, "gen.track.t.level.lc1.panel")[0];
    let xs: Vec<f64> = points(panel).iter().map(|p| p[0]).collect();
    let width = xs.iter().cloned().fold(f64::MIN, f64::max) - xs.iter().cloned().fold(f64::MAX, f64::min);
    // Two spurs each side of one main at 24.765 mm (0.16 points per mm) is about 4 spacings = 15.8 points, plus a tie.
    assert!(width > 15.0, "the panel spans the yard: {width}");
}

