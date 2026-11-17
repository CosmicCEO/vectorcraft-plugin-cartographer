use serde_json::{Value, json};
use cartographer_core::rail_builder as builder;
use cartographer_core::joint::{Band, Found, JointSpec, TOLERANCE, find, nearest_frog};
use cartographer_core::rail_standards::*;

type P = [f64; 2];

fn at_deg(deg: f64, len: f64, from: P) -> Vec<P> {
    let r = deg.to_radians();
    vec![from, [from[0] + len * r.cos(), from[1] + len * r.sin()]]
}

#[test]
fn two_lines_that_cross_at_right_angles() {
    let a = vec![[0.0, 0.0], [400.0, 0.0]];
    let b = vec![[150.0, -100.0], [150.0, 100.0]];
    let Some(Found::Crossing { point, s_a, s_b, angle_deg }) = find(&a, &b) else { panic!("not found") };
    assert!((point[0] - 150.0).abs() < 1e-9 && point[1].abs() < 1e-9);
    assert!((s_a - 150.0).abs() < 1e-9 && (s_b - 100.0).abs() < 1e-9 && (angle_deg - 90.0).abs() < 1e-9);
}

#[test]
fn an_oblique_crossing_reports_the_acute_angle_whichever_way_round() {
    // b runs at 20 degrees through the middle of a (along x): the angle is 20, not 160.
    let a = vec![[0.0, 0.0], [600.0, 0.0]];
    for deg in [20.0, 200.0, 160.0] {
        let b = at_deg(deg, 300.0, [300.0 - 150.0 * deg.to_radians().cos(), -150.0 * deg.to_radians().sin()]);
        let Some(Found::Crossing { angle_deg, .. }) = find(&a, &b) else { panic!("{deg}") };
        let want = if deg == 160.0 { 20.0 } else if deg == 200.0 { 20.0 } else { deg };
        assert!((angle_deg - want).abs() < 1e-6, "{deg}: {angle_deg}");
    }
    let (n, deg) = nearest_frog(9.5);
    assert_eq!(n, 6);
    assert!((deg - 9.527).abs() < 1e-9);
}

#[test]
fn a_crossing_curved_lines_is_found_on_the_curve() {
    // a is a quarter circle of radius 300 about (0, 300); b is the vertical x = 212.13 (the circle's point at 45 degrees is (212.13, 87.87)).
    let a: Vec<P> = (0..=90).map(|i| { let t = i as f64 * std::f64::consts::FRAC_PI_2 / 90.0; [300.0 * t.sin(), 300.0 - 300.0 * t.cos()] }).collect();
    let b = vec![[212.132, -50.0], [212.132, 200.0]];
    let Some(Found::Crossing { point, angle_deg, .. }) = find(&a, &b) else { panic!("not found") };
    assert!((point[1] - 87.868).abs() < 0.1, "{point:?}");
    assert!((angle_deg - 45.0).abs() < 0.6, "{angle_deg}"); // the line leaves the circle at 45 degrees to the vertical
}

#[test]
fn an_end_on_the_other_line_is_a_branch_and_there_is_a_tolerance() {
    let main = vec![[0.0, 0.0], [600.0, 0.0]];
    let on = vec![[300.0, 0.0], [380.0, 120.0]]; // starts exactly on the main
    let Some(Found::Branch { branch, s_main, gap, .. }) = find(&main, &on) else { panic!("branch") };
    assert_eq!(branch, 1);
    assert!((s_main - 300.0).abs() < 1e-9 && gap < 1e-9);
    let short = vec![[300.0, 20.0], [380.0, 120.0]]; // 20 mm short of the main: still meets it
    let Some(Found::Branch { gap, .. }) = find(&main, &short) else { panic!("branch within tolerance") };
    assert!((gap - 20.0).abs() < 1e-9);
    let far = vec![[300.0, TOLERANCE + 10.0], [380.0, 120.0]];
    assert!(find(&main, &far).is_none(), "further than the tolerance: they do not meet");
    // which line is the branch does not depend on the order they are given in
    let Some(Found::Branch { branch, .. }) = find(&on, &main) else { panic!("branch") };
    assert_eq!(branch, 0);
}

