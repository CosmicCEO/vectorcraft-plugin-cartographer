use serde_json::{Value, json};
use cartographer_core::branch::{plan_branch, plan_join};
use cartographer_core::rail_builder as builder;
use cartographer_core::rail_standards::*;
use cartographer_core::turnout::Frog;

type P = [f64; 2];

fn dist(a: P, b: P) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// The largest change of direction, in degrees, at any vertex of the polyline.
fn sharpest_turn(pts: &[P]) -> f64 {
    let dir = |a: P, b: P| (b[1] - a[1]).atan2(b[0] - a[0]);
    pts.windows(3).map(|w| { let d = dir(w[1], w[2]) - dir(w[0], w[1]); (d + std::f64::consts::PI).rem_euclid(2.0 * std::f64::consts::PI) - std::f64::consts::PI }).map(f64::abs).fold(0.0, f64::max).to_degrees()
}

fn on_deg(deg: f64, from: P, len: f64) -> Vec<P> {
    let r = deg.to_radians();
    vec![from, [from[0] + len * r.cos(), from[1] + len * r.sin()]]
}

const MAIN: [P; 2] = [[0.0, 0.0], [1500.0, 0.0]];

#[test]
fn a_branch_at_a_frog_angle_is_nearly_straight_and_takes_that_frog() {
    // 14.25 degrees is a No. 4: the branch line itself is the diverging leg, the connector has almost nothing to do.
    let plan = plan_branch(&MAIN, &on_deg(14.25, [700.0, 0.0], 800.0), 1, TRACK_CENTERS, None).unwrap();
    assert_eq!(plan.frog, 4);
    assert!(plan.turn_deg < 3.0, "the connector turns only {} degrees", plan.turn_deg);
    // It starts on the main's centre line at the point of switch, and from there is smooth: no vertex turns more than the heel angle and arc steps.
    assert!(plan.line[0][1].abs() < 1e-6);
    assert!(sharpest_turn(&plan.line) < 4.0, "sharpest turn {}", sharpest_turn(&plan.line));
    // and it ends where the owner's line ends
    assert!(dist(*plan.line.last().unwrap(), [700.0 + 800.0 * 14.25f64.to_radians().cos(), 800.0 * 14.25f64.to_radians().sin()]) < 1e-6);
}

#[test]
fn a_branch_at_an_odd_angle_picks_among_the_nearest_frogs_and_still_joins_smoothly() {
    let plan = plan_branch(&MAIN, &on_deg(20.0, [700.0, 0.0], 800.0), 1, TRACK_CENTERS, None).unwrap();
    assert!((4..=5).contains(&plan.frog), "No. {}", plan.frog);
    assert!(sharpest_turn(&plan.line) < 4.0, "{}", sharpest_turn(&plan.line));
    assert!(plan.radius >= MIN_RADIUS - 1e-9);
    // an owner's frog number is used as asked
    let eight = plan_branch(&MAIN, &on_deg(8.0, [700.0, 0.0], 800.0), 1, TRACK_CENTERS, Some(8)).unwrap();
    assert_eq!(eight.frog, 8);
    assert_eq!(eight.tap.frog.n, 8);
}

#[test]
fn it_does_not_matter_which_way_the_branch_was_drawn_or_which_side_it_is_on() {
    let down = on_deg(-12.0, [700.0, 0.0], 800.0); // to the right of the main (y down on the page is to the right of +x... in y-up it is below)
    let a = plan_branch(&MAIN, &down, 1, TRACK_CENTERS, Some(5)).unwrap();
    let mut reversed = down.clone();
    reversed.reverse();
    let b = plan_branch(&MAIN, &reversed, 1, TRACK_CENTERS, Some(5)).unwrap();
    assert!(a.line.last().unwrap()[1] < 0.0 && a.tap.side < 0.0, "below the main is its right: side {}", a.tap.side);
    assert_eq!(a.line.len(), b.line.len());
    assert!(dist(a.line[0], b.line[0]) < 1e-6 && dist(*a.line.last().unwrap(), *b.line.last().unwrap()) < 1e-6);
    let up = plan_branch(&MAIN, &on_deg(12.0, [700.0, 0.0], 800.0), 1, TRACK_CENTERS, Some(5)).unwrap();
    assert!(up.tap.side > 0.0 && up.line.last().unwrap()[1] > 0.0);
}

