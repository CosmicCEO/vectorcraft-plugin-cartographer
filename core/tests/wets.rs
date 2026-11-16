use cartographer_core::builder::{build_text, set_wet_text};
use cartographer_core::output::Out;
use cartographer_core::roads::Spec;
use cartographer_core::vcdoc::Item;
use cartographer_core::wet::{self, WetSpec};
use serde_json::{Value, json};

const TWO: &str = "lane back | lane fwd";

fn item(id: i64, name: &str, pts: &[[f64; 2]]) -> Item {
    Item { closed: false, id: Some(id), name: name.into(), subs: vec![pts.to_vec()] }
}

/// A road east along y = 400 from x = 0 to 400 (7.2 m wide: edges y 396.4 and 403.6) and a creek north through x = 200.
fn creek_scene() -> Vec<Item> {
    vec![item(1, "road.a", &[[0.0, 400.0], [400.0, 400.0]]), item(2, "water.c", &[[200.0, 700.0], [200.0, 100.0]])]
}

fn spec(waters: Value, wets: Value) -> Spec {
    Spec::from_document_data(&json!({"segments": {"a": {"cross_section": TWO}}, "waters": waters, "wets": wets}))
}

fn creek() -> Value {
    json!({"c": {"kind": "flowing", "width": 10, "bank_left": 1.5, "bank_right": 1.5, "slope": 2}})
}

fn run(items: &[Item], sp: &Spec) -> (Value, Vec<String>) {
    let mut out = Out::new();
    let notes = wet::build(items, sp, &mut out);
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
    (a - b).abs() < 0.03
}

fn has(v: &Value, n: &str) -> bool {
    names(v).iter().any(|x| x == n)
}

#[test]
fn a_bridge_spans_the_water_and_its_banks_with_an_abutment_beyond_each() {
    let sp = spec(creek(), json!({"x1": {"road": "a", "water": "c", "treatment": "bridge", "abutment": 4, "deck_height": 3.5}}));
    let (v, notes) = run(&creek_scene(), &sp);
    assert!(notes.is_empty(), "{notes:?}");
    // The creek is 10 m wide (x 195..205) with 3 m banks: the deck runs from 4 m before the top of the west bank (188) to 4 m past the east bank (212).
    let (lo, hi) = bbox(&pts(&v, "gen.wet.x1.deck.surface.0.lane"));
    assert!(near(lo[0], 188.0) && near(hi[0], 212.0) && near(lo[1], 396.4) && near(hi[1], 400.0), "{lo:?} {hi:?}");
    let (a, b) = (bbox(&pts(&v, "gen.wet.x1.abutment.1")), bbox(&pts(&v, "gen.wet.x1.abutment.2")));
    assert!(near(a.0[0], 188.0) && near(b.0[0], 212.0) && near(a.0[1], 396.4) && near(a.1[1], 403.6), "{a:?} {b:?}");
    assert!(has(&v, "gen.wet.x1.parapet.1") && has(&v, "gen.wet.x1.parapet.2"));
    // A wing wall flares out 10 m back from each corner of each abutment, 2.5 m outward: west end, north (left facing east) corner, and east end, south corner.
    let (lo, hi) = bbox(&pts(&v, "gen.wet.x1.wing.1.left"));
    assert!(near(lo[0], 178.0) && near(hi[0], 188.0) && near(lo[1], 393.9) && near(hi[1], 396.4), "{lo:?} {hi:?}");
    let (lo, hi) = bbox(&pts(&v, "gen.wet.x1.wing.2.right"));
    assert!(near(lo[0], 212.0) && near(hi[0], 222.0) && near(lo[1], 403.6) && near(hi[1], 406.1), "{lo:?} {hi:?}");
    assert!(!has(&v, "gen.wet.x1.pier.1"), "a 10 m creek needs no pier");
    // A guardrail runs from the end of the wing wall (10 m back) to 40 m back, 2.5 m off the pavement edge; rock lies beside the abutment, 6 m long, 0.5 to 5 m off the edge.
    let (lo, hi) = bbox(&pts(&v, "gen.wet.x1.rail.1.left"));
    assert!(near(lo[0], 148.0) && near(hi[0], 178.0) && near(lo[1], 393.9) && near(hi[1], 393.9), "{lo:?} {hi:?}");
    let (lo, hi) = bbox(&pts(&v, "gen.wet.x1.riprap.1.left"));
    assert!(near(lo[0], 182.0) && near(hi[0], 188.0) && near(lo[1], 391.4) && near(hi[1], 395.9), "{lo:?} {hi:?}");
    // The road's own strip is not drawn here (the road layer has it); the crossing adds the deck above.
    assert!(!names(&v).iter().any(|n| n.contains("embank") || n.contains("pipe")));
}

