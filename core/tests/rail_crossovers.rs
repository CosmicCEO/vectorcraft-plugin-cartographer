use serde_json::{Value, json};
use cartographer_core::rail_builder as builder;
use cartographer_core::rail_standards::*;
use cartographer_core::turnout::Frog;
use cartographer_core::yard::YardSpec;

fn input(pts: &[[f64; 2]]) -> Value {
    let anchors: Vec<Value> = pts.iter().map(|p| json!({"p": p})).collect();
    json!({"objects": [{"id": 1, "name": "track.a", "path": {"subpaths": [{"anchors": anchors, "closed": false}]}}]})
}

fn line(mm: f64) -> Value {
    input(&[[0.0, 0.0], [mm * PT_PER_MM, 0.0]])
}

fn yard(spec: Value) -> Value {
    json!({"_context": {"documentData": {"yards": {"a": spec}}}})
}

fn object<'a>(out: &'a Value, name: &str) -> Option<&'a Value> {
    out["objects"].as_array().unwrap().iter().find(|o| o["name"] == name)
}

/// Points of subpath `i` in millimetres, y UP.
fn sub(o: &Value, i: usize) -> Vec<[f64; 2]> {
    o["path"]["subpaths"][i]["anchors"].as_array().unwrap().iter().map(|a| [a["p"][0].as_f64().unwrap() / PT_PER_MM, -a["p"][1].as_f64().unwrap() / PT_PER_MM]).collect()
}

fn centre(out: &Value, route: &str, last: bool) -> [f64; 2] {
    let rails = object(out, &format!("gen.track.a.{route}.rails")).unwrap_or_else(|| panic!("no route {route}"));
    let (l, r) = (sub(rails, 0), sub(rails, 1));
    let pick = |v: &Vec<[f64; 2]>| if last { v[v.len() - 1] } else { v[0] };
    let (a, b) = (pick(&l), pick(&r));
    [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0]
}

fn near(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}

#[test]
fn a_crossover_runs_from_one_main_to_the_next() {
    let half = TRACK_CENTERS / 2.0;
    let f = Frog::new(6);
    let out = builder::build(&line(900.0), &yard(json!({"mains": 2, "crossovers": [{"from": 1, "to": 2, "at": 300}]}))).unwrap();
    assert!(out["documentData"]["notes"].as_array().unwrap().is_empty(), "{}", out["documentData"]["notes"]);
    let (start, end) = (centre(&out, "X1", false), centre(&out, "X1", true));
    // Main 1 is the left main (y = +half), main 2 the right (y = -half): the crossover goes from left to right, i.e. downward.
    assert!(near(start[0], 300.0, 0.3) && near(start[1], half, 0.02), "{start:?}");
    assert!(near(end[1], -half, 0.05), "it arrives on the second main: {end:?}");
    // Along the line it takes two leads and the straight run between: 2 x (end of the closure arc) + run cos(a), less the arc's own cos share.
    let ext = YardSpec::from_json(&json!({"mains": 2, "crossovers": [{"from": 1, "to": 2}]})).unwrap().crossover_extent();
    assert!(near(end[0] - start[0], ext, 0.6), "{} vs {ext}", end[0] - start[0]);
    let run = f.crossover_run(TRACK_CENTERS).unwrap();
    assert!(near(run, (TRACK_CENTERS - 2.0 * f.end()[1]) / f.alpha.sin(), 1e-9));
    // Its turnouts are counted and said.
    let spec = YardSpec::from_json(&json!({"mains": 2, "crossovers": [{"from": 1, "to": 2}, {"from": 2, "to": 1, "at": 600}]})).unwrap();
    assert_eq!(spec.turnouts(), 4);
    assert!(spec.summary().contains("2 crossovers"), "{}", spec.summary());
}

#[test]
fn the_other_way_round_goes_from_right_to_left() {
    let half = TRACK_CENTERS / 2.0;
    let out = builder::build(&line(900.0), &yard(json!({"mains": 2, "crossovers": [{"from": 2, "to": 1, "at": 300}]}))).unwrap();
    let (start, end) = (centre(&out, "X1", false), centre(&out, "X1", true));
    assert!(near(start[1], -half, 0.02) && near(end[1], half, 0.05), "{start:?} {end:?}");
}

#[test]
fn without_at_the_crossover_is_in_the_middle() {
    let out = builder::build(&line(1000.0), &yard(json!({"mains": 2, "crossovers": [{"from": 1, "to": 2}]}))).unwrap();
    let (start, end) = (centre(&out, "X1", false), centre(&out, "X1", true));
    assert!(near((start[0] + end[0]) / 2.0, 500.0, 0.6), "{start:?} {end:?}");
}