#[test]
fn a_crossing_close_to_an_end_is_a_branch_not_a_crossing() {
    let a = vec![[0.0, 0.0], [600.0, 0.0]];
    let b = vec![[300.0, -200.0], [300.0, 10.0]]; // crosses a, but only 10 mm of it is past a
    assert!(matches!(find(&a, &b), Some(Found::Branch { branch: 1, .. })), "{:?}", find(&a, &b));
}

#[test]
fn ends_that_meet_are_a_join() {
    let a = vec![[0.0, 0.0], [300.0, 0.0]];
    let b = vec![[305.0, 8.0], [600.0, 40.0]];
    let Some(Found::Join { gap, angle_deg, .. }) = find(&a, &b) else { panic!("join") };
    assert!((gap - 9.43).abs() < 0.01 && angle_deg > 5.0 && angle_deg < 8.0, "{gap} {angle_deg}");
}

#[test]
fn what_was_found_is_said_in_words() {
    let a = vec![[0.0, 0.0], [600.0, 0.0]];
    let b = at_deg(10.0, 400.0, [300.0 - 200.0 * 10f64.to_radians().cos(), -200.0 * 10f64.to_radians().sin()]);
    let s = find(&a, &b).unwrap().describe("a", "b");
    assert!(s.contains("lines a and b cross") && s.contains("300 mm along a") && s.contains("nearest NMRA frog is No. 6"), "{s}");
}

// ---- in a document

fn path(id: i64, name: &str, pts: &[P]) -> Value {
    let anchors: Vec<Value> = pts.iter().map(|p| json!({"p": [p[0] * PT_PER_MM, -p[1] * PT_PER_MM]})).collect();
    json!({"id": id, "name": name, "path": {"subpaths": [{"anchors": anchors, "closed": false}]}})
}

fn doc(lines: &[(&str, Vec<P>)]) -> Value {
    json!({"objects": lines.iter().enumerate().map(|(i, (n, p))| path(i as i64 + 1, &format!("track.{n}"), p)).collect::<Vec<_>>()})
}

fn params(data: Value) -> Value {
    json!({"_context": {"documentData": data}})
}

fn ties_of(out: &Value, id: &str) -> Vec<[f64; 2]> {
    let name = format!("gen.track.{id}.t1.ties");
    let Some(o) = out["objects"].as_array().unwrap().iter().find(|o| o["name"] == name.as_str()) else { return vec![] };
    o["path"]["subpaths"].as_array().unwrap().iter().map(|s| {
        let a = s["anchors"].as_array().unwrap();
        let g = |i: usize, k: usize| a[i]["p"][k].as_f64().unwrap() / PT_PER_MM;
        [(g(0, 0) + g(1, 0)) / 2.0, -(g(0, 1) + g(1, 1)) / 2.0]
    }).collect()
}

#[test]
fn the_track_that_runs_through_keeps_its_ties_the_other_stops_at_its_edge() {
    // a: two tracks along x. b: one track along y, crossing a at x = 300. a is wider, so it runs through.
    let lines = doc(&[("a", vec![[0.0, 0.0], [600.0, 0.0]]), ("b", vec![[300.0, -200.0], [300.0, 200.0]])]);
    let data = |joints: Value| params(json!({"tracks": {"a": {"tracks": 2}, "b": {"tracks": 1}}, "joints": joints}));
    let without = builder::build(&lines, &data(json!({}))).unwrap();
    let with = builder::build(&lines, &data(json!({"j1": {"lines": ["a", "b"], "kind": "crossing"}}))).unwrap();
    assert!(with["documentData"]["notes"].as_array().unwrap().is_empty(), "{}", with["documentData"]["notes"]);
    // a is untouched
    assert_eq!(ties_of(&without, "a").len(), ties_of(&with, "a").len());
    // b loses the ties inside a's strip: |y| <= half of a's footprint = TRACK_CENTERS / 2 + TIE_LENGTH / 2
    let half = TRACK_CENTERS / 2.0 + TIE_LENGTH / 2.0;
    let (before, after) = (ties_of(&without, "b"), ties_of(&with, "b"));
    assert!(before.iter().any(|t| t[1].abs() < half), "there were ties there to remove");
    assert!(after.iter().all(|t| t[1].abs() > half - 1e-9), "none inside the strip");
    let removed = before.len() - after.len();
    let expected = (2.0 * half / TIE_PITCH).round() as usize;
    assert!(removed.abs_diff(expected) <= 1, "removed {removed}, strip holds about {expected}");
}