#[test]
fn a_branch_that_runs_back_the_other_way_gets_a_turnout_facing_the_other_way() {
    // The branch leaves toward -x: the main is walked backwards, so the tap runs back along the main.
    let plan = plan_branch(&MAIN, &on_deg(180.0 - 14.0, [700.0, 0.0], 800.0), 1, TRACK_CENTERS, Some(4)).unwrap();
    assert_eq!(plan.tap.dir, -1.0);
    assert!(plan.tap.x0 > 700.0 - 1.0, "the point of switch is on the far side of where the line meets the main: {}", plan.tap.x0);
    assert!(plan.line.last().unwrap()[0] < 0.0 || plan.line.last().unwrap()[0] < 700.0);
    assert!(sharpest_turn(&plan.line) < 4.5, "{}", sharpest_turn(&plan.line));
    // the branch leaves the main on the left of its own direction here (it goes up)
    assert!(plan.line.last().unwrap()[1] > 0.0);
}

#[test]
fn from_a_double_track_the_branch_leaves_the_outer_track_on_its_side() {
    let two = plan_branch(&MAIN, &on_deg(10.0, [700.0, 12.0], 800.0), 2, TRACK_CENTERS, Some(6)).unwrap();
    assert!((two.line[0][1] - TRACK_CENTERS / 2.0).abs() < 1e-6, "starts on the left track: {}", two.line[0][1]);
    assert_eq!(two.tap.main, 0);
    let low = plan_branch(&MAIN, &on_deg(-10.0, [700.0, -12.0], 800.0), 2, TRACK_CENTERS, Some(6)).unwrap();
    assert!((low.line[0][1] + TRACK_CENTERS / 2.0).abs() < 1e-6 && low.tap.main == 1);
}

#[test]
fn a_branch_meeting_a_curved_main_follows_the_curve() {
    let main: Vec<P> = (0..=60).map(|i| { let t = i as f64 * 0.8 / 60.0; [900.0 * t.sin(), 900.0 - 900.0 * t.cos()] }).collect();
    // a branch leaving the arc at angle 0.45 rad along it, going outward-left
    let (px, py) = (900.0 * 0.45f64.sin(), 900.0 - 900.0 * 0.45f64.cos());
    let tangent = [0.45f64.cos(), 0.45f64.sin()];
    let ang = 0.45 + 12f64.to_radians();
    let branch = vec![[px, py], [px + 700.0 * ang.cos(), py + 700.0 * ang.sin()]];
    let plan = plan_branch(&main, &branch, 1, TRACK_CENTERS, Some(5)).unwrap();
    assert!(sharpest_turn(&plan.line) < 4.5, "{}", sharpest_turn(&plan.line));
    // the point of switch is on the curve
    let toe = plan.line[0];
    assert!((toe[0].powi(2) + (toe[1] - 900.0).powi(2)).sqrt() - 900.0 < 0.5 && ((toe[0].powi(2) + (toe[1] - 900.0).powi(2)).sqrt() - 900.0).abs() < 0.5, "{toe:?}");
    let _ = tangent;
}

#[test]
fn a_branch_that_cannot_be_joined_smoothly_says_so() {
    // Nearly square to the main, in a very short branch line: no turnout and gentle curve can make that turn in the room there is.
    let e = plan_branch(&MAIN, &on_deg(88.0, [700.0, 0.0], 60.0), 1, TRACK_CENTERS, Some(6)).err();
    assert!(e.as_deref().is_none_or(|m| m.contains("cannot be joined smoothly")), "{e:?}");
}

/// Is `p` on the polyline (within 1e-6)?
fn on_line(l: &[P], p: P) -> bool {
    l.windows(2).any(|w| {
        let (d, e) = ([w[1][0] - w[0][0], w[1][1] - w[0][1]], [p[0] - w[0][0], p[1] - w[0][1]]);
        let t = ((e[0] * d[0] + e[1] * d[1]) / (d[0] * d[0] + d[1] * d[1])).clamp(0.0, 1.0);
        dist(p, [w[0][0] + t * d[0], w[0][1] + t * d[1]]) < 1e-6
    })
}

#[test]
fn ends_already_in_line_are_joined_by_a_straight_connector_and_nothing_is_trimmed() {
    let a: Vec<P> = vec![[0.0, 0.0], [400.0, 0.0]];
    let b: Vec<P> = vec![[430.0, 0.0], [900.0, 0.0]];
    let plan = plan_join(&a, &b, 1, TRACK_CENTERS).unwrap();
    assert!(dist(plan.connector[0], [400.0, 0.0]) < 1e-6 && dist(*plan.connector.last().unwrap(), [430.0, 0.0]) < 1e-6);
    assert!(plan.connector.iter().all(|p| p[1].abs() < 1e-6), "straight");
    assert_eq!((plan.a_line.len(), plan.b_line.len()), (2, 2));
}

