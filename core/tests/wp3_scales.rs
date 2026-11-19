use cartographer_core::builder::build_text;
use cartographer_core::output::Out;
use cartographer_core::storage::Style;
use cartographer_core::style::StyleKit;
use serde_json::{Value, json};

fn line(id: i64, name: &str, pts: &[[f64; 2]]) -> Value {
    json!({"id": id, "name": name, "path": {"subpaths": [{"anchors": pts.iter().map(|p| json!({"p": p})).collect::<Vec<_>>(), "closed": false}]}})
}

fn objects(text: &str) -> Vec<Value> {
    serde_json::from_str::<Value>(text).unwrap()["objects"].as_array().unwrap().clone()
}

fn width_of(objs: &[Value], name: &str) -> f64 {
    objs.iter().find(|o| o["name"] == name).unwrap_or_else(|| panic!("no {name}"))["strokes"][0]["width"].as_f64().unwrap()
}

#[test]
fn one_marking_keeps_one_width_across_the_pieces_of_a_road() {
    let mut out = Out::new();
    out.set_style(StyleKit::new(Style::HandDrawn, 3, 0.4));
    for piece in 1..=4 {
        out.path(&format!("gen.road.a.{piece}.line.1"), &[vec![[0.0, f64::from(piece)], [10.0, f64::from(piece)]]], |p| p, [0.2; 3], 0.12, false);
    }
    let objs = objects(&out.finish());
    let w: Vec<f64> = (1..=4).map(|k| width_of(&objs, &format!("gen.road.a.{k}.line.1"))).collect();
    assert!(w.windows(2).all(|p| p[0] == p[1]), "{w:?}");
}

#[test]
fn crosswalk_bars_arrows_and_pipes_keep_their_true_width_at_a_small_scale() {
    let mut out = Out::new();
    out.set_min_stroke(0.12 * 10000.0 / 1000.0);
    for name in ["gen.junction.j1.crosswalk.road", "gen.crossing.x1.crosswalk.n", "gen.road.a.1.arrow", "gen.crossing.x1.arrows", "gen.wet.w1.pipe"] {
        out.path(name, &[vec![[0.0, 0.0], [5.0, 0.0]]], |p| p, [1.0; 3], 0.5, false);
    }
    out.path("gen.road.a.1.line.0", &[vec![[0.0, 0.0], [5.0, 0.0]]], |p| p, [0.2; 3], 0.15, false);
    let objs = objects(&out.finish());
    for name in ["gen.junction.j1.crosswalk.road", "gen.crossing.x1.crosswalk.n", "gen.road.a.1.arrow", "gen.crossing.x1.arrows", "gen.wet.w1.pipe"] {
        assert_eq!(width_of(&objs, name), 0.5, "{name}");
    }
    assert_eq!(width_of(&objs, "gen.road.a.1.line.0"), 1.2, "lines still get the minimum");
}

#[test]
fn a_narrow_creek_gets_the_band_minimum_at_a_small_scale() {
    let input = json!({"objects": [line(1, "water.c", &[[0.0, 100.0], [600.0, 100.0]])]});
    let build = |scale: &str| {
        let p = json!({"_context": {"documentData": {"schema": 1, "waters": {"c": {"kind": "flowing", "width": 2}}, "settings": {"target_print_scale": scale}}, "artboard": [0, 0, 800, 800]}});
        objects(&build_text(&input, &p).unwrap())
    };
    let span = |objs: &[Value]| {
        let ys: Vec<f64> = objs.iter().filter(|o| o["name"].as_str().is_some_and(|n| n.starts_with("gen.water.c") && n.contains("surface") || n.ends_with(".water")))
            .flat_map(|o| o["path"]["subpaths"].as_array().unwrap().iter().flat_map(|s| s["anchors"].as_array().unwrap().iter().map(|a| a["p"][1].as_f64().unwrap())).collect::<Vec<_>>()).collect();
        assert!(!ys.is_empty(), "no water surface among {:?}", objs.iter().map(|o| o["name"].clone()).collect::<Vec<_>>());
        ys.iter().cloned().fold(f64::MIN, f64::max) - ys.iter().cloned().fold(f64::MAX, f64::min)
    };
    let (plain, small) = (span(&build("true")), span(&build("1:24000")));
    assert!((plain - 2.0).abs() < 0.05, "plain creek is 2 m: {plain}");
    assert!((small - 0.3 * 24.0).abs() < 0.05, "at 1:24000 the creek is at least 7.2 m: {small}");
}

#[test]
fn a_bridge_spans_the_widened_creek() {
    let input = json!({"objects": [line(1, "road.a", &[[0.0, 100.0], [600.0, 100.0]]), line(2, "water.c", &[[300.0, 0.0], [300.0, 400.0]])]});
    let span = |scale: &str| {
        let p = json!({"_context": {"documentData": {"schema": 1, "segments": {"a": {"cross_section": "lane back | lane fwd"}}, "waters": {"c": {"kind": "flowing", "width": 2}},
            "wets": {"x1": {"road": "a", "water": "c", "treatment": "bridge"}}, "settings": {"target_print_scale": scale}}, "artboard": [0, 0, 800, 800]}});
        let objs = objects(&build_text(&input, &p).unwrap());
        let deck = objs.iter().find(|o| o["name"].as_str().is_some_and(|n| n.starts_with("gen.wet.") && n.contains(".deck.surface."))).expect("a deck");
        let xs: Vec<f64> = deck["path"]["subpaths"][0]["anchors"].as_array().unwrap().iter().map(|a| a["p"][0].as_f64().unwrap()).collect();
        xs.iter().cloned().fold(f64::MIN, f64::max) - xs.iter().cloned().fold(f64::MAX, f64::min)
    };
    let (plain, small) = (span("true"), span("1:24000"));
    assert!(small - plain >= 7.2 - 2.0 - 0.05, "the deck grows with the creek it spans: {plain} m at true, {small} m at 1:24000");
}

#[test]
fn a_road_named_like_a_symbol_keeps_the_minimum_and_pitch_marks_do_not() {
    let mut out = Out::new();
    out.set_min_stroke(1.2);
    for name in ["gen.road.arrowhead.1.line.0", "gen.road.pipeline.1.line.0", "gen.road.crosswalk-ln.1.line.0"] {
        out.path(name, &[vec![[0.0, 0.0], [5.0, 0.0]]], |p| p, [0.2; 3], 0.15, false);
    }
    for name in ["gen.crossing.x1.ties", "gen.crossing.x1.gore.chevrons"] {
        out.path(name, &[vec![[0.0, 0.0], [5.0, 0.0]]], |p| p, [0.2; 3], 0.25, false);
    }
    let objs = objects(&out.finish());
    for name in ["gen.road.arrowhead.1.line.0", "gen.road.pipeline.1.line.0", "gen.road.crosswalk-ln.1.line.0"] {
        assert_eq!(width_of(&objs, name), 1.2, "{name}");
    }
    for name in ["gen.crossing.x1.ties", "gen.crossing.x1.gore.chevrons"] {
        assert_eq!(width_of(&objs, name), 0.25, "{name}");
    }
}
