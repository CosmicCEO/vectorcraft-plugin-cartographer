use serde_json::{Value, json};
use cartographer_core::rail_builder as builder;
use cartographer_core::rail_standards::*;
use cartographer_core::turnout::Frog;
use cartographer_core::yard::YardSpec;

fn input(name: &str, pts: &[[f64; 2]]) -> Value {
    let anchors: Vec<Value> = pts.iter().map(|p| json!({"p": p})).collect();
    json!({"objects": [{"id": 1, "name": name, "path": {"subpaths": [{"anchors": anchors, "closed": false}]}}]})
}

/// A straight line along +x of `mm` millimetres from the origin (document points, y down).
fn line(mm: f64) -> Value {
    input("track.a", &[[0.0, 0.0], [mm * PT_PER_MM, 0.0]])
}

fn yard(spec: Value) -> Value {
    json!({"_context": {"documentData": {"yards": {"a": spec}}}})
}

fn object<'a>(out: &'a Value, name: &str) -> Option<&'a Value> {
    out["objects"].as_array().unwrap().iter().find(|o| o["name"] == name)
}

/// Points of subpath `i` in millimetres, y UP (so "left of an eastward line" is positive).
fn sub(o: &Value, i: usize) -> Vec<[f64; 2]> {
    o["path"]["subpaths"][i]["anchors"].as_array().unwrap().iter().map(|a| [a["p"][0].as_f64().unwrap() / PT_PER_MM, -a["p"][1].as_f64().unwrap() / PT_PER_MM]).collect()
}

/// Centre of the track at its first or last point: the middle of its two rails.
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

/// Where spur j leaves a ladder of frog `f` (x along the line, y from the outer main) and how much line `n` spurs need, worked out from the
/// turnout's own numbers: the closing lead gains q sideways by the time it is parallel, so the ladder is j spacings out less q.
fn toe(f: &Frog, j: usize) -> [f64; 2] {
    let e = f.end();
    [e[0] + (j as f64 * TRACK_CENTERS - f.q() - e[1]) / f.alpha.tan(), j as f64 * TRACK_CENTERS - f.q()]
}

fn ladder_len(f: &Frog, n: usize) -> f64 {
    let e = f.end();
    toe(f, n)[0] + e[0] * f.alpha.cos() + e[1] * f.alpha.sin()
}

#[test]
fn frog_number_six_geometry() {
    let f = Frog::new(6);
    assert!(near(f.alpha.to_degrees(), 9.527, 1e-9));
    assert!(near(f.radius, 18.704 * 25.4, 1e-9)); // RP-12.35 line (11)
    assert!(near(f.frog_point(), 3.544 * 25.4, 1e-9)); // line (8)
    let e = f.end();
    assert!(near(f.lateral(e[0]), e[1], 1e-9));
    assert!(near(f.lateral(f.u_at(7.0)), 7.0, 1e-9) && near(f.lateral(f.u_at(20.0)), 20.0, 1e-9));
}

#[test]
fn one_main_one_left_spur_stub() {
    let f = Frog::new(6);
    let out = builder::build(&line(600.0), &yard(json!({"mains": 1, "left": 1}))).unwrap();
    assert!(object(&out, "gen.track.a.m1.rails").is_some() && object(&out, "gen.track.a.L1.rails").is_some());
    // The spur is the ladder: it ends parallel to the main, one spacing to the left, at the end of the yard.
    let end = centre(&out, "L1", true);
    assert!(near(end[0], 600.0, 1e-3) && near(end[1], TRACK_CENTERS, 0.01), "{end:?}");
    // It starts at the point of switch on the main.
    let start = centre(&out, "L1", false);
    assert!(near(start[0], 0.0, 0.3) && near(start[1], 0.0, 0.01), "{start:?}");
    assert!(near(YardSpec::from_json(&json!({"left": 1})).unwrap().min_length(), ladder_len(&f, 1), 1e-9));
}

#[test]
fn ladder_toes_are_spaced_one_track_apart_along_the_ladder() {
    let f = Frog::new(6);
    let out = builder::build(&line(2500.0), &yard(json!({"mains": 1, "left": 3}))).unwrap();
    for j in 1..3usize {
        let at = centre(&out, &format!("L{j}"), false);
        let [x, y] = toe(&f, j);
        assert!(near(at[0], x, 0.5) && near(at[1], y, 0.02), "L{j}: {at:?} want ({x}, {y})");
        // and each ends parallel at j spacings
        assert!(near(centre(&out, &format!("L{j}"), true)[1], j as f64 * TRACK_CENTERS, 0.01));
    }
    assert!(near(centre(&out, "L3", true)[1], 3.0 * TRACK_CENTERS, 0.01));
}