#[test]
fn main_says_which_runs_through() {
    let lines = doc(&[("a", vec![[0.0, 0.0], [600.0, 0.0]]), ("b", vec![[300.0, -200.0], [300.0, 200.0]])]);
    let data = params(json!({"tracks": {"a": {"tracks": 1}, "b": {"tracks": 1}}, "joints": {"j1": {"lines": ["a", "b"], "kind": "crossing", "main": "b"}}}));
    let out = builder::build(&lines, &data).unwrap();
    let full_a = builder::build(&lines, &params(json!({"tracks": {"a": {"tracks": 1}}}))).unwrap();
    assert!(ties_of(&out, "a").len() < ties_of(&full_a, "a").len(), "with main b, line a gives way");
    // equal width and no `main`: the first by name runs through
    let out = builder::build(&lines, &params(json!({"tracks": {}, "joints": {"j1": {"lines": ["a", "b"], "kind": "crossing"}}}))).unwrap();
    let b_full = builder::build(&lines, &params(json!({"tracks": {}}))).unwrap();
    assert_eq!(ties_of(&out, "a").len(), ties_of(&b_full, "a").len());
    assert!(ties_of(&out, "b").len() < ties_of(&b_full, "b").len());
}

#[test]
fn a_joint_that_cannot_be_applied_is_left_out_with_a_note() {
    let lines = doc(&[("a", vec![[0.0, 0.0], [600.0, 0.0]]), ("b", vec![[300.0, 100.0], [300.0, 300.0]])]); // b never reaches a
    let note = |joints: Value| {
        let out = builder::build(&lines, &params(json!({"joints": joints}))).unwrap();
        out["documentData"]["notes"][0].as_str().unwrap().to_string()
    };
    assert!(note(json!({"j1": {"lines": ["a", "b"], "kind": "crossing"}})).contains("do not meet within 30 mm"));
    assert!(note(json!({"j1": {"lines": ["a", "zz"], "kind": "crossing"}})).contains("not in the document"));
    assert!(note(json!({"j1": {"lines": ["a"], "kind": "crossing"}})).contains("two lines"));
    let touching = doc(&[("a", vec![[0.0, 0.0], [600.0, 0.0]]), ("b", vec![[300.0, 0.0], [300.0, 200.0]])]);
    let out = builder::build(&touching, &params(json!({"joints": {"j1": {"lines": ["a", "b"], "kind": "crossing"}}}))).unwrap();
    let note = out["documentData"]["notes"][0].as_str().unwrap();
    assert!(note.contains("Joint j1 is a crossing but line b ends on line a"), "{note}");
}

#[test]
fn set_joint_stores_a_crossing_and_reuses_its_id() {
    let lines = doc(&[("a", vec![[0.0, 0.0], [600.0, 0.0]]), ("b", vec![[300.0, -200.0], [300.0, 200.0]])]);
    let p = |d: &str, data: Value| json!({"description": d, "_context": {"command": "set_joint", "documentData": data}});
    let first: Value = serde_json::from_str(&builder::set_joint_text(&lines, &p("{\"kind\":\"crossing\",\"main\":\"a\"}", json!({}))).unwrap()).unwrap();
    assert_eq!(first["documentData"]["joints"]["j1"], json!({"kind": "crossing", "main": "a", "lines": ["a", "b"]}));
    let again: Value = serde_json::from_str(&builder::set_joint_text(&lines, &p("{\"kind\":\"crossing\"}", first["documentData"].clone())).unwrap()).unwrap();
    assert_eq!(again["documentData"]["joints"].as_object().unwrap().len(), 1, "same pair, same joint");
    assert!(again["documentData"]["joints"]["j1"].get("main").is_none(), "the new description replaces the old");
}

