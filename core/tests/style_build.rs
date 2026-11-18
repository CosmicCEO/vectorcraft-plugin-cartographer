use cartographer_core::builder::build_text;
use cartographer_core::output::Out;
use cartographer_core::storage::Style;
use cartographer_core::style::StyleKit;
use serde_json::{Value, json};

const TWO: &str = "lane back | lane fwd";

fn line(id: i64, name: &str, pts: &[[f64; 2]]) -> Value {
    json!({"id": id, "name": name, "path": {"subpaths": [{"anchors": pts.iter().map(|p| json!({"p": p})).collect::<Vec<_>>(), "closed": false}]}})
}

fn road_input() -> Value {
    json!({"objects": [line(1, "road.a", &[[0.0, 0.0], [400.0, 0.0]])]})
}

fn params(settings: Value) -> Value {
    json!({"_context": {"documentData": {"schema": 1, "segments": {"a": {"cross_section": TWO}}, "tracks": {"a": {"tracks": 1}}, "settings": settings}, "artboard": [0, 0, 600, 800]}})
}

fn build(input: &Value, settings: Value) -> String {
    build_text(input, &params(settings)).unwrap()
}

fn objects(text: &str) -> Vec<Value> {
    serde_json::from_str::<Value>(text).unwrap()["objects"].as_array().unwrap().clone()
}

/// What is drawn, without the stored settings record (which of course differs).
fn drawn(text: &str) -> Value {
    Value::Array(objects(text))
}

fn generated(text: &str, prefix: &str) -> Vec<Value> {
    objects(text).into_iter().filter(|o| o["name"].as_str().is_some_and(|n| n.starts_with(prefix))).collect()
}

fn points(o: &Value) -> Vec<[f64; 2]> {
    o["path"]["subpaths"].as_array().unwrap().iter().flat_map(|s| s["anchors"].as_array().unwrap().iter().map(|a| [a["p"][0].as_f64().unwrap(), a["p"][1].as_f64().unwrap()])).collect()
}

fn colours(o: &Value) -> Vec<[f64; 3]> {
    let c = |v: &Value| [v["r"].as_f64().unwrap(), v["g"].as_f64().unwrap(), v["b"].as_f64().unwrap()];
    let mut out = Vec::new();
    for f in o["fills"].as_array().into_iter().flatten() {
        out.push(c(&f["color"]));
    }
    for s in o["strokes"].as_array().into_iter().flatten() {
        out.push(c(&s["paint"]["color"]));
    }
    out
}

#[test]
fn medium_contrast_output_is_the_same_as_no_settings() {
    let none = drawn(&build(&road_input(), json!({})));
    assert_eq!(drawn(&build(&road_input(), json!({"style": "medium-contrast"}))), none);
    assert_eq!(drawn(&build(&road_input(), json!({"style": "medium-contrast", "seed": 9, "target_print_scale": "true"}))), none);
}

#[test]
fn hand_drawn_moves_the_road_and_keeps_the_line() {
    let plain = build(&road_input(), json!({}));
    let hand = build(&road_input(), json!({"style": "hand-drawn"}));
    assert_ne!(plain, hand);
    let (a, b) = (generated(&plain, "gen.road."), generated(&hand, "gen.road."));
    assert!(!a.is_empty());
    let mut moved = 0;
    let mut worst: f64 = 0.0;
    for (x, y) in a.iter().zip(&b) {
        if points(x).len() != points(y).len() {
            continue; // a curve thinned differently
        }
        for (p, q) in points(x).iter().zip(points(y)) {
            let d = (p[0] - q[0]).abs().max((p[1] - q[1]).abs());
            worst = worst.max(d);
            if d > 0.0 {
                moved += 1;
            }
        }
    }
    assert!(moved > 0, "some coordinate moved");
    assert!(worst <= 0.15 + 1e-3, "the displacement stays within the amplitude: {worst}");
    let kept = |t: &str| objects(t).into_iter().filter(|o| o["name"].as_str().is_none_or(|n| !n.starts_with("gen."))).collect::<Vec<_>>();
    assert_eq!(kept(&plain), kept(&hand), "the line you drew is not changed by the style");
}

