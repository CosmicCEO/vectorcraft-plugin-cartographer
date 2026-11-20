mod common;
use serde_json::{Value, json};
use cartographer_core::rail_builder as builder;
use cartographer_core::rail_standards::*;

fn input(pts: &[[f64; 2]]) -> Value {
    let anchors: Vec<Value> = pts.iter().map(|p| json!({"p": p})).collect();
    json!({"objects": [{"id": 1, "name": "track.a", "path": {"subpaths": [{"anchors": anchors, "closed": false}]}}]})
}

fn rails(out: &Value) -> Vec<Value> {
    out["objects"].as_array().unwrap().iter().find(|o| o["name"] == "gen.track.a.t1.rails").unwrap()["path"]["subpaths"].as_array().unwrap().clone()
}

fn pt(v: &Value) -> [f64; 2] {
    [v[0].as_f64().unwrap() / PT_PER_MM, v[1].as_f64().unwrap() / PT_PER_MM]
}

/// Points along the cubic Bezier segments of a subpath, millimetres (document y is down; the circle test does not care).
fn sample(sub: &Value, per_segment: usize) -> Vec<[f64; 2]> {
    let a = sub["anchors"].as_array().unwrap();
    let mut out = Vec::new();
    for w in a.windows(2) {
        let (p0, p3) = (pt(&w[0]["p"]), pt(&w[1]["p"]));
        let c1 = if w[0]["out"].is_null() { p0 } else { pt(&w[0]["out"]) };
        let c2 = if w[1]["in"].is_null() { p3 } else { pt(&w[1]["in"]) };
        for k in 0..=per_segment {
            let t = k as f64 / per_segment as f64;
            let u = 1.0 - t;
            let b = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
            out.push([b[0] * p0[0] + b[1] * c1[0] + b[2] * c2[0] + b[3] * p3[0], b[0] * p0[1] + b[1] * c1[1] + b[2] * c2[1] + b[3] * p3[1]]);
        }
    }
    out
}

#[test]
fn a_curved_line_gives_true_curves_close_to_the_exact_offset() {
    // A quarter circle of radius 300 mm about (0, 300), drawn as 160 points (document points, y down).
    let r = 300.0;
    let line: Vec<[f64; 2]> = (0..=160).map(|i| { let a = i as f64 * std::f64::consts::FRAC_PI_2 / 160.0; [r * a.sin() * PT_PER_MM, (r - r * a.cos()) * PT_PER_MM] }).collect();
    let params = json!({"_context": {"documentData": {"tracks": {"a": {"tracks": 1, "ballast": false, "ties": false}}}}});
    let out = builder::build(&input(&line), &params).unwrap();
    let subs = rails(&out);
    for (sub, want) in subs.iter().zip([r + GAUGE / 2.0, r - GAUGE / 2.0]) {
        let anchors = sub["anchors"].as_array().unwrap();
        assert!(anchors.len() < 80, "thinned: {} anchors", anchors.len());
        assert!(anchors.iter().any(|a| !a["out"].is_null()), "has Bezier handles");
        let worst = sample(sub, 40).iter().map(|p| ((p[0]).hypot(p[1] - r) - want).abs()).fold(0.0, f64::max);
        assert!(worst < 0.03, "rail is {worst} mm off the exact offset circle");
    }
}

#[test]
fn a_real_corner_stays_sharp() {
    let line = [[0.0, 0.0], [100.0 * PT_PER_MM, 0.0], [100.0 * PT_PER_MM, 100.0 * PT_PER_MM]];
    let params = json!({"_context": {"documentData": {"tracks": {"a": {"tracks": 1, "ballast": false, "ties": false}}}}});
    let out = builder::build(&input(&line), &params).unwrap();
    for sub in rails(&out) {
        let a = sub["anchors"].as_array().unwrap();
        assert_eq!(a.len(), 3);
        assert!(a[1]["in"].is_null() && a[1]["out"].is_null(), "the corner has no handles");
    }
}

#[test]
fn spurs_on_a_bezier_line_have_evenly_spaced_ties() {
    // The drawn Bezier is flattened to a polyline; a plain offset of it would jump by millimetres at every vertex on the outer spurs.
    let pt = PT_PER_MM;
    let anchors = vec![json!({"p": [20.0 * pt, 200.0 * pt], "out": [220.0 * pt, 200.0 * pt]}), json!({"p": [620.0 * pt, 80.0 * pt], "in": [420.0 * pt, 80.0 * pt]})];
    let input = json!({"objects": [{"id": 1, "name": "track.a", "path": {"subpaths": [{"anchors": anchors, "closed": false}]}}]});
    let params = json!({"_context": {"documentData": {"yards": {"a": {"mains": 1, "left": 3, "ends": "stub"}}}}});
    let out = builder::build(&input, &params).unwrap();
    for name in ["m1", "L1", "L2", "L3"] {
        let ties = out["objects"].as_array().unwrap().iter().find(|o| o["name"] == format!("gen.track.a.{name}.ties")).unwrap();
        let mids: Vec<[f64; 2]> = common::tie_segments(ties).iter().map(|t| [(t[0][0] + t[1][0]) / 2.0 / pt, (t[0][1] + t[1][1]) / 2.0 / pt]).collect();
        assert!(mids.len() > 40, "{name}");
        for w in mids.windows(2) {
            let d = ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt();
            assert!((2.8..=3.8).contains(&d), "{name}: tie spacing {d} mm (a jump means the offset opened a gap)");
        }
    }
}