#[test]
fn set_joint_says_in_plain_words_why_it_cannot() {
    let p = |d: &str| json!({"description": d, "_context": {"command": "set_joint", "documentData": {}}});
    let cross = doc(&[("a", vec![[0.0, 0.0], [600.0, 0.0]]), ("b", vec![[300.0, -200.0], [300.0, 200.0]])]);
    let one = doc(&[("a", vec![[0.0, 0.0], [600.0, 0.0]])]);
    let apart = doc(&[("a", vec![[0.0, 0.0], [600.0, 0.0]]), ("b", vec![[300.0, 100.0], [300.0, 300.0]])]);
    let tee = doc(&[("a", vec![[0.0, 0.0], [600.0, 0.0]]), ("b", vec![[300.0, 0.0], [380.0, 120.0]])]);
    assert!(builder::set_joint_text(&one, &p("{}")).unwrap_err().contains("Select one line of each of the two tracks"));
    assert!(builder::set_joint_text(&apart, &p("{}")).unwrap_err().contains("do not meet"));
    let e = builder::set_joint_text(&tee, &p("{}")).unwrap_err();
    assert!(e.contains("You asked for a crossing, but line b ends on line a"), "{e}");
    assert!(builder::set_joint_text(&cross, &p("{\"main\":\"z\"}")).unwrap_err().contains("one of the two lines"));
    let e = builder::set_joint_text(&cross, &p("{\"kind\":\"branch\"}")).unwrap_err();
    assert!(e.contains("You asked for a branch, but lines a and b cross"), "{e}");
    assert!(builder::set_joint_text(&cross, &p("{\"kind\":\"crossing\",\"frog\":6}")).unwrap_err().contains("turnout of a branch"));
    assert!(builder::set_joint_text(&cross, &p("{\"colour\":1}")).unwrap_err().contains("Unknown joint field"));
    assert!(JointSpec::from_json(&json!(3)).is_err());
}

#[test]
fn a_band_covers_points_within_half_a_width_of_its_line() {
    let band = Band::new(vec![[0.0, 0.0], [100.0, 0.0]], 10.0);
    assert!(band.covers([50.0, 9.9]) && band.covers([50.0, -9.9]) && band.covers([-9.0, 0.0]));
    assert!(!band.covers([50.0, 10.1]) && !band.covers([111.0, 0.0]) && !band.covers([-10.1, 0.0]));
}

#[test]
fn all_roadbed_is_stacked_below_all_ties_below_all_rails_whatever_the_line_order() {
    let lines = doc(&[("a", vec![[0.0, 0.0], [600.0, 0.0]]), ("b", vec![[300.0, -200.0], [300.0, 200.0]])]);
    let out = builder::build(&lines, &params(json!({"tracks": {"a": {"tracks": 2}, "b": {"tracks": 1}}}))).unwrap();
    let rank = |n: &str| if n.ends_with(".ballast") { Some(0) } else if n.ends_with(".ties") || n.ends_with(".switch-ties") { Some(1) } else if n.ends_with(".rails") { Some(2) } else { None };
    let ranks: Vec<i32> = out["objects"].as_array().unwrap().iter().filter_map(|o| o["name"].as_str().and_then(rank)).collect();
    assert!(ranks.windows(2).all(|w| w[0] <= w[1]), "{ranks:?}");
    assert!(ranks.contains(&0) && ranks.contains(&1) && ranks.contains(&2));
    // and line b's roadbed is among the lowest, below line a's ties
    let names: Vec<&str> = out["objects"].as_array().unwrap().iter().filter_map(|o| o["name"].as_str()).collect();
    let pos = |n: &str| names.iter().position(|x| *x == n).unwrap();
    assert!(pos("gen.track.b.ballast") < pos("gen.track.a.t1.ties"));
}