#[test]
fn two_mains_eight_left_four_right_merge_back() {
    let f = Frog::new(6);
    let spec = json!({"mains": 2, "left": 8, "right": 4, "ends": "merge"});
    let need = 2.0 * ladder_len(&f, 8);
    assert!(near(YardSpec::from_json(&spec).unwrap().min_length(), need, 1e-9));
    assert!(near(YardSpec::from_json(&spec).unwrap().turnouts() as f64, 2.0 * 12.0, 0.0));

    let out = builder::build(&line(need + 200.0), &yard(spec.clone())).unwrap();
    // The only thing to say is that the ties of the long routes are drawn as strips (see `ties::MAX_TIES`).
    assert!(out["documentData"]["notes"].as_array().unwrap().iter().all(|n| n.as_str().unwrap().contains("solid strip")), "{}", out["documentData"]["notes"]);
    let names: Vec<String> = out["objects"].as_array().unwrap().iter().filter_map(|o| o["name"].as_str().map(str::to_string)).collect();
    for r in ["m1", "m2"].into_iter().map(String::from).chain((1..=8).map(|j| format!("L{j}"))).chain((1..=4).map(|j| format!("R{j}"))) {
        assert!(names.contains(&format!("gen.track.a.{r}.rails")) && names.contains(&format!("gen.track.a.{r}.ties")), "{r}");
    }
    // Mains sit half a spacing either side of the drawn line; spur 1 on the left is one spacing outside the left main.
    let half = TRACK_CENTERS / 2.0;
    assert!(near(centre(&out, "m1", true)[1], half, 0.01) && near(centre(&out, "m2", true)[1], -half, 0.01));
    // The merging spurs end on the mirrored ladder: the outermost left spur finishes back on the left main at the far end.
    let end = centre(&out, "L8", true);
    assert!(near(end[0], need + 200.0, 0.5) && near(end[1], half, 0.02), "{end:?}");
    assert!(near(centre(&out, "R4", true)[1], -half, 0.02));
    // In the middle of the yard the spurs are parallel, one spacing apart, 8 on the left and 4 on the right.
    let rails = |r: &str| sub(object(&out, &format!("gen.track.a.{r}.rails")).unwrap(), 0);
    let mid = (need + 200.0) / 2.0;
    let y_at = |r: &str| { let p = rails(r); let i = p.iter().position(|q| q[0] >= mid).unwrap(); p[i][1] - GAUGE / 2.0 }; // subpath 0 is the left rail
    for j in 1..=8 { assert!(near(y_at(&format!("L{j}")), half + j as f64 * TRACK_CENTERS, 0.01), "L{j}"); }
    for j in 1..=4 { assert!(near(y_at(&format!("R{j}")), -half - j as f64 * TRACK_CENTERS, 0.01), "R{j}"); }
}

#[test]
fn too_short_a_line_draws_only_the_mains_and_says_so() {
    let out = builder::build(&line(500.0), &yard(json!({"mains": 2, "left": 8, "right": 4, "ends": "merge"}))).unwrap();
    assert!(object(&out, "gen.track.a.m2.rails").is_some() && object(&out, "gen.track.a.L1.rails").is_none());
    let note = out["documentData"]["notes"][0].as_str().unwrap();
    assert!(note.contains("needs") && note.contains("500"), "{note}");
}

#[test]
fn set_yard_checks_the_selected_line_and_the_words() {
    let p = |d: &str| json!({"description": d, "_context": {"command": "set_yard", "documentData": {"tracks": {"t1": {"tracks": 2}}}}});
    let ok = builder::set_yard_text(&input("track.t1", &[[0.0, 0.0], [3000.0 * PT_PER_MM, 0.0]]), &p("{\"mains\":2,\"left\":8,\"right\":4,\"ends\":\"merge\"}")).unwrap();
    let ok: Value = serde_json::from_str(&ok).unwrap();
    assert_eq!(ok["documentData"]["yards"]["t1"]["left"], 8);
    assert!(ok["documentData"]["tracks"].get("t1").is_none(), "a yard replaces the plain tracks on that line");
    let short = builder::set_yard_text(&input("track.t1", &[[0.0, 0.0], [500.0 * PT_PER_MM, 0.0]]), &p("{\"left\":8,\"ends\":\"merge\"}")).unwrap_err();
    assert!(short.contains("needs") && short.contains("longer"), "{short}");
    for bad in ["{\"mains\":0}", "{\"left\":17}", "{\"ends\":\"loop\"}", "{\"frog\":3}", "{\"frog\":21}", "{\"frog\":6.5}", "{\"colour\":1}", "{\"spacing\":1}", "[1]"] {
        assert!(builder::set_yard_text(&line(2000.0), &p(bad)).is_err(), "{bad}");
    }
}

#[test]
fn oblique_line_keeps_spurs_one_spacing_from_the_main() {
    let d = 1200.0 * PT_PER_MM / 2f64.sqrt();
    let out = builder::build(&input("track.a", &[[0.0, 0.0], [d, d]]), &yard(json!({"mains": 1, "left": 2, "ends": "merge"}))).unwrap();
    // The line runs down-right on the page (y down); "left" of travel is up-right. Distance of spur 2's far end from the main's line is two spacings.
    let end = centre(&out, "L2", true);
    let main_end = centre(&out, "m1", true);
    assert!(near((end[0] - main_end[0]).hypot(end[1] - main_end[1]), 0.0, 0.05), "merged spur ends on the main: {end:?} {main_end:?}");
    let p = sub(object(&out, "gen.track.a.L1.rails").unwrap(), 0); // the left rail of spur 1
    let body = p[p.len() / 2];
    // The main runs down-right on the page, so its left is up-right: (1, 1) / sqrt 2. The left rail is half a gauge further out.
    let lateral = (body[0] + body[1]) / 2f64.sqrt();
    assert!(near(lateral, TRACK_CENTERS + GAUGE / 2.0, 0.05), "{lateral}");
}