#[test]
fn every_frog_number_fits_between_tracks_a_standard_distance_apart() {
    for n in 4..=20u32 {
        let f = Frog::new(n);
        let pts = f.crossover(0.0, 0.0, 1.0, TRACK_CENTERS).unwrap_or_else(|| panic!("No. {n} does not fit"));
        let end = pts[pts.len() - 1];
        assert!(near(end[1], TRACK_CENTERS, 0.02), "No. {n}: arrives {} mm over, want {TRACK_CENTERS}", end[1]);
        // and also for the narrowest NMRA centres, old-time 12 ft
        assert!(f.crossover(0.0, 0.0, 1.0, 12.0 * FOOT).is_some(), "No. {n} at 12 ft");
    }
}

#[test]
fn both_mains_get_switch_ties_that_cover_the_whole_crossover() {
    let out = builder::build(&line(900.0), &yard(json!({"mains": 2, "crossovers": [{"from": 1, "to": 2, "at": 300}]}))).unwrap();
    for name in ["gen.track.a.m1.switch-ties", "gen.track.a.m2.switch-ties", "gen.track.a.X1.rails"] {
        assert!(object(&out, name).is_some(), "{name}");
    }
    // The tracks are 24.8 mm apart and a lengthened tie reaches 17 mm out from its own main, so the two mains' ties between them reach the
    // whole crossover: it has no ties of its own.
    assert!(object(&out, "gen.track.a.X1.ties").is_none());
}

#[test]
fn the_roadbed_is_one_band_across_the_mains() {
    let out = builder::build(&line(900.0), &yard(json!({"mains": 2, "crossovers": [{"from": 1, "to": 2}]}))).unwrap();
    let bed = object(&out, "gen.track.a.ballast").unwrap();
    assert_eq!(bed["path"]["subpaths"].as_array().unwrap().len(), 1, "no strip of bare ground between the two mains");
}

#[test]
fn bad_crossovers_are_refused_in_plain_words() {
    for (bad, want) in [
        (json!({"mains": 2, "crossovers": [{"from": 1, "to": 1}]}), "neighbour"),
        (json!({"mains": 3, "crossovers": [{"from": 1, "to": 3}]}), "neighbour"),
        (json!({"mains": 1, "crossovers": [{"from": 1, "to": 2}]}), "only 1 main"),
        (json!({"mains": 2, "crossovers": [{"from": 1}]}), "needs `from` and `to`"),
        (json!({"mains": 2, "crossovers": [{"from": 1, "to": 2, "colour": 1}]}), "Unknown crossover field"),
        (json!({"mains": 2, "crossovers": "yes"}), "must be a list"),
        (json!({"mains": 2, "frog": 20, "spacing": "10mm", "crossovers": [{"from": 1, "to": 2}]}), "overlap"),
    ] {
        let e = YardSpec::from_json(&bad).unwrap_err();
        assert!(e.contains(want), "{bad}: {e}");
    }
}

#[test]
fn set_yard_refuses_a_line_too_short_for_the_crossover() {
    let p = |d: &str| json!({"description": d, "_context": {"command": "set_yard", "documentData": {}}});
    let e = builder::set_yard_text(&line(120.0), &p("{\"mains\":2,\"crossovers\":[{\"from\":1,\"to\":2}]}")).unwrap_err();
    assert!(e.contains("crossover between mains 1 and 2 needs"), "{e}");
    let e = builder::set_yard_text(&line(500.0), &p("{\"mains\":2,\"crossovers\":[{\"from\":1,\"to\":2,\"at\":72}]}")).unwrap_err();
    assert!(e.contains("450"), "a description in metres (72 m), checked in model mm (450): {e}");
    assert!(builder::set_yard_text(&line(900.0), &p("{\"mains\":2,\"crossovers\":[{\"from\":1,\"to\":2}]}")).is_ok());
}

#[test]
fn a_crossover_that_does_not_fit_is_left_out_with_a_note() {
    let out = builder::build(&line(900.0), &yard(json!({"mains": 2, "crossovers": [{"from": 1, "to": 2, "at": 850}]}))).unwrap();
    assert!(object(&out, "gen.track.a.X1.rails").is_none());
    assert!(out["documentData"]["notes"][0].as_str().unwrap().contains("does not fit"));
}

#[test]
fn no_object_is_ever_written_without_points() {
    // VectorCraft refuses an object whose path has no anchors, so a route with no room for a tie must not write an empty tie object.
    let out = builder::build(&line(900.0), &yard(json!({"mains": 2, "right": 1, "crossovers": [{"from": 1, "to": 2}]}))).unwrap();
    for o in out["objects"].as_array().unwrap() {
        if let Some(subs) = o["path"]["subpaths"].as_array() {
            assert!(!subs.is_empty() && subs.iter().all(|s| s["anchors"].as_array().is_some_and(|a| !a.is_empty())), "{}", o["name"]);
        }
    }
}