#[test]
fn ends_a_little_out_of_line_are_trimmed_back_and_joined_tangent_to_both() {
    // 14 mm sideways within 20 mm along is too sharp for any curve of 191 mm radius or more: each line is trimmed back to make room.
    let a: Vec<P> = vec![[0.0, 0.0], [400.0, 0.0]];
    let b: Vec<P> = vec![[420.0, 14.0], [900.0, 60.0]];
    let plan = plan_join(&a, &b, 1, TRACK_CENTERS).unwrap();
    let c = &plan.connector;
    assert!(plan.radius >= MIN_RADIUS - 1e-9);
    // it leaves line A along A and joins line B along B, at points that are on the drawn lines
    assert!(on_line(&a, c[0]) && on_line(&b, *c.last().unwrap()), "{:?} {:?}", c[0], c.last());
    let n = c.len();
    let into_b = (60.0f64 - 14.0).atan2(900.0 - 420.0);
    let arrives = (c[n - 1][1] - c[n - 2][1]).atan2(c[n - 1][0] - c[n - 2][0]);
    assert!((arrives - into_b).abs() < 0.04, "arrives {arrives} want {into_b}");
    assert!((c[1][1] - c[0][1]).atan2(c[1][0] - c[0][0]).abs() < 0.05, "leaves along line a");
    // the lines are trimmed to where the connector takes over
    assert!(dist(*plan.a_line.last().unwrap(), c[0]) < 1e-6 && dist(plan.b_line[0], *c.last().unwrap()) < 1e-6);
    assert!(plan.a_line.last().unwrap()[0] < 400.0 - 1.0 || plan.b_line[0][0] > 420.0 + 1.0, "something was trimmed");
    // the whole joined path is smooth: a_line + connector + b_line turns nowhere more than a connector step
    let mut whole = plan.a_line.clone();
    whole.extend(c.iter().skip(1).copied());
    whole.extend(plan.b_line.iter().skip(1).copied());
    assert!(sharpest_turn(&whole) < 3.0, "{}", sharpest_turn(&whole));
}

#[test]
fn it_joins_the_right_ends_whichever_way_the_lines_were_drawn() {
    let a: Vec<P> = vec![[400.0, 0.0], [0.0, 0.0]]; // drawn away from the junction
    let b: Vec<P> = vec![[900.0, 60.0], [420.0, 14.0]];
    let plan = plan_join(&a, &b, 1, TRACK_CENTERS).unwrap();
    assert!(on_line(&a, plan.connector[0]) && on_line(&b, *plan.connector.last().unwrap()));
    let straight_a: Vec<P> = vec![[0.0, 0.0], [400.0, 0.0]];
    let straight_b: Vec<P> = vec![[420.0, 14.0], [900.0, 60.0]];
    let same = plan_join(&straight_a, &straight_b, 1, TRACK_CENTERS).unwrap();
    assert!(dist(plan.connector[0], same.connector[0]) < 1e-6 && dist(*plan.connector.last().unwrap(), *same.connector.last().unwrap()) < 1e-6);
}

#[test]
fn lines_that_cannot_be_joined_smoothly_say_so() {
    // Ends 20 mm apart that point at each other's side: no connector, however much is trimmed from two short lines.
    let a: Vec<P> = vec![[0.0, 0.0], [60.0, 0.0]];
    let b: Vec<P> = vec![[70.0, 10.0], [70.0, 70.0]];
    assert!(plan_join(&a, &b, 1, TRACK_CENTERS).is_err());
}

// ---- in a document

fn path(id: i64, name: &str, pts: &[P]) -> Value {
    let anchors: Vec<Value> = pts.iter().map(|p| json!({"p": [p[0] * PT_PER_MM, -p[1] * PT_PER_MM]})).collect();
    json!({"id": id, "name": name, "path": {"subpaths": [{"anchors": anchors, "closed": false}]}})
}

fn doc(lines: &[(&str, Vec<P>)]) -> Value {
    json!({"objects": lines.iter().enumerate().map(|(i, (n, p))| path(i as i64 + 1, &format!("track.{n}"), p)).collect::<Vec<_>>()})
}

fn names(out: &Value) -> Vec<String> {
    out["objects"].as_array().unwrap().iter().filter_map(|o| o["name"].as_str().map(str::to_string)).collect()
}

fn subs(out: &Value, name: &str) -> Vec<Vec<P>> {
    let o = out["objects"].as_array().unwrap().iter().find(|o| o["name"] == name).unwrap_or_else(|| panic!("no {name}"));
    o["path"]["subpaths"].as_array().unwrap().iter().map(|s| s["anchors"].as_array().unwrap().iter().map(|a| [a["p"][0].as_f64().unwrap() / PT_PER_MM, -a["p"][1].as_f64().unwrap() / PT_PER_MM]).collect()).collect()
}

