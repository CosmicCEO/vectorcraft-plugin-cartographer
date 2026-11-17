use cartographer_core::output::Out;
use cartographer_core::rail_builder;
use cartographer_core::vcdoc::items_from_filter_input;
use serde_json::{Value, json};

/// One track line, in document points (metres of ground), 100 m long.
fn track_input() -> Value {
    json!({"objects": [{"id": 1, "name": "track.a", "path": {"subpaths": [{"anchors": [{"p": [0.123456789, 0.0]}, {"p": [100.123456789, 0.0]}], "closed": false}]}}]})
}

#[test]
fn a_bad_stored_description_is_an_error_not_a_panic() {
    let items = items_from_filter_input(&track_input());
    let mut out = Out::new();
    let e = rail_builder::build_into(&items, &json!({"tracks": {"a": {"tracks": 99}}}), &mut out).unwrap_err();
    assert!(e.contains("1 to"), "{e}");
    // The settings are put back even when the build fails.
    out.path("gen.road.after", &[vec![[1.23456789, 0.0], [2.0, 1.0]]], |p| p, [0.0; 3], 1.0, false);
    assert!(out.finish().contains("[1.235,0]"));
}

#[test]
fn build_into_writes_five_decimals_and_restores_three() {
    let items = items_from_filter_input(&track_input());
    let mut out = Out::new();
    let data = json!({"tracks": {"a": {"tracks": 1}}});
    let notes = rail_builder::build_into(&items, &data, &mut out).unwrap();
    assert!(notes.is_empty(), "{notes:?}");
    // A road-style object written afterwards with the same output has 3 decimals again.
    out.path("gen.road.after", &[vec![[1.23456789, 0.0], [2.0, 1.0]]], |p| p, [0.0; 3], 1.0, false);
    let text = out.finish();
    assert!(text.contains("[1.235,0]"), "{text}");
    let rail_part = &text[..text.find("gen.road.after").unwrap()];
    let long = rail_part.split(|c: char| !(c.is_ascii_digit() || c == '.')).any(|n| n.split('.').nth(1).is_some_and(|f| f.len() > 3));
    assert!(long, "the track objects use more than 3 decimals somewhere");
}
