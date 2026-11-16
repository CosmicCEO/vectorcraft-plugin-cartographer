use cartographer_core::geom::Len2;
use cartographer_core::output::Out;
use cartographer_core::vcdoc::items_from_filter_input;
use serde_json::json;

fn shape(closed: bool) -> serde_json::Value {
    json!({"objects": [{"id": 1, "name": "water.l", "path": {"subpaths": [{"anchors": [{"p": [0, 0]}, {"p": [10, 0]}, {"p": [10, 10]}], "closed": closed}]}}]})
}

#[test]
fn closed_subpath_ends_where_it_began() {
    let items = items_from_filter_input(&shape(true));
    let sub = &items[0].subs[0];
    assert_eq!(sub.first(), sub.last());
    assert!(sub.len() >= 4);
}

#[test]
fn open_line_stays_open() {
    let items = items_from_filter_input(&shape(false));
    let sub = &items[0].subs[0];
    assert_eq!(sub.first(), Some(&[0.0, 0.0]));
    assert_eq!(sub.last(), Some(&[10.0, 10.0]));
}

#[test]
fn keep_outline_writes_empty_fills() {
    let mut out = Out::new();
    out.keep_outline(7, [0.0, 0.5, 0.6], 1.0);
    let text = out.finish();
    assert!(text.contains("\"id\":7"), "{text}");
    assert!(text.contains("\"fills\":[]"), "{text}");
}

#[test]
fn out_skips_empty_and_degenerate_objects() {
    let id = |p: [f64; 2]| p;
    let mut out = Out::new();
    out.path("a", &[], id, [0.0; 3], 1.0, false);
    out.path("b", &[vec![[1.0, 1.0]]], id, [0.0; 3], 1.0, false);
    out.polygon("c", &[[0.0, 0.0], [1.0, 0.0]], id, [0.0; 3]);
    out.fills("d", &[vec![[0.0, 0.0], [1.0, 1.0]]], id, [0.0; 3]);
    out.curve("e", &[vec![]], id, [0.0; 3], 1.0);
    out.path("f", &[vec![[0.0, f64::NAN], [1.0, 1.0]]], id, [0.0; 3], 1.0, false);
    let text = out.finish();
    assert!(!text.contains("\"name\""), "{text}");
    assert!(!text.contains("NaN"), "{text}");
}

#[test]
fn out_still_writes_a_good_object() {
    let mut out = Out::new();
    out.path("ok", &[vec![[0.0, 0.0], [1.0, 1.0]]], |p| p, [0.0; 3], 1.0, false);
    assert!(out.finish().contains("\"name\":\"ok\""));
}

#[test]
fn len2_matches_hypot() {
    assert_eq!(3.0_f64.len2(4.0), 5.0);
}
