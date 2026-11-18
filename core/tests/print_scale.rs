use cartographer_core::builder::build_text;
use cartographer_core::crosssection::{Anchor, layout, parse};
use cartographer_core::library::Library;
use serde_json::{Value, json};

const NARROW: &str = "lane back 0.5 | lane fwd 0.5";

fn lib(min: f64) -> Library {
    let mut l = Library::builtin();
    l.set_min_width(min);
    l
}

fn placed(text: &str, min: f64) -> Vec<cartographer_core::crosssection::Placed> {
    let l = lib(min);
    layout(&parse(text, &l).unwrap(), &l, [0.0, 1.0], Anchor::Auto).unwrap()
}

fn road(settings: Value, cross_section: &str) -> Result<String, String> {
    let input = json!({"objects": [{"id": 1, "name": "road.a", "path": {"subpaths": [{"anchors": [{"p": [0.0, 0.0]}, {"p": [400.0, 0.0]}], "closed": false}]}}]});
    let params = json!({"_context": {"documentData": {"schema": 1, "segments": {"a": {"cross_section": cross_section}}, "settings": settings}, "artboard": [0, 0, 600, 800]}});
    build_text(&input, &params)
}

fn objects(text: &str) -> Vec<Value> {
    serde_json::from_str::<Value>(text).unwrap()["objects"].as_array().unwrap().clone()
}

fn stroke_widths(text: &str) -> Vec<f64> {
    objects(text).iter().filter(|o| o["name"].as_str().is_some_and(|n| n.starts_with("gen.road."))).flat_map(|o| o["strokes"].as_array().into_iter().flatten().map(|s| s["width"].as_f64().unwrap()).collect::<Vec<_>>()).collect()
}

#[test]
fn a_narrow_band_is_widened_at_1_to_5000() {
    assert_eq!(placed(NARROW, 0.0)[0].width, 0.5);
    let wide = placed(NARROW, 0.3 * 5000.0 / 1000.0);
    assert!(wide.iter().all(|e| (e.width - 1.5).abs() < 1e-9), "{:?}", wide.iter().map(|e| e.width).collect::<Vec<_>>());
}

#[test]
fn wide_bands_are_not_changed() {
    let p = placed("lane back | lane fwd", 1.5);
    assert!(p.iter().all(|e| (e.width - 3.6).abs() < 1e-9));
}

#[test]
fn widening_is_about_the_centre() {
    let plain = placed(NARROW, 0.0);
    let wide = placed(NARROW, 1.5);
    let (pl, pr) = (plain.first().unwrap().left, plain.last().unwrap().right);
    let (wl, wr) = (wide.first().unwrap().left, wide.last().unwrap().right);
    assert!((pl + pr).abs() < 1e-9 && (wl + wr).abs() < 1e-9, "both stay symmetric about the line: {pl} {pr} / {wl} {wr}");
    assert!((wr - 1.5).abs() < 1e-9 && (wl + 1.5).abs() < 1e-9);
}

#[test]
fn the_spine_does_not_move() {
    let kept = |t: &str| objects(t).into_iter().filter(|o| o["name"].as_str().is_none_or(|n| !n.starts_with("gen."))).collect::<Vec<_>>();
    let a = road(json!({"target_print_scale": "true"}), "lane back | lane fwd").unwrap();
    let b = road(json!({"target_print_scale": "1:5000"}), "lane back | lane fwd").unwrap();
    assert_eq!(kept(&a), kept(&b));
}

#[test]
fn a_build_at_a_scale_widens_the_road_and_strokes_have_a_minimum() {
    let plain = road(json!({}), NARROW).unwrap();
    let scaled = road(json!({"target_print_scale": "1:5000"}), NARROW).unwrap();
    assert_ne!(objects(&plain), objects(&scaled));
    assert!(stroke_widths(&plain).iter().any(|&w| w < 0.6), "plain strokes are thin");
    assert!(stroke_widths(&scaled).iter().all(|&w| w >= 0.6 - 1e-9), "{:?}", stroke_widths(&scaled));
}

#[test]
fn extreme_scales_stay_finite() {
    for scale in ["1:1", "1:1000000"] {
        let text = road(json!({"target_print_scale": scale}), "lane back | lane fwd").unwrap();
        let v: Value = serde_json::from_str(&text).unwrap();
        let s = v.to_string();
        assert!(!s.contains("NaN") && !s.contains("inf"), "{scale}");
        assert!(stroke_widths(&text).iter().all(|w| w.is_finite() && *w > 0.0), "{scale}");
    }
}

#[test]
fn a_track_is_not_widened_or_thickened() {
    let input = json!({"objects": [{"id": 1, "name": "track.a", "path": {"subpaths": [{"anchors": [{"p": [0.0, 0.0]}, {"p": [300.0, 0.0]}], "closed": false}]}}]});
    let params = |s: &str| json!({"_context": {"documentData": {"schema": 1, "tracks": {"a": {"tracks": 1}}, "settings": {"target_print_scale": s}}, "artboard": [0, 0, 600, 800]}});
    let a = build_text(&input, &params("true")).unwrap();
    let b = build_text(&input, &params("1:5000")).unwrap();
    let rail = |t: &str| objects(t).into_iter().filter(|o| o["name"].as_str().is_some_and(|n| n.starts_with("gen.track."))).collect::<Vec<_>>();
    assert_eq!(rail(&a), rail(&b));
}