#[test]
fn a_bridge_over_a_slanted_creek_is_longer_along_the_road() {
    // The creek crosses the road at 30 degrees: 10 m of water is 20 m along the road.
    let c = [[200.0 - 0.866 * 300.0, 400.0 + 0.5 * 300.0], [200.0 + 0.866 * 300.0, 400.0 - 0.5 * 300.0]];
    let items = vec![item(1, "road.a", &[[0.0, 400.0], [400.0, 400.0]]), item(2, "water.c", &c)];
    let sp = spec(creek(), json!({"x1": {"road": "a", "water": "c", "treatment": "bridge", "abutment": 0}}));
    let (v, _) = run(&items, &sp);
    // water 20 m along the road (190..210) and banks 3 / sin 30 = 6 m each end: 184..216.
    let (lo, hi) = bbox(&pts(&v, "gen.wet.x1.deck.surface.0.lane"));
    assert!(near(lo[0], 184.0) && near(hi[0], 216.0), "{lo:?} {hi:?}");
}

#[test]
fn a_culvert_has_dashed_pipes_headwalls_and_the_road_on_fill() {
    let sp = spec(creek(), json!({"x1": {"road": "a", "water": "c", "treatment": "culvert", "count": 2, "size": 1.2, "fill_height": 2.5, "slope": 2}}));
    let (v, notes) = run(&creek_scene(), &sp);
    assert!(notes.is_empty(), "{notes:?}");
    // Fill 2.5 m high at 2:1 reaches 5 m. Along the road it covers the 10 m of water, 3 m of bank each side and 5 m of slope each side: 26 m, x 187..213.
    let (lo, hi) = bbox(&pts(&v, "gen.wet.x1.embank.slope.right"));
    assert!(near(lo[0], 187.0) && near(hi[0], 213.0) && near(lo[1], 403.6) && near(hi[1], 408.6), "{lo:?} {hi:?}");
    let (lo, hi) = bbox(&pts(&v, "gen.wet.x1.embank.slope.left"));
    assert!(near(lo[1], 391.4) && near(hi[1], 396.4), "{lo:?} {hi:?}");
    // Two 1.2 m pipes 0.72 m apart run along the creek (north-south) for 7.2 m of road plus 10 m of fill: y 391.4..408.6, x 198.44..201.56.
    let (lo, hi) = bbox(&pts(&v, "gen.wet.x1.pipe"));
    assert!(near(lo[0], 198.44) && near(hi[0], 201.56) && near(lo[1], 391.4) && near(hi[1], 408.6), "{lo:?} {hi:?}");
    // A headwall across the pipes at each end, 0.4 m wider than the group each side (3.12 m group): x 198.04..201.96.
    for n in ["gen.wet.x1.headwall.1", "gen.wet.x1.headwall.2"] {
        let (lo, hi) = bbox(&pts(&v, n));
        assert!(near(lo[0], 198.04) && near(hi[0], 201.96), "{n}: {lo:?} {hi:?}");
    }
    let ys: Vec<f64> = ["gen.wet.x1.headwall.1", "gen.wet.x1.headwall.2"].iter().map(|n| pts(&v, n)[0][1]).collect();
    assert!((near(ys[0], 391.4) && near(ys[1], 408.6)) || (near(ys[1], 391.4) && near(ys[0], 408.6)), "{ys:?}");
    assert!(!has(&v, "gen.wet.x1.deck.surface.0.lane"));
}

fn pond_scene() -> Vec<Item> {
    let square = [[100.0, 100.0], [300.0, 100.0], [300.0, 300.0], [100.0, 300.0], [100.0, 100.0]];
    vec![item(1, "road.a", &[[0.0, 200.0], [400.0, 200.0]]), item(2, "water.p", &square)]
}

