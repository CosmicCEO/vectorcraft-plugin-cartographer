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

#[test]
fn zero_negative_and_non_finite_strokes_are_skipped() {
    let id = |p: [f64; 2]| p;
    let line = vec![vec![[0.0, 0.0], [5.0, 5.0]]];
    let mut out = Out::new();
    for w in [0.0, -2.0, f64::NAN, f64::INFINITY] {
        out.path("a", &line, id, [0.0; 3], w, false);
        out.curve("b", &line, id, [0.0; 3], w);
        out.segments("c", [[[0.0, 0.0], [1.0, 1.0]]].into_iter(), id, [0.0; 3], w);
    }
    let text = out.finish();
    assert!(!text.contains("\"name\""), "{text}");
}

#[test]
fn a_segment_with_a_non_finite_point_is_skipped_not_drawn_to_the_origin() {
    let mut out = Out::new();
    out.segments("ties", [[[0.0, 0.0], [1.0, 1.0]], [[f64::NAN, 3.0], [4.0, 4.0]]].into_iter(), |p| p, [0.0; 3], 0.5);
    let text = out.finish();
    assert!(text.contains("\"name\":\"ties\""), "{text}");
    assert!(!text.contains("\"p\":[0,3]") && !text.contains("NaN"), "{text}");
}

#[test]
fn a_line_whose_points_are_all_the_same_is_skipped() {
    let mut out = Out::new();
    out.path("dot", &[vec![[1.0, 1.0], [1.0, 1.0]]], |p| p, [0.0; 3], 1.0, false);
    assert!(!out.finish().contains("\"name\""));
}

#[test]
fn keep_styled_with_a_bad_width_just_keeps_the_object() {
    let mut out = Out::new();
    out.keep_styled(3, [0.0; 3], 0.0);
    let text = out.finish();
    assert!(text.contains("\"id\":3") && !text.contains("strokes"), "{text}");
}

#[test]
fn a_closed_ring_keeps_its_corners_through_curve_fills() {
    let ring = vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0], [0.0, 0.0]];
    let mut out = Out::new();
    out.curve_fills("lake", &[ring], |p| p, [0.0; 3]);
    let text = out.finish();
    assert!(text.matches("\"p\":[").count() >= 4, "{text}");
    assert!(text.contains("\"p\":[10,10]"), "{text}");
}

#[test]
fn a_closed_two_anchor_lens_is_closed() {
    let v = json!({"objects": [{"id": 1, "name": "water.l", "path": {"subpaths": [{"anchors": [{"p": [0, 0], "out": [3, 5]}, {"p": [10, 0], "out": [7, -5]}], "closed": true}]}}]});
    let items = items_from_filter_input(&v);
    assert!(items[0].closed);
    let sub = &items[0].subs[0];
    assert_eq!(sub.first(), sub.last());
    assert!(sub.len() > 17, "the closing curve is kept: {}", sub.len());
}

#[test]
fn an_open_loop_line_is_not_a_shape() {
    let v = json!({"objects": [{"id": 1, "name": "road.a", "path": {"subpaths": [{"anchors": [{"p": [0, 0]}, {"p": [10, 0]}, {"p": [10, 10]}, {"p": [0, 0]}], "closed": false}]}}]});
    assert!(!items_from_filter_input(&v)[0].closed);
}

#[test]
fn the_closed_flag_follows_the_document() {
    assert!(items_from_filter_input(&shape(true))[0].closed);
    assert!(!items_from_filter_input(&shape(false))[0].closed);
}

#[test]
fn a_plain_polygon_keeps_a_repeated_first_point_as_drawn() {
    // Road Builder writes some rings that end where they began (a slope band that starts from nothing); they stay exactly as given.
    let mut out = Out::new();
    out.polygon("slope", &[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 0.0]], |p| p, [0.0; 3]);
    assert_eq!(out.finish().matches("\"p\":[").count(), 4);
}
