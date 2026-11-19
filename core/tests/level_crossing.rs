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
    assert_eq!(f["options"], json!(["level", "gated", "road over", "track over"]));
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


// ---- review fixes

fn ys_of(o: &Value) -> Vec<f64> {
    points(o).iter().map(|p| p[1]).collect()
}

#[test]
fn a_road_drawn_from_one_edge_gets_a_crossing_only_where_the_road_is() {
    let mut d = data(json!({"level_crossings": {"lc1": {"road": "a", "track": "t", "treatment": "level"}}}));
    d["segments"]["a"]["anchor"] = json!("left");
    let objs = objects(&build_text(&input(), &params(d, "")).unwrap());
    let ys = ys_of(named(&objs, "gen.track.t.level.lc1.panel")[0]);
    let (lo, hi) = (ys.iter().cloned().fold(f64::MAX, f64::min), ys.iter().cloned().fold(f64::MIN, f64::max));
    // The road covers y 100.0 to 107.2 (anchored at its left edge); the panel reaches 0.5 m past each edge, no more.
    assert!(lo > 99.0 && hi < 108.0, "panel from {lo} to {hi}");
    let ties: Vec<[f64; 2]> = named(&objs, "gen.track.t.").into_iter().filter(|o| o["name"].as_str().unwrap().contains("ties")).flat_map(points).collect();
    assert!(ties.iter().any(|p| p[1] > 93.0 && p[1] < 99.0), "ties stay on the bare side of the road line");
}

#[test]
fn a_road_in_pieces_still_gets_its_crossing() {
    let pieces = json!({"objects": [line(1, "road.a", &[[0.0, 100.0], [200.0, 100.0]]), line(3, "road.a.2", &[[200.0, 100.0], [600.0, 100.0]]), line(2, "track.t", &[[300.0, 0.0], [300.0, 200.0]])]});
    let objs = objects(&build_text(&pieces, &params(data(json!({"level_crossings": {"lc1": {"road": "a", "track": "t", "treatment": "level"}}})), "")).unwrap());
    assert_eq!(named(&objs, "gen.track.t.level.lc1.panel").len(), 1);
    let two = json!({"objects": [line(1, "road.a", &[[0.0, 100.0], [200.0, 100.0]]), line(3, "road.a.2", &[[200.0, 100.0], [600.0, 100.0]]), line(2, "track.t", &[[300.0, 0.0], [300.0, 200.0]])]});
    assert!(set_level_crossing_text(&two, &params(data(json!({})), "")).is_ok(), "two pieces of one road and a track are one road and one track");
}

#[test]
fn the_hand_drawn_style_does_not_move_the_panel() {
    let lc = json!({"level_crossings": {"lc1": {"road": "a", "track": "t", "treatment": "level"}}});
    let mut hand = data(lc.clone());
    hand["settings"] = json!({"style": "hand-drawn"});
    let a = objects(&build_text(&input(), &params(data(lc), "")).unwrap());
    let b = objects(&build_text(&input(), &params(hand, "")).unwrap());
    assert_eq!(points(named(&a, "gen.track.t.level.lc1.panel")[0]), points(named(&b, "gen.track.t.level.lc1.panel")[0]));
}

#[test]
fn a_track_crossing_a_road_with_no_level_crossing_gets_a_note() {
    let v: Value = serde_json::from_str(&build_text(&input(), &params(data(json!({})), "")).unwrap()).unwrap();
    let notes = v["documentData"]["notes"].to_string();
    assert!(notes.contains("track t crosses road a") && notes.contains("level crossing"), "{notes}");
    let with = json!({"level_crossings": {"lc1": {"road": "a", "track": "t", "treatment": "level"}}});
    let v: Value = serde_json::from_str(&build_text(&input(), &params(data(with), "")).unwrap()).unwrap();
    assert!(!v["documentData"]["notes"].to_string().contains("crosses road"), "no note once the crossing exists");
}

#[test]
fn each_gate_stands_on_the_approaching_drivers_right() {
    // The road runs along +x at y = 100 (y grows down the page); traffic heading +x has its right at larger y.
    let objs = objects(&build_text(&input(), &params(data(json!({"level_crossings": {"lc1": {"road": "a", "track": "t", "treatment": "gated"}}})), "")).unwrap());
    let pole = |tag: &str| points(named(&objs, &format!("gen.track.t.level.lc1.gate.{tag}"))[0])[0];
    let (before, after) = (pole("before"), pole("after"));
    assert!(before[0] < 300.0 && before[1] > 100.0, "traffic from the left (heading +x) has its gate below the road line: {before:?}");
    assert!(after[0] > 300.0 && after[1] < 100.0, "traffic from the right (heading -x) has its gate above the road line: {after:?}");
}