#[test]
fn a_causeway_puts_the_road_on_fill_across_a_pond() {
    let sp = spec(json!({"p": {"kind": "still"}}), json!({"x1": {"road": "a", "water": "p", "treatment": "causeway", "fill_height": 2, "slope": 2}}));
    let (v, notes) = run(&pond_scene(), &sp);
    assert!(notes.is_empty(), "{notes:?}");
    // The road enters at x = 100 and leaves at x = 300; the fill runs 8 m beyond each (92..308) with 4 m slopes (2 m at 2:1): y 203.6..207.6 south, 192.4..196.4 north.
    let (lo, hi) = bbox(&pts(&v, "gen.wet.x1.embank.slope.right"));
    assert!(near(lo[0], 92.0) && near(hi[0], 308.0) && near(lo[1], 203.6) && near(hi[1], 207.6), "{lo:?} {hi:?}");
    let (lo, hi) = bbox(&pts(&v, "gen.wet.x1.embank.slope.left"));
    assert!(near(lo[1], 192.4) && near(hi[1], 196.4), "{lo:?} {hi:?}");
    // A bridge over the same pond spans from where the road enters (plus the bank) to where it leaves.
    let sp = spec(json!({"p": {"kind": "still", "bank": 0}}), json!({"x1": {"road": "a", "water": "p", "treatment": "bridge", "abutment": 5}}));
    let (b, _) = run(&pond_scene(), &sp);
    let (lo, hi) = bbox(&pts(&b, "gen.wet.x1.deck.surface.0.lane"));
    assert!(near(lo[0], 95.0) && near(hi[0], 305.0), "{lo:?} {hi:?}");
    // 200 m of water needs 4 piers (200 / 45 rounded up is 5 spans): at x 140, 180, 220, 260, each 3 m long and 0.6 x 7.2 = 4.32 m across.
    for (i, x) in [(1, 140.0), (2, 180.0), (3, 220.0), (4, 260.0)] {
        let (lo, hi) = bbox(&pts(&b, &format!("gen.wet.x1.pier.{i}")));
        assert!(near(lo[0], x - 1.5) && near(hi[0], x + 1.5) && near(lo[1], 197.84) && near(hi[1], 202.16), "pier {i}: {lo:?} {hi:?}");
    }
    assert!(!has(&b, "gen.wet.x1.pier.5"));
}

#[test]
fn a_shore_crossing_runs_from_the_shore_to_the_end_of_the_road() {
    let items = vec![item(1, "road.a", &[[0.0, 200.0], [400.0, 200.0]]), item(2, "water.s", &[[250.0, 0.0], [250.0, 400.0]])];
    let sp = spec(json!({"s": {"kind": "shore", "side": "left", "bank": 0}}), json!({"x1": {"road": "a", "water": "s", "treatment": "causeway", "fill_height": 1, "slope": 2}}));
    let (v, notes) = run(&items, &sp);
    assert!(notes.is_empty(), "{notes:?}");
    // The road reaches the shore at x = 250 and runs on to its end at 400: the fill runs from 242 to the end of the road.
    let (lo, hi) = bbox(&pts(&v, "gen.wet.x1.embank.slope.right"));
    assert!(near(lo[0], 242.0) && near(hi[0], 400.0), "{lo:?} {hi:?}");
}

#[test]
fn problems_are_notes_in_plain_words() {
    let pond = spec(json!({"p": {"kind": "still"}}), json!({"x1": {"road": "a", "water": "p", "treatment": "culvert"}}));
    let (_, n) = run(&pond_scene(), &pond);
    assert!(n.iter().any(|x| x.contains("a culvert is for a creek or river")), "{n:?}");
    let miss = spec(creek(), json!({"x1": {"road": "a", "water": "c", "treatment": "bridge"}}));
    let away = vec![item(1, "road.a", &[[0.0, 800.0], [400.0, 800.0]]), item(2, "water.c", &[[200.0, 700.0], [200.0, 100.0]])];
    assert!(run(&away, &miss).1.iter().any(|x| x.contains("does not cross the water")));
    let along = vec![item(1, "road.a", &[[190.0, 700.0], [210.0, 100.0]]), item(2, "water.c", &[[200.0, 700.0], [200.0, 100.0]])];
    assert!(run(&along, &miss).1.iter().any(|x| x.contains("nearly along the water")));
    let no_water = vec![item(1, "road.a", &[[0.0, 400.0], [400.0, 400.0]])];
    assert!(run(&no_water, &miss).1.iter().any(|x| x.contains("is not in the document")));
    let one_pass = spec(json!({"p": {"kind": "still"}}), json!({"x1": {"road": "a", "water": "p", "treatment": "bridge"}}));
    let ends_inside = vec![item(1, "road.a", &[[0.0, 200.0], [200.0, 200.0]]), pond_scene().remove(1)];
    assert!(run(&ends_inside, &one_pass).1.iter().any(|x| x.contains("needs to cross the shape twice")));
}