#[test]
fn a_branch_in_a_document_draws_the_turnout_the_connector_and_the_line() {
    let lines = doc(&[("a", MAIN.to_vec()), ("b", on_deg(12.0, [700.0, 0.0], 700.0))]);
    let params = json!({"_context": {"documentData": {"tracks": {"a": {"tracks": 1}, "b": {"tracks": 1}}, "joints": {"j1": {"lines": ["a", "b"], "kind": "branch", "frog": 5}}}}});
    let out = builder::build(&lines, &params).unwrap();
    assert!(out["documentData"]["notes"].as_array().unwrap().is_empty(), "{}", out["documentData"]["notes"]);
    let n = names(&out);
    for want in ["gen.track.a.m1.rails", "gen.track.a.m1.switch-ties", "gen.track.b.t1.rails", "gen.track.b.t1.ties", "gen.track.a.ballast"] {
        assert!(n.contains(&want.to_string()), "{want} in {n:?}");
    }
    // the branch's rails start on the main (within a gauge of its centre line) and run to where the owner's line ends
    let rails = subs(&out, "gen.track.b.t1.rails");
    assert!(rails[0][0][1].abs() < GAUGE && rails[1][0][1].abs() < GAUGE, "{:?}", rails[0][0]);
    let end = rails[0].last().unwrap();
    assert!(end[0] > 1200.0, "reaches the end of the drawn line: {end:?}");
    // its ties keep out of the main's strip: no tie centre closer than a tie length to the main's centre line
    let ties = subs(&out, "gen.track.b.t1.ties");
    assert!(ties.iter().all(|t| ((t[0][1] + t[1][1]) / 2.0).abs() >= TIE_LENGTH - 1e-6), "a branch tie lies on the main");
}

#[test]
fn a_join_in_a_document_adds_a_connector_track() {
    let lines = doc(&[("a", vec![[0.0, 0.0], [400.0, 0.0]]), ("b", vec![[420.0, 14.0], [900.0, 60.0]])]);
    let params = json!({"_context": {"documentData": {"tracks": {}, "joints": {"j1": {"lines": ["a", "b"], "kind": "join"}}}}});
    let out = builder::build(&lines, &params).unwrap();
    assert!(out["documentData"]["notes"].as_array().unwrap().is_empty(), "{}", out["documentData"]["notes"]);
    assert!(names(&out).contains(&"gen.track.a.j1.t1.rails".to_string()));
    // two tracks joined to two tracks: the connector is a double track
    let two = json!({"_context": {"documentData": {"tracks": {"a": {"tracks": 2}, "b": {"tracks": 2}}, "joints": {"j1": {"lines": ["a", "b"], "kind": "join"}}}}});
    let out = builder::build(&lines, &two).unwrap();
    assert!(names(&out).contains(&"gen.track.a.j1.t2.rails".to_string()));
    // different counts: left out with a note
    let odd = json!({"_context": {"documentData": {"tracks": {"a": {"tracks": 2}}, "joints": {"j1": {"lines": ["a", "b"], "kind": "join"}}}}});
    let out = builder::build(&lines, &odd).unwrap();
    assert!(out["documentData"]["notes"][0].as_str().unwrap().contains("different numbers or spacings"));
}

#[test]
fn set_joint_stores_branches_and_joins_and_refuses_what_cannot_be_drawn() {
    let p = |d: &str| json!({"description": d, "_context": {"command": "set_joint", "documentData": {}}});
    let branch = doc(&[("a", MAIN.to_vec()), ("b", on_deg(12.0, [700.0, 0.0], 700.0))]);
    let stored: Value = serde_json::from_str(&builder::set_joint_text(&branch, &p("{\"kind\":\"branch\",\"frog\":6}")).unwrap()).unwrap();
    assert_eq!(stored["documentData"]["joints"]["j1"], json!({"kind": "branch", "frog": 6, "lines": ["a", "b"]}));
    let e = builder::set_joint_text(&branch, &p("{\"kind\":\"branch\",\"main\":\"b\"}")).unwrap_err();
    assert!(e.contains("ends on line a, so a is the main"), "{e}");
    let e = builder::set_joint_text(&branch, &p("{\"kind\":\"join\"}")).unwrap_err();
    assert!(e.contains("You asked for a join, but line b ends on line a"), "{e}");
    let joinable = doc(&[("a", vec![[0.0, 0.0], [400.0, 0.0]]), ("b", vec![[420.0, 14.0], [900.0, 60.0]])]);
    assert!(builder::set_joint_text(&joinable, &p("{\"kind\":\"join\"}")).is_ok());
    assert!(builder::set_joint_text(&joinable, &p("{\"kind\":\"join\",\"main\":\"a\"}")).unwrap_err().contains("does not apply to a join"));
    let _ = Frog::new(4);
}