#[test]
fn the_missing_crossing_note_survives_a_busy_document() {
    // 35 stored crossings whose roads are gone make 35 notes; the note about the real, unmade crossing must still be kept.
    let mut many = serde_json::Map::new();
    for i in 0..35 {
        many.insert(format!("old{i}"), json!({"road": format!("gone{i}"), "track": "t", "treatment": "level"}));
    }
    let v: Value = serde_json::from_str(&build_text(&input(), &params(data(json!({"level_crossings": Value::Object(many)})), "")).unwrap()).unwrap();
    let notes = v["documentData"]["notes"].to_string();
    assert!(notes.contains("track t crosses road a"), "{notes}");
}


fn built(treatment: &str) -> Vec<Value> {
    objects(&build_text(&input(), &params(data(json!({"level_crossings": {"lc1": {"road": "a", "track": "t", "treatment": treatment}}})), "")).unwrap())
}

#[test]
fn road_over_lays_the_road_deck_on_top_of_the_track() {
    let objs = built("road over");
    assert!(named(&objs, "gen.track.t.level.lc1.panel").is_empty(), "a bridge has no crossing panel");
    let deck = named(&objs, "gen.track.t.level.lc1.deck");
    assert!(!deck.is_empty(), "the road surface is laid over the track");
    let pos = |o: &Value| objs.iter().position(|x| x == o).unwrap();
    let last_rail = named(&objs, "gen.track.t").iter().filter(|o| o["name"].as_str().unwrap().ends_with(".rails")).map(|o| pos(o)).max().unwrap();
    assert!(deck.iter().all(|d| pos(d) > last_rail), "the deck is above the rails");
    // The deck spans the road (y 100 ± 3.5) and is wider than the track's ties.
    let span = |v: Vec<f64>| (v.iter().cloned().fold(f64::MAX, f64::min), v.iter().cloned().fold(f64::MIN, f64::max));
    let (y0, y1) = span(deck.iter().flat_map(|d| points(d)).map(|p| p[1]).collect());
    let (x0, x1) = span(deck.iter().flat_map(|d| points(d)).map(|p| p[0]).collect());
    let (t0, t1) = span(named(&objs, "gen.track.t").iter().filter(|o| o["name"].as_str().unwrap().ends_with(".ties")).flat_map(|o| points(o)).map(|p| p[0]).collect());
    assert!(y0 <= 96.5 && y1 >= 103.5, "{y0}..{y1}");
    assert!(x0 < t0 && x1 > t1, "deck {x0}..{x1}, ties {t0}..{t1}");
    assert_eq!(named(&objs, "gen.track.t.level.lc1.bridge").len(), 2, "a parapet on each side of the road");
}

#[test]
fn track_over_keeps_every_tie_and_draws_the_bridge_sides() {
    let objs = built("track over");
    assert!(named(&objs, "gen.track.t.level.lc1.panel").is_empty() && named(&objs, "gen.track.t.level.lc1.deck").is_empty());
    let ties = |objs: &[Value]| named(objs, "gen.track.t").iter().filter(|o| o["name"].as_str().unwrap().ends_with(".ties")).map(|o| o["path"]["subpaths"].as_array().unwrap().len()).sum::<usize>();
    let plain = objects(&build_text(&input(), &params(data(json!({})), "")).unwrap());
    assert_eq!(ties(&objs), ties(&plain), "the track runs on over the road with all its ties");
    let sides = named(&objs, "gen.track.t.level.lc1.bridge");
    assert_eq!(sides.len(), 2, "a parapet on each side of the track");
    for s in sides {
        let p = points(s);
        assert!(p.iter().all(|q| (q[0] - 300.0).abs() > 1.0 && (q[0] - 300.0).abs() < 5.0), "beside the track: {p:?}");
        let (y0, y1) = (p.iter().map(|q| q[1]).fold(f64::MAX, f64::min), p.iter().map(|q| q[1]).fold(f64::MIN, f64::max));
        assert!(y0 < 96.5 && y1 > 103.5, "spans the road: {y0}..{y1}");
    }
}

#[test]
fn the_card_and_the_command_offer_both_bridges() {
    for t in ["road over", "track over"] {
        let text = set_level_crossing_text(&input(), &params(data(json!({})), &format!(r#"{{"treatment": "{t}"}}"#))).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&text).unwrap()["documentData"]["level_crossings"]["lc1"]["treatment"], t);
    }
    let card = cartographer_core::schema::level_crossing_schema();
    assert_eq!(card["fields"][0]["options"], json!(["level", "gated", "road over", "track over"]));
}