#[test]
fn the_description_is_read_and_checked() {
    let w = |v: Value| WetSpec::from_json(&v);
    assert!(w(json!({"water": "c", "treatment": "bridge"})).unwrap_err().contains("ids of a road and a water"));
    assert!(w(json!({"road": "a", "water": "c"})).unwrap_err().contains("needs \"treatment\""));
    assert!(w(json!({"road": "a", "water": "c", "treatment": "ferry"})).unwrap_err().contains("treatment must be"));
    assert!(w(json!({"road": "a", "water": "c", "treatment": "culvert", "shape": "oval"})).unwrap_err().contains("shape must be pipe or box"));
    let c = w(json!({"road": "a", "water": "c", "treatment": "pipe", "count": 99, "shape": "box"})).unwrap();
    assert_eq!((c.count, c.size, c.fill_height), (6, 2.4, 2.5));
    let causeway = w(json!({"road": "a", "water": "c", "treatment": "embankment"})).unwrap();
    assert_eq!((causeway.fill_height, causeway.slope), (2.0, 2.0));
    assert!(c.summary().contains("culvert of 6 boxes 2.4 m wide, road on 2.5 m of fill at 2:1"));
    assert!(WetSpec::from_json(&json!({"road": "a", "water": "c", "treatment": "bridge", "deck_height": 3.5})).unwrap().summary().contains("3.5 m clearance above the water"));
}

fn input_of(items: &[Item]) -> Value {
    json!({"objects": items.iter().map(|i| json!({"id": i.id, "name": i.name, "type": "path", "path": {"subpaths": [{"anchors": i.subs[0].iter().map(|p| json!({"p": p})).collect::<Vec<_>>(), "closed": false}]}})).collect::<Vec<_>>()})
}

#[test]
fn the_command_stores_one_crossing_per_road_and_water_pair() {
    let input = input_of(&creek_scene());
    let data = json!({"segments": {"a": {"cross_section": TWO}}, "waters": creek()});
    let run = |d: &str, data: &Value| set_wet_text(&input, &json!({"description": d, "_context": {"documentData": data.clone()}}));
    let out: Value = serde_json::from_str(&run(r#"{"treatment": "culvert", "count": 3}"#, &data).unwrap()).unwrap();
    assert_eq!(out["documentData"]["wets"]["x1"]["road"], "a");
    assert_eq!(out["documentData"]["wets"]["x1"]["water"], "c");
    assert_eq!(out["documentData"]["wets"]["x1"]["count"], 3);
    assert_eq!(out["documentData"]["waters"]["c"]["width"], 10.0, "other data is kept");
    let again: Value = serde_json::from_str(&run(r#"{"treatment": "bridge"}"#, &out["documentData"]).unwrap()).unwrap();
    assert_eq!(again["documentData"]["wets"].as_object().unwrap().len(), 1, "the same pair updates the same crossing");
    assert_eq!(again["documentData"]["wets"]["x1"]["treatment"], "bridge");
    assert!(run("{nope", &data).unwrap_err().contains("not valid JSON"));
    assert!(run(r#"{"treatment": "ferry"}"#, &data).unwrap_err().contains("treatment must be"));
    let only_road = input_of(&creek_scene()[..1]);
    assert!(set_wet_text(&only_road, &json!({"description": "{}", "_context": {"documentData": data.clone()}})).unwrap_err().contains("one line of one road and one water line"));
    // A build draws it and keeps the data.
    let built: Value = serde_json::from_str(&build_text(&input, &json!({"_context": {"documentData": again["documentData"].clone(), "artboard": [0, 0, 400, 800]}})).unwrap()).unwrap();
    assert!(names(&built).iter().any(|n| n.starts_with("gen.wet.x1.deck.")), "{:?}", names(&built).iter().filter(|n| n.contains("wet")).collect::<Vec<_>>());
    assert_eq!(built["documentData"]["wets"]["x1"]["treatment"], "bridge");
}
