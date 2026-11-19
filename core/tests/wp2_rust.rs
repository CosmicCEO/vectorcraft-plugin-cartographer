use cartographer_core::builder::{fair_notes, set_tracks_text};
use cartographer_core::output::Out;
use cartographer_core::rail_builder;
use cartographer_core::vcdoc::items_from_filter_input;
use serde_json::{Value, json};

fn line(id: i64, name: &str, pts: &[[f64; 2]]) -> Value {
    json!({"id": id, "name": name, "path": {"subpaths": [{"anchors": pts.iter().map(|p| json!({"p": p})).collect::<Vec<_>>(), "closed": false}]}})
}

#[test]
fn describe_track_never_renames_a_road_model_line() {
    for name in ["centerline", "profile.zero", "flag.gap.1", "arrow.thru.N.12", "J7.signal.right", "C1"] {
        let input = json!({"objects": [line(1, name, &[[0.0, 0.0], [100.0, 0.0]])]});
        let p = json!({"description": "{\"tracks\": 1}", "_context": {"documentData": {}}});
        let r = set_tracks_text(&input, &p);
        assert!(r.is_err(), "{name} was taken as a plain line: {r:?}");
    }
}

#[test]
fn a_road_written_after_the_rails_goes_back_on_the_roadbed_stack() {
    let items = items_from_filter_input(&json!({"objects": [line(1, "track.a", &[[0.0, 0.0], [200.0, 0.0]])]}));
    let mut out = Out::new();
    rail_builder::build_into(&items, &json!({"tracks": {"a": {"tracks": 1}}}), &mut out).unwrap();
    out.fills("gen.road.x.1.surface", &[vec![[0.0, 0.0], [5.0, 0.0], [5.0, 5.0]]], |p| p, [0.5, 0.5, 0.5]);
    let v: Value = serde_json::from_str(&out.finish()).unwrap();
    let names: Vec<String> = v["objects"].as_array().unwrap().iter().filter_map(|o| o["name"].as_str().map(String::from)).collect();
    let road = names.iter().position(|n| n == "gen.road.x.1.surface").unwrap();
    let rails = names.iter().position(|n| n.ends_with(".rails")).unwrap();
    assert!(road < rails, "{names:?}");
}

#[test]
fn notes_are_shared_fairly_between_roads_and_tracks() {
    let roads: Vec<String> = (0..40).map(|i| format!("road {i}")).collect();
    let tracks = vec!["track 0".to_string()];
    let n = fair_notes(roads.clone(), tracks.clone(), 30);
    assert_eq!(n.len(), 30);
    assert!(n.contains(&"track 0".to_string()), "the track note survives");
    assert_eq!(fair_notes(roads[..3].to_vec(), tracks.clone(), 30).len(), 4, "short lists are kept whole");
    let many: Vec<String> = (0..40).map(|i| format!("track {i}")).collect();
    let both = fair_notes(roads, many, 30);
    assert_eq!(both.iter().filter(|x| x.starts_with("road")).count(), 15);
}
