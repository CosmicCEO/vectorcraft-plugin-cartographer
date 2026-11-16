use cartographer_core::builder::{build_text, set_embankment_text};
use cartographer_core::embank::{EmbankmentSpec, Sides};
use cartographer_core::output::Out;
use cartographer_core::roads::{Spec, build_clipped};
use cartographer_core::vcdoc::Item;
use serde_json::{Value, json};

const TWO: &str = "lane back | lane fwd";

fn road(embankments: Value) -> (Vec<Item>, Spec) {
    // Road a runs north from (200, 700) to (200, 100): 600 m, 7.2 m wide (edges 3.6 m each side of the line).
    let items = vec![Item { closed: false, id: Some(1), name: "road.a".into(), subs: vec![vec![[200.0, 700.0], [200.0, 100.0]]] }];
    (items, Spec::from_document_data(&json!({"segments": {"a": {"cross_section": TWO}}, "embankments": {"a": embankments}})))
}

fn build(items: &[Item], spec: &Spec) -> Value {
    let mut out = Out::new();
    build_clipped(items, spec, &[], &mut out);
    serde_json::from_str(&out.finish()).unwrap()
}

fn names(v: &Value) -> Vec<String> {
    v["objects"].as_array().unwrap().iter().filter_map(|o| o["name"].as_str().map(String::from)).collect()
}

fn pts(v: &Value, name: &str) -> Vec<[f64; 2]> {
    let o = v["objects"].as_array().unwrap().iter().find(|o| o["name"] == name).unwrap_or_else(|| panic!("no object {name}: {:?}", names(v)));
    o["path"]["subpaths"].as_array().unwrap().iter().flat_map(|s| s["anchors"].as_array().unwrap().iter().map(|a| [a["p"][0].as_f64().unwrap(), a["p"][1].as_f64().unwrap()])).collect()
}

fn bbox(p: &[[f64; 2]]) -> ([f64; 2], [f64; 2]) {
    (
        [p.iter().map(|q| q[0]).fold(f64::INFINITY, f64::min), p.iter().map(|q| q[1]).fold(f64::INFINITY, f64::min)],
        [p.iter().map(|q| q[0]).fold(f64::NEG_INFINITY, f64::max), p.iter().map(|q| q[1]).fold(f64::NEG_INFINITY, f64::max)],
    )
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.02
}

#[test]
fn a_road_on_fill_has_a_slope_band_each_side_as_wide_as_height_times_slope() {
    let (items, spec) = road(json!([{"piece": 1, "from": 100, "to": 300, "height": 3, "slope": 2, "ramp": 20}]));
    let v = build(&items, &spec);
    // 3 m high at 2:1 reaches 6 m past the road's edge (x 196.4 and 203.6): west band x 190.4..196.4, east band x 203.6..209.6, y 400..600 (s 100 to 300).
    let (lo, hi) = bbox(&pts(&v, "gen.road.a.1.embank1.slope.left"));
    assert!(near(lo[0], 190.4) && near(hi[0], 196.4) && near(lo[1], 400.0) && near(hi[1], 600.0), "{lo:?} {hi:?}");
    let (lo, hi) = bbox(&pts(&v, "gen.road.a.1.embank1.slope.right"));
    assert!(near(lo[0], 203.6) && near(hi[0], 209.6), "{lo:?} {hi:?}");
    // The slope grows from nothing over the 20 m ramp: at the start (y 600) it has no width, 10 m in (y 590) it is half width.
    let ring = pts(&v, "gen.road.a.1.embank1.slope.right");
    let width_at = |y: f64| ring.iter().filter(|p| (p[1] - y).abs() < 0.01).map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max) - 203.6;
    assert!(near(width_at(600.0), 0.0), "{}", width_at(600.0));
    assert!(near(width_at(590.0), 3.0), "{}", width_at(590.0));
    assert!(near(width_at(500.0), 6.0));
    // Hatch lines cross each band.
    assert!(names(&v).iter().any(|n| n == "gen.road.a.1.embank1.hatch.left") && names(&v).iter().any(|n| n == "gen.road.a.1.embank1.hatch.right"));
    // The road itself is still drawn.
    assert!(names(&v).iter().any(|n| n.starts_with("gen.road.a.1.surface.")));
}

