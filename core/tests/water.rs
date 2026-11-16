use cartographer_core::builder::{build_text, set_water_text};
use cartographer_core::output::Out;
use cartographer_core::roads::Spec;
use cartographer_core::vcdoc::Item;
use cartographer_core::water::{Flow, Kind, WaterSpec, build, offset_ring, water_id};
use serde_json::{Value, json};

fn item(id: i64, name: &str, pts: &[[f64; 2]]) -> Item {
    Item { closed: false, id: Some(id), name: name.into(), subs: vec![pts.to_vec()] }
}

fn spec(waters: Value) -> Spec {
    Spec::from_document_data(&json!({"waters": waters}))
}

fn run(items: &[Item], sp: &Spec, artboard: Option<[f64; 4]>) -> (Value, Vec<String>) {
    let mut out = Out::new();
    let notes = build(items, sp, artboard, &mut out);
    (serde_json::from_str(&out.finish()).unwrap(), notes)
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

fn has(v: &Value, n: &str) -> bool {
    names(v).iter().any(|x| x == n)
}

/// A creek running north through (200, 700) to (200, 100) (document y runs down the page).
fn creek() -> Vec<Item> {
    vec![item(1, "water.c", &[[200.0, 700.0], [200.0, 100.0]])]
}

#[test]
fn a_creek_is_a_strip_with_banks_either_side_and_arrows_with_the_flow() {
    let sp = spec(json!({"c": {"kind": "flowing", "width": 10, "bank_left": 1.5, "bank_right": 1.5, "slope": 2, "flow": "fwd"}}));
    let (v, notes) = run(&creek(), &sp, None);
    assert!(notes.is_empty(), "{notes:?}");
    // Water 10 m wide: x 195..205. Banks 1.5 m high at 2:1 are 3 m wide each side: x 192..195 (west, the line's left) and 205..208.
    let (lo, hi) = bbox(&pts(&v, "gen.water.c.water"));
    assert!(near(lo[0], 195.0) && near(hi[0], 205.0) && near(lo[1], 100.0) && near(hi[1], 700.0), "{lo:?} {hi:?}");
    let (lo, hi) = bbox(&pts(&v, "gen.water.c.bank.left"));
    assert!(near(lo[0], 192.0) && near(hi[0], 195.0), "{lo:?} {hi:?}");
    let (lo, hi) = bbox(&pts(&v, "gen.water.c.bank.right"));
    assert!(near(lo[0], 205.0) && near(hi[0], 208.0), "{lo:?} {hi:?}");
    // The banks are drawn before (under) the water.
    let n = names(&v);
    let at = |s: &str| n.iter().position(|x| x == s).unwrap();
    assert!(at("gen.water.c.bank.left") < at("gen.water.c.water") && at("gen.water.c.bank.right") < at("gen.water.c.water"));
    // The flow arrows point north (document y gets smaller toward the tip) with the line.
    let a = pts(&v, "gen.water.c.flow");
    assert!(a.len() >= 3 && a.len() % 3 == 0 && a[1][1] < a[0][1] && a[1][1] < a[2][1], "{a:?}");
    // Against the line they point south; with no flow there are none.
    let (back, _) = run(&creek(), &spec(json!({"c": {"kind": "flowing", "flow": "back"}})), None);
    let b = pts(&back, "gen.water.c.flow");
    assert!(b[1][1] > b[0][1] && b[1][1] > b[2][1]);
    let (none, _) = run(&creek(), &spec(json!({"c": {"kind": "flowing", "flow": "none"}})), None);
    assert!(!has(&none, "gen.water.c.flow"));
}

#[test]
fn banks_can_differ_and_vanish() {
    let sp = spec(json!({"c": {"kind": "creek", "width": 6, "bank_left": 3, "bank_right": 0, "slope": 3}}));
    let (v, _) = run(&creek(), &sp, None);
    let (lo, hi) = bbox(&pts(&v, "gen.water.c.bank.left"));
    assert!(near(lo[0], 188.0) && near(hi[0], 197.0), "3 m high at 3:1 is 9 m wide: {lo:?} {hi:?}");
    assert!(!has(&v, "gen.water.c.bank.right"), "a bank of height 0 is not drawn");
}

#[test]
fn a_lake_is_a_filled_shape_with_a_bank_band_all_round() {
    let square = [[100.0, 100.0], [300.0, 100.0], [300.0, 300.0], [100.0, 300.0], [100.0, 100.0]];
    let (v, notes) = run(&[item(1, "water.l", &square)], &spec(json!({"l": {"kind": "still", "bank_left": 1.5, "bank_right": 1.5, "slope": 2}})), None);
    assert!(notes.is_empty(), "{notes:?}");
    let (lo, hi) = bbox(&pts(&v, "gen.water.l.water"));
    assert!(near(lo[0], 100.0) && near(hi[0], 300.0) && near(lo[1], 100.0) && near(hi[1], 300.0), "{lo:?} {hi:?}");
    // A 3 m band all round, mitred at the corners: 97..303.
    let (lo, hi) = bbox(&pts(&v, "gen.water.l.bank"));
    assert!(near(lo[0], 97.0) && near(hi[0], 303.0) && near(lo[1], 97.0) && near(hi[1], 303.0), "{lo:?} {hi:?}");
    // With no stored description a closed shape is still water and an open line a creek.
    let (d, _) = run(&[item(1, "water.l", &square)], &Spec::from_document_data(&json!({})), None);
    assert!(has(&d, "gen.water.l.water") && has(&d, "gen.water.l.bank"));
    let (d, _) = run(&creek(), &Spec::from_document_data(&json!({})), None);
    assert!(has(&d, "gen.water.c.flow"));
}

#[test]
fn a_shore_fills_the_water_side_to_beyond_the_page_and_puts_the_bank_on_land() {
    // A shore drawn east along y = 200 with the water on its right (south, larger document y). The page is 400 x 300.
    let line = [item(1, "water.s", &[[100.0, 200.0], [300.0, 200.0]])];
    let sp = spec(json!({"s": {"kind": "shore", "side": "right", "bank_left": 1.5, "slope": 2}}));
    let (v, _) = run(&line, &sp, Some([0.0, 0.0, 400.0, 300.0]));
    // far = (400 + 300) x 3 + 500 = 2600: the water runs 2600 m along the line each way and 2600 m out.
    let (lo, hi) = bbox(&pts(&v, "gen.water.s.water"));
    assert!(near(lo[0], -2500.0) && near(hi[0], 2900.0) && near(lo[1], 200.0) && near(hi[1], 2800.0), "{lo:?} {hi:?}");
    // The bank is on the land side (north, smaller y), 3 m wide.
    let (lo, hi) = bbox(&pts(&v, "gen.water.s.bank"));
    assert!(near(lo[1], 197.0) && near(hi[1], 200.0) && near(lo[0], 100.0) && near(hi[0], 300.0), "{lo:?} {hi:?}");
    // Water on the left is on the north side instead.
    let (l, _) = run(&line, &spec(json!({"s": {"kind": "shore", "side": "left"}})), Some([0.0, 0.0, 400.0, 300.0]));
    let (lo, hi) = bbox(&pts(&l, "gen.water.s.water"));
    assert!(near(lo[1], -2400.0) && near(hi[1], 200.0), "{lo:?} {hi:?}");
}

#[test]
fn kinds_and_words_are_read_and_checked() {
    let k = |w: &str| WaterSpec::from_json(&json!({"kind": w})).map(|s| s.kind);
    for (w, want) in [("pond", Kind::Still), ("Lake", Kind::Still), ("creek", Kind::Flowing), ("river", Kind::Flowing), ("coast", Kind::Shore), ("shoreline", Kind::Shore)] {
        assert_eq!(k(w).unwrap(), want, "{w}");
    }
    assert!(k("ocean-of-lava").unwrap_err().contains("kind must be still"));
    assert!(WaterSpec::from_json(&json!({})).unwrap_err().contains("needs \"kind\""));
    assert!(WaterSpec::from_json(&json!({"kind": "creek", "flow": "sideways"})).unwrap_err().contains("flow must be"));
    assert!(WaterSpec::from_json(&json!({"kind": "shore", "side": "up"})).unwrap_err().contains("side must be left or right"));
    let s = WaterSpec::from_json(&json!({"kind": "creek", "bank": 2, "width": 9999, "slope": 0.1, "speed": 99})).unwrap();
    assert_eq!((s.bank_left, s.bank_right, s.width, s.slope, s.speed, s.flow), (2.0, 2.0, 300.0, 1.0, Some(15.0), Flow::Fwd));
    assert!(s.summary().contains("creek or river 300 m wide, flowing with the line at 15 m/s"));
    assert!(WaterSpec::from_json(&json!({"kind": "creek"})).unwrap().check_shape(true).unwrap_err().contains("open line"));
    assert!(WaterSpec::from_json(&json!({"kind": "pond"})).unwrap().check_shape(false).unwrap_err().contains("closed shape"));
    assert_eq!(water_id("water.cedar.2"), Some("cedar".into()));
    assert_eq!(water_id("water"), Some("main".into()));
    assert_eq!(water_id("road.a"), None);
}

#[test]
fn a_lake_that_is_not_closed_is_a_note_not_a_failure() {
    let (v, notes) = run(&creek(), &spec(json!({"c": {"kind": "still"}})), None);
    assert!(notes.iter().any(|n| n.contains("needs a closed shape")), "{notes:?}");
    assert!(names(&v).is_empty());
}

#[test]
fn rings_move_outward_whichever_way_they_are_wound() {
    let cw = [[0.0, 0.0], [0.0, 10.0], [10.0, 10.0], [10.0, 0.0]];
    let ccw: Vec<[f64; 2]> = cw.iter().rev().copied().collect();
    for ring in [cw.to_vec(), ccw] {
        let out = offset_ring(&ring, 2.0);
        let (lo, hi) = bbox(&out);
        assert!(near(lo[0], -2.0) && near(hi[0], 12.0) && near(lo[1], -2.0) && near(hi[1], 12.0), "{lo:?} {hi:?}");
        let inward = bbox(&offset_ring(&ring, -2.0));
        assert!(near(inward.0[0], 2.0) && near(inward.1[0], 8.0));
    }
}

fn input_of(items: &[Item]) -> Value {
    json!({"objects": items.iter().map(|i| json!({"id": i.id, "name": i.name, "type": "path", "path": {"subpaths": [{"anchors": i.subs[0].iter().map(|p| json!({"p": p})).collect::<Vec<_>>(), "closed": false}]}})).collect::<Vec<_>>()})
}

#[test]
fn the_command_names_the_line_stores_the_description_and_builds_it() {
    let items = vec![item(7, "", &[[200.0, 700.0], [200.0, 100.0]]), item(8, "something else", &[[0.0, 0.0], [5.0, 5.0]])];
    let mut input = input_of(&items);
    input["objects"] = json!([input["objects"][0].clone()]);
    let params = |d: &str, data: Value| json!({"description": d, "_context": {"documentData": data, "artboard": [0, 0, 400, 800]}});
    let out: Value = serde_json::from_str(&set_water_text(&input, &params(r#"{"width": 12, "name": "Cedar Creek", "bank": 2}"#, json!({}))).unwrap()).unwrap();
    // The kind was worked out from the shape (an open line is a creek), the line renamed, the description stored.
    assert_eq!(out["objects"][0]["name"], "water.w1");
    assert_eq!(out["documentData"]["waters"]["w1"]["kind"], "flowing");
    assert_eq!(out["documentData"]["waters"]["w1"]["name"], "Cedar Creek");
    assert_eq!(out["documentData"]["waters"]["w1"]["width"], 12.0);
    // A second line gets the next id.
    let two = {
        let mut i = input_of(&[item(9, "", &[[0.0, 0.0], [100.0, 100.0]])]);
        i["objects"][0]["id"] = json!(9);
        i
    };
    let second: Value = serde_json::from_str(&set_water_text(&two, &params(r#"{"kind": "shore"}"#, out["documentData"].clone())).unwrap()).unwrap();
    assert_eq!(second["objects"][0]["name"], "water.w2");
    // Building draws it and keeps the data and puts the line on top as a teal outline without a fill.
    let named = input_of(&[item(7, "water.w1", &[[200.0, 700.0], [200.0, 100.0]])]);
    let built: Value = serde_json::from_str(&build_text(&named, &json!({"_context": {"documentData": out["documentData"].clone(), "artboard": [0, 0, 400, 800]}})).unwrap()).unwrap();
    assert!(names(&built).iter().any(|n| n == "gen.water.w1.water"));
    assert_eq!(built["documentData"]["waters"]["w1"]["name"], "Cedar Creek");
    let kept = built["objects"].as_array().unwrap().iter().find(|o| o["id"] == 7).unwrap();
    assert_eq!(kept["fills"], json!([]));
    // Plain-words problems.
    assert!(set_water_text(&json!({"objects": []}), &params("{}", json!({}))).unwrap_err().contains("Select the line"));
    assert!(set_water_text(&input, &params(r#"{"kind": "pond"}"#, json!({}))).unwrap_err().contains("closed shape"));
    assert!(set_water_text(&input, &params("{nope", json!({}))).unwrap_err().contains("not valid JSON"));
    let road = input_of(&[item(5, "road.a", &[[0.0, 0.0], [5.0, 5.0]])]);
    assert!(set_water_text(&road, &params("{}", json!({}))).unwrap_err().contains("is a road line"));
}

fn x_range_at(v: &Value, name: &str, x: f64) -> (f64, f64) {
    let p: Vec<[f64; 2]> = pts(v, name).into_iter().filter(|q| (q[0] - x).abs() < 0.01).collect();
    assert!(!p.is_empty(), "no point of {name} at x = {x}");
    (p.iter().map(|q| q[1]).fold(f64::INFINITY, f64::min), p.iter().map(|q| q[1]).fold(f64::NEG_INFINITY, f64::max))
}

#[test]
fn two_creeks_joined_end_to_end_conform_at_the_joint() {
    // A is 10 m wide from x 0 to 100, B 20 m wide from x 100 to 200, both along y = 200. At the joint both are 15 m wide (192.5..207.5),
    // easing to their own width over 40 m (half the line is 50 m, capped at 40): A is 10 m wide again from x 60 back, B 20 m from x 140 on.
    let items = [item(1, "water.a", &[[0.0, 200.0], [100.0, 200.0]]), item(2, "water.b", &[[100.0, 200.0], [200.0, 200.0]])];
    let sp = spec(json!({"a": {"kind": "flowing", "width": 10, "bank": 0}, "b": {"kind": "flowing", "width": 20, "bank": 0}}));
    let (v, _) = run(&items, &sp, None);
    let (lo, hi) = x_range_at(&v, "gen.water.a.water", 100.0);
    assert!(near(lo, 192.5) && near(hi, 207.5), "{lo} {hi}");
    let (lo, hi) = x_range_at(&v, "gen.water.b.water", 100.0);
    assert!(near(lo, 192.5) && near(hi, 207.5), "{lo} {hi}");
    let (lo, hi) = x_range_at(&v, "gen.water.a.water", 0.0);
    assert!(near(lo, 195.0) && near(hi, 205.0), "{lo} {hi}");
    let (lo, hi) = x_range_at(&v, "gen.water.b.water", 200.0);
    assert!(near(lo, 190.0) && near(hi, 210.0), "{lo} {hi}");
    // Halfway through the ramp (20 m from the joint) A is 12.5 m wide.
    let (lo, hi) = x_range_at(&v, "gen.water.a.water", 80.0);
    assert!(near(lo, 193.75) && near(hi, 206.25), "{lo} {hi}");
}

#[test]
fn lines_drawn_against_each_other_conform_with_their_banks_swapped_and_a_tributary_is_left_alone() {
    // B is drawn from its far end toward the joint: its right is A's left. A has a 3 m left bank and no right bank, B the reverse: at the joint the joint keeps 3 m on that side.
    let items = [item(1, "water.a", &[[0.0, 200.0], [100.0, 200.0]]), item(2, "water.b", &[[200.0, 200.0], [100.0, 200.0]])];
    let sp = spec(json!({"a": {"kind": "flowing", "width": 10, "bank_left": 3, "bank_right": 0, "slope": 1}, "b": {"kind": "flowing", "width": 10, "bank_left": 0, "bank_right": 3, "slope": 1}}));
    let (v, _) = run(&items, &sp, None);
    // B's right bank is on A's left once the sides swap, so A's left bank (3 m) averages with 3 m and stays 3 m: y 192..195 beside the 10 m water.
    // (Without the swap it would average with B's 0 m left bank and be 1.5 m.)
    let (lo, hi) = x_range_at(&v, "gen.water.a.bank.left", 100.0);
    assert!(near(lo, 192.0) && near(hi, 195.0), "{lo} {hi}");
    // A tributary ending in the middle of a river is not changed: width and banks as described.
    let t = [item(1, "water.r", &[[0.0, 200.0], [200.0, 200.0]]), item(2, "water.t", &[[100.0, 100.0], [100.0, 200.0]])];
    let sp = spec(json!({"r": {"kind": "flowing", "width": 20, "bank": 1}, "t": {"kind": "flowing", "width": 6, "bank": 1}}));
    let (v, _) = run(&t, &sp, None);
    let (lo, hi) = bbox(&pts(&v, "gen.water.t.water"));
    assert!(near(lo[0], 97.0) && near(hi[0], 103.0), "{lo:?} {hi:?}");
    // All banks are drawn before any water, so the river's water is never covered by the tributary's bank.
    let n = names(&v);
    let last_bank = n.iter().rposition(|x| x.contains(".bank")).unwrap();
    let first_water = n.iter().position(|x| x.ends_with(".water")).unwrap();
    assert!(last_bank < first_water, "{n:?}");
}