#[test]
fn hand_drawn_is_repeatable_and_seeded() {
    let one = build(&road_input(), json!({"style": "hand-drawn", "seed": 1}));
    assert_eq!(one, build(&road_input(), json!({"style": "hand-drawn", "seed": 1})));
    assert_ne!(one, build(&road_input(), json!({"style": "hand-drawn", "seed": 2})));
}

#[test]
fn two_objects_that_share_a_point_get_the_same_displaced_point() {
    let mut out = Out::new();
    out.set_style(StyleKit::new(Style::HandDrawn, 5, 0.5));
    let a = vec![[0.0, 0.0], [10.0, 0.0], [10.0, 5.0], [0.0, 5.0]];
    let b = vec![[10.0, 5.0], [20.0, 5.0], [20.0, 9.0], [10.0, 9.0]];
    out.fills("gen.road.a.1.surface", &[a], |p| p, [0.5, 0.5, 0.5]);
    out.fills("gen.road.b.1.surface", &[b], |p| p, [0.5, 0.5, 0.5]);
    let objs = objects(&out.finish());
    let (pa, pb) = (points(&objs[0]), points(&objs[1]));
    assert_eq!(pa[2], pb[0], "the shared corner (10, 5) lands on the same point in both");
    assert_ne!(pa[2], [10.0, 5.0], "and it did move");
}

#[test]
fn a_track_is_never_displaced() {
    let input = json!({"objects": [line(1, "track.a", &[[0.0, 0.0], [300.0, 0.0]])]});
    let plain = build(&input, json!({}));
    let hand = build(&input, json!({"style": "hand-drawn", "seed": 3}));
    let (a, b) = (generated(&plain, "gen.track."), generated(&hand, "gen.track."));
    assert!(!a.is_empty());
    assert_eq!(a.iter().map(points).collect::<Vec<_>>(), b.iter().map(points).collect::<Vec<_>>(), "rails, ties and roadbed keep their exact geometry");
    let bw = build(&input, json!({"style": "black-white"}));
    for o in generated(&bw, "gen.track.") {
        assert!(colours(&o).iter().all(|c| c[0] == c[1] && c[1] == c[2]), "{} is grey", o["name"]);
    }
}

#[test]
fn colours_follow_the_style() {
    let bw = build(&road_input(), json!({"style": "black-white"}));
    let roads = generated(&bw, "gen.road.");
    assert!(!roads.is_empty());
    for o in &roads {
        assert!(colours(o).iter().all(|c| c[0] == c[1] && c[1] == c[2]), "{}: {:?}", o["name"], colours(o));
    }
    let plain = build(&road_input(), json!({}));
    assert!(generated(&plain, "gen.road.").iter().any(|o| colours(o).iter().any(|c| c[0] != c[2])), "the plain build has colour");
}

#[test]
fn a_bad_settings_record_builds_plain() {
    let plain = drawn(&build(&road_input(), json!({})));
    assert_eq!(drawn(&build(&road_input(), json!({"style": "nope"}))), plain);
    assert_eq!(drawn(&build(&road_input(), json!("not an object"))), plain);
}

#[test]
fn a_newer_schema_is_still_refused() {
    let mut p = params(json!({"style": "hand-drawn"}));
    p["_context"]["documentData"]["schema"] = json!(9);
    assert!(build_text(&road_input(), &p).unwrap_err().contains("newer Cartographer"));
}

#[test]
fn a_failed_rail_build_turns_the_displacement_back_on() {
    use cartographer_core::rail_builder;
    use cartographer_core::vcdoc::items_from_filter_input;
    let items = items_from_filter_input(&json!({"objects": [line(1, "track.a", &[[0.0, 0.0], [100.0, 0.0]])]}));
    let mut out = Out::new();
    out.set_style(StyleKit::new(Style::HandDrawn, 5, 0.5));
    assert!(rail_builder::build_into(&items, &json!({"tracks": {"a": {"tracks": 99}}}), &mut out).is_err());
    out.fills("gen.road.after", &[vec![[10.0, 5.0], [20.0, 5.0], [20.0, 9.0], [10.0, 9.0]]], |p| p, [0.5, 0.5, 0.5]);
    assert_ne!(points(&objects(&out.finish())[0])[0], [10.0, 5.0], "a road written after a failed rail build is still displaced");
}
