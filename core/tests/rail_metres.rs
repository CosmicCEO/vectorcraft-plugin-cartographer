use cartographer_core::builder::{build_text, set_tracks_text, set_yard_text};
use cartographer_core::rail_units::{metres_text, record_to_model, upgrade};
use serde_json::{Value, json};

fn line(id: i64, name: &str, pts: &[[f64; 2]]) -> Value {
    json!({"id": id, "name": name, "path": {"subpaths": [{"anchors": pts.iter().map(|p| json!({"p": p})).collect::<Vec<_>>(), "closed": false}]}})
}

fn params(data: Value, description: &str) -> Value {
    json!({"description": description, "_context": {"documentData": data, "artboard": [0, 0, 800, 800]}})
}

fn stored(text: &str) -> Value {
    serde_json::from_str::<Value>(text).unwrap()["documentData"].clone()
}

#[test]
fn describe_track_takes_and_stores_metres() {
    let input = json!({"objects": [line(1, "track.t1", &[[0.0, 0.0], [300.0, 0.0]])]});
    let out = set_tracks_text(&input, &params(json!({}), r#"{"tracks": 2, "spacing": 4}"#)).unwrap();
    let d = stored(&out);
    assert_eq!(d["tracks"]["t1"]["spacing"], 4.0, "stored in metres: {}", d["tracks"]["t1"]);
    assert_eq!(d["schema"], 2);
    // Drawn 4 m apart: the two rails' centres are 4 points apart across the line (y).
    let built = build_text(&input, &params(d, "")).unwrap();
    let v: Value = serde_json::from_str(&built).unwrap();
    let ys: Vec<f64> = v["objects"].as_array().unwrap().iter().filter(|o| o["name"].as_str().is_some_and(|n| n.ends_with(".ties"))).flat_map(|o| o["path"]["subpaths"].as_array().unwrap().iter().map(|s| s["anchors"][0]["p"][1].as_f64().unwrap() + s["anchors"][1]["p"][1].as_f64().unwrap()).collect::<Vec<_>>()).collect();
    let (lo, hi) = (ys.iter().cloned().fold(f64::MAX, f64::min) / 2.0, ys.iter().cloned().fold(f64::MIN, f64::max) / 2.0);
    assert!((hi - lo - 4.0).abs() < 0.05, "tie centres {lo} .. {hi}");
}

#[test]
fn describe_yard_takes_lengths_in_metres() {
    let input = json!({"objects": [line(1, "track.t1", &[[0.0, 0.0], [600.0, 0.0]])]});
    let out = set_yard_text(&input, &params(json!({}), r#"{"mains": 1, "left": 2, "at": 10, "length": 300}"#)).unwrap();
    let y = &stored(&out)["yards"]["t1"];
    assert_eq!((y["at"].as_f64(), y["length"].as_f64()), (Some(10.0), Some(300.0)), "{y}");
    let short = set_yard_text(&input, &params(json!({}), r#"{"mains": 1, "left": 8, "ends": "merge", "length": 20}"#)).unwrap_err();
    assert!(!short.contains("mm") && short.contains(" m"), "the refusal is in metres: {short}");
}

#[test]
fn a_document_saved_in_millimetres_is_upgraded() {
    let old = json!({"schema": 1, "tracks": {"t1": {"tracks": 2, "spacing": 24.765, "ballast": true, "ties": true}}, "yards": {"y": {"mains": 1, "at": 100.0, "length": 2000.0, "crossovers": [{"from": 1, "to": 1, "at": 500.0}]}}});
    let up = upgrade(&old);
    assert_eq!(up["schema"], 2);
    assert!((up["tracks"]["t1"]["spacing"].as_f64().unwrap() - 3.962).abs() < 0.001);
    assert!((up["yards"]["y"]["length"].as_f64().unwrap() - 320.0).abs() < 0.001);
    assert!((up["yards"]["y"]["crossovers"][0]["at"].as_f64().unwrap() - 80.0).abs() < 0.001);
    assert_eq!(upgrade(&up), up, "upgrading twice changes nothing");
    // The same drawing from the old record and from its metres.
    let input = json!({"objects": [line(1, "track.t1", &[[0.0, 0.0], [300.0, 0.0]])]});
    let objs = |d: Value| serde_json::from_str::<Value>(&build_text(&input, &params(d, "")).unwrap()).unwrap()["objects"].clone();
    assert_eq!(objs(old.clone()), objs(up.clone()));
    let built = stored(&build_text(&input, &params(old, "")).unwrap());
    assert_eq!(built["schema"], 2);
    assert!((built["tracks"]["t1"]["spacing"].as_f64().unwrap() - 3.962).abs() < 0.001, "a build writes the record in metres");
}

#[test]
fn text_lengths_are_read_in_real_units() {
    let m = record_to_model(&json!({"spacing": "13ft", "at": "2m", "length": 1.5}));
    assert_eq!(m["spacing"], "13ft", "feet are real feet, handled inside");
    assert_eq!(m["at"].as_f64(), Some(12.5), "2 m of ground is 12.5 mm of 1:160 model");
    assert_eq!(m["length"].as_f64(), Some(9.375));
}

#[test]
fn messages_speak_metres() {
    assert_eq!(metres_text("needs 2524 mm of line"), "needs 403.84 m of line");
    assert_eq!(metres_text("24.8 mm between centres"), "3.97 m between centres");
    assert_eq!(metres_text("13 ft prototype = 24.8 mm"), "13 ft = 3.97 m");
    assert_eq!(metres_text("No. 6 turnout"), "No. 6 turnout");
}

#[test]
fn millimetres_and_inches_in_a_description_are_real_ones() {
    let m = record_to_model(&json!({"spacing": "4000mm", "at": "100cm", "length": "1000in"}));
    assert_eq!(m["spacing"].as_f64(), Some(25.0), "4000 mm of ground is 4 m");
    assert_eq!(m["at"].as_f64(), Some(6.25));
    assert!((m["length"].as_f64().unwrap() - 25.4 * 6.25).abs() < 1e-9);
}

#[test]
fn a_yard_can_stop_short_of_the_end_of_its_line() {
    let input = json!({"objects": [line(1, "track.t1", &[[0.0, 0.0], [600.0, 0.0]])]});
    let out = set_yard_text(&input, &params(json!({}), r#"{"mains": 1, "left": 2, "ends": "merge", "at": 10, "before_end": 30}"#)).unwrap();
    let d = stored(&out);
    assert_eq!(d["yards"]["t1"]["before_end"].as_f64(), Some(30.0), "stored in metres: {}", d["yards"]["t1"]);
    let v: Value = serde_json::from_str(&build_text(&input, &params(d, "")).unwrap()).unwrap();
    let xs: Vec<f64> = v["objects"].as_array().unwrap().iter()
        .filter(|o| o["name"].as_str().is_some_and(|n| n.starts_with("gen.track.t1.L")))
        .flat_map(|o| o["path"]["subpaths"].as_array().unwrap().iter().flat_map(|s| s["anchors"].as_array().unwrap().iter().map(|a| a["p"][0].as_f64().unwrap())).collect::<Vec<_>>())
        .collect();
    assert!(!xs.is_empty(), "the spurs are drawn");
    let (lo, hi) = (xs.iter().cloned().fold(f64::MAX, f64::min), xs.iter().cloned().fold(f64::MIN, f64::max));
    assert!(lo > 9.0 && hi < 571.0 && hi > 560.0, "the spurs run from 10 m to 30 m before the end (570 m): {lo} .. {hi}");
}

#[test]
fn before_end_and_length_cannot_both_be_given() {
    let input = json!({"objects": [line(1, "track.t1", &[[0.0, 0.0], [600.0, 0.0]])]});
    let e = set_yard_text(&input, &params(json!({}), r#"{"left": 1, "length": 300, "before_end": 30}"#)).unwrap_err();
    assert!(e.contains("before_end") && e.contains("length") && e.contains("not both"), "{e}");
    assert!(set_yard_text(&input, &params(json!({}), r#"{"left": 1, "before_end": -1}"#)).is_err());
}

#[test]
fn every_turnout_in_a_ladder_stands_on_ties() {
    // Owner, 1.6 check: after the first turnout the ladder ran bare. The ladder's own ties must start where it is clear of the main,
    // before the first spur leaves it, so every turnout along it has ties under it.
    let input = json!({"objects": [line(1, "track.t1", &[[5.0, 50.0], [255.0, 50.0]])]});
    let d = json!({"schema": 2, "yards": {"t1": {"mains": 1, "left": 3, "frog": 6}}});
    let v: Value = serde_json::from_str(&build_text(&input, &params(d, "")).unwrap()).unwrap();
    let start = |name: &str| v["objects"].as_array().unwrap().iter().find(|o| o["name"] == name).unwrap()["path"]["subpaths"].as_array().unwrap()
        .iter().map(|s| s["anchors"][0]["p"][0].as_f64().unwrap()).fold(f64::MAX, f64::min);
    let (switch_end, ladder, spur1) = (
        v["objects"].as_array().unwrap().iter().find(|o| o["name"] == "gen.track.t1.m1.switch-ties").unwrap()["path"]["subpaths"].as_array().unwrap()
            .iter().map(|s| s["anchors"][0]["p"][0].as_f64().unwrap()).fold(f64::MIN, f64::max),
        start("gen.track.t1.L3.ties"),
        start("gen.track.t1.L1.ties"),
    );
    assert!(ladder < spur1, "the ladder has ties before the first spur leaves it: ladder from {ladder}, spur 1 from {spur1}");
    assert!(ladder - switch_end < 3.0, "the ladder's ties take over where the main's switch ties end: {switch_end} then {ladder}");
}

fn segs(v: &Value, name: &str) -> Vec<[[f64; 2]; 2]> {
    v["objects"].as_array().unwrap().iter().filter(|o| o["name"] == name).flat_map(|o| o["path"]["subpaths"].as_array().unwrap().iter().map(|s| {
        let p = |i: usize| [s["anchors"][i]["p"][0].as_f64().unwrap(), s["anchors"][i]["p"][1].as_f64().unwrap()];
        [p(0), p(1)]
    }).collect::<Vec<_>>()).collect()
}

fn cross(a: [[f64; 2]; 2], b: [[f64; 2]; 2]) -> bool {
    let o = |p: [f64; 2], q: [f64; 2], r: [f64; 2]| (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0]);
    o(a[0], a[1], b[0]) * o(a[0], a[1], b[1]) < -1e-9 && o(b[0], b[1], a[0]) * o(b[0], b[1], a[1]) < -1e-9
}

#[test]
fn a_spur_leaving_the_ladder_is_not_tied_twice() {
    for desc in [json!({"mains": 1, "left": 3, "frog": 6}), json!({"mains": 1, "right": 3, "ends": "merge", "frog": 8})] {
        let input = json!({"objects": [line(1, "track.t1", &[[0.0, 50.0], [350.0, 50.0]])]});
        let v: Value = serde_json::from_str(&build_text(&input, &params(json!({"schema": 2, "yards": {"t1": desc}}), "")).unwrap()).unwrap();
        let side = if desc.get("left").is_some() { "L" } else { "R" };
        let ext = segs(&v, &format!("gen.track.t1.{side}3.switch-ties"));
        assert!(!ext.is_empty());
        for j in 1..3 {
            let ties = segs(&v, &format!("gen.track.t1.{side}{j}.ties"));
            let hits = ext.iter().filter(|e| ties.iter().any(|t| cross(**e, *t))).count();
            assert_eq!(hits, 0, "{desc}: {hits} of the ladder's switch ties cross spur {j}'s own ties");
        }
    }
}

#[test]
fn switch_ties_follow_the_tie_budget() {
    // A big yard runs out of separate ties and draws some routes as a band: such a route gets no switch-tie strokes on top of its band.
    let input = json!({"objects": [line(1, "track.t1", &[[0.0, 50.0], [3000.0, 50.0]])]});
    let v: Value = serde_json::from_str(&build_text(&input, &params(json!({"schema": 2, "yards": {"t1": {"mains": 4, "left": 16, "right": 16}}}), "")).unwrap()).unwrap();
    let objs = v["objects"].as_array().unwrap();
    let banded: Vec<&str> = objs.iter().filter(|o| o["name"].as_str().is_some_and(|n| n.ends_with(".ties")) && o["fills"].is_array()).map(|o| o["name"].as_str().unwrap()).collect();
    assert!(!banded.is_empty(), "the budget is spent in this yard");
    for b in banded {
        let ext = b.replace(".ties", ".switch-ties");
        assert!(!objs.iter().any(|o| o["name"] == ext.as_str()), "{ext} is drawn over a band");
    }
}