#[test]
fn one_side_only_and_the_whole_line_by_default() {
    let (items, spec) = road(json!([{"height": 1, "slope": 3, "sides": "left", "ramp": 0}]));
    let v = build(&items, &spec);
    let n = names(&v);
    assert!(n.iter().any(|x| x == "gen.road.a.1.embank1.slope.left") && !n.iter().any(|x| x.contains("slope.right")), "{n:?}");
    let (lo, hi) = bbox(&pts(&v, "gen.road.a.1.embank1.slope.left"));
    assert!(near(lo[0], 193.4) && near(hi[0], 196.4) && near(lo[1], 100.0) && near(hi[1], 700.0), "the whole 600 m, a 3 m band: {lo:?} {hi:?}");
}

#[test]
fn an_embankment_belongs_to_its_own_line() {
    let (items, spec) = road(json!([{"piece": 2, "height": 2}]));
    assert!(!names(&build(&items, &spec)).iter().any(|n| n.contains("embank")));
}

#[test]
fn the_description_is_read_and_checked_in_plain_words() {
    let e = |v: Value| EmbankmentSpec::from_json(&v);
    assert_eq!(e(json!({})).unwrap().sides, Sides::Both);
    assert!(e(json!({"sides": "up"})).unwrap_err().contains("sides must be both, left or right"));
    assert!(e(json!({"from": 100, "to": 100})).unwrap_err().contains("must end after it starts"));
    let s = e(json!({"height": 500, "slope": 0.1, "from": 10, "to": 90})).unwrap();
    assert_eq!((s.height, s.slope), (30.0, 1.0));
    assert!(e(json!({"height": 3, "slope": 2, "from": 120, "to": 300})).unwrap().summary().contains("road on fill 3 m high with 2:1 slopes from 120 m to 300 m"));
    assert_eq!(e(json!({"height": 3, "slope": 2})).unwrap().reach(), 6.0);
}

#[test]
fn the_command_stores_replaces_and_removes_and_a_build_draws_it() {
    let its = vec![Item { closed: false, id: Some(1), name: "road.a".into(), subs: vec![vec![[200.0, 700.0], [200.0, 100.0]]] }];
    let input = json!({"objects": its.iter().map(|i| json!({"id": i.id, "name": i.name, "type": "path", "path": {"subpaths": [{"anchors": i.subs[0].iter().map(|p| json!({"p": p})).collect::<Vec<_>>(), "closed": false}]}})).collect::<Vec<_>>()});
    let data = json!({"segments": {"a": {"cross_section": TWO}}});
    let run = |d: &str, data: &Value| set_embankment_text(&input, &json!({"description": d, "_context": {"documentData": data.clone()}}));
    let out: Value = serde_json::from_str(&run(r#"{"height": 3, "slope": 2, "from": 100, "to": 300}"#, &data).unwrap()).unwrap();
    assert_eq!(out["documentData"]["embankments"]["a"].as_array().unwrap().len(), 1);
    assert_eq!(out["documentData"]["segments"]["a"]["cross_section"], TWO);
    let again: Value = serde_json::from_str(&run(r#"{"height": 4, "from": 103, "to": 300}"#, &out["documentData"]).unwrap()).unwrap();
    assert_eq!(again["documentData"]["embankments"]["a"].as_array().unwrap().len(), 1);
    assert_eq!(again["documentData"]["embankments"]["a"][0]["height"], 4.0);
    let more: Value = serde_json::from_str(&run(r#"{"height": 2, "from": 400, "to": 500}"#, &again["documentData"]).unwrap()).unwrap();
    assert_eq!(more["documentData"]["embankments"]["a"].as_array().unwrap().len(), 2);
    let gone: Value = serde_json::from_str(&run(r#"{"from": 400, "to": 500, "remove": true}"#, &more["documentData"]).unwrap()).unwrap();
    assert_eq!(gone["documentData"]["embankments"]["a"].as_array().unwrap().len(), 1);
    assert!(run("{nope", &data).unwrap_err().contains("not valid JSON"));
    assert!(run(r#"{"sides": "up"}"#, &data).unwrap_err().contains("sides must be"));
    let none = json!({"objects": []});
    assert!(set_embankment_text(&none, &json!({"description": "{}", "_context": {"documentData": data}})).unwrap_err().contains("one line of one road"));
    let built: Value = serde_json::from_str(&build_text(&input, &json!({"_context": {"documentData": out["documentData"].clone()}})).unwrap()).unwrap();
    assert!(names(&built).iter().any(|n| n == "gen.road.a.1.embank1.slope.left"));
    assert_eq!(built["documentData"]["embankments"]["a"][0]["height"], 3.0);
}
