//! Shared by the rail tests: the ties a build shows. A route's ties are one dashed line since 2.0 (see `ties::runs`); these helpers turn
//! such a line back into one segment per tie, so the tests check tie positions as before.
#![allow(dead_code)]

use cartographer_core::rail_standards::PT_PER_MM;
use cartographer_core::ties;
use serde_json::Value;

type P = [f64; 2];

/// A subpath as a polyline, its Bezier segments flattened finely.
pub fn flatten(sub: &Value) -> Vec<P> {
    let a = sub["anchors"].as_array().unwrap();
    let pt = |v: &Value| -> Option<P> { Some([v.get(0)?.as_f64()?, v.get(1)?.as_f64()?]) };
    let mut out = vec![pt(&a[0]["p"]).unwrap()];
    for w in a.windows(2) {
        let (p0, p3) = (pt(&w[0]["p"]).unwrap(), pt(&w[1]["p"]).unwrap());
        let (p1, p2) = (pt(&w[0]["out"]).unwrap_or(p0), pt(&w[1]["in"]).unwrap_or(p3));
        let n = if p1 == p0 && p2 == p3 { 1 } else { 64 };
        for k in 1..=n {
            let t = k as f64 / n as f64;
            let u = 1.0 - t;
            let b = |i: usize| u * u * u * p0[i] + 3.0 * u * u * t * p1[i] + 3.0 * u * t * t * p2[i] + t * t * t * p3[i];
            out.push([b(0), b(1)]);
        }
    }
    out
}

/// The tie segments of one object: a tie line (`*.ties`) expanded dash by dash, any other object's subpaths as they are.
pub fn tie_segments(o: &Value) -> Vec<[P; 2]> {
    let subs = o["path"]["subpaths"].as_array().cloned().unwrap_or_default();
    if o["name"].as_str().is_some_and(|n| n.ends_with(".ties")) {
        subs.iter().flat_map(|s| ties::expand(&flatten(s), PT_PER_MM)).collect()
    } else {
        subs.iter().map(|s| { let f = flatten(s); [f[0], *f.last().unwrap()] }).collect()
    }
}

/// Every point of every tie of the objects whose name starts with `prefix` and contains "ties".
pub fn tie_points(objs: &[Value], prefix: &str) -> Vec<P> {
    objs.iter().filter(|o| o["name"].as_str().is_some_and(|n| n.starts_with(prefix) && n.contains("ties"))).flat_map(tie_segments).flat_map(|s| s).collect()
}
