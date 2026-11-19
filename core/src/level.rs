//! Level crossings: a road line and a track line that cross (the design's "road meets track"). The road stays on the ground and the track goes
//! over it: the track's ties are left out under the road (`avoid_bands`, the same mechanism as a track crossing another track), a grey crossing
//! panel is laid between the ballast and the rails, and the rails run across. `gated` adds a crossbuck and a gate arm on each approach.
//! Two grade separations use the same record: `road over` (a road bridge: the road's surface is laid over the track, with a parapet on each
//! side) and `track over` (a rail bridge: the track runs on over the road with all its ties, with a parapet on each side of it).
//! Stored as `level_crossings: {id: {road, track, treatment}}`; set with the command Make Level Crossing (`set_level_crossing`). Objects are named
//! `gen.track.<track>.level.<id>.*`.

use crate::crosssection::{layout, parse};
use crate::geom::P;
use crate::joint::Band;
use crate::output::{Layer, Out};
use crate::rail_builder::{crossing_half_mm, to_mm, track_id};
use crate::rail_standards::PT_PER_MM;
use crate::roads::{Spec, segment_id};
use crate::vcdoc::Item;
use serde_json::{Map, Value, json};
use std::collections::HashMap;

pub const TREATMENTS: [&str; 4] = ["level", "gated", "road over", "track over"];
const PANEL: [f64; 3] = [0.72, 0.72, 0.72];
/// Crossbucks are dark so they show on a white page (the road is not under them: they stand beside it).
const SIGN: [f64; 3] = [0.15, 0.15, 0.15];
const RED: [f64; 3] = [0.85, 0.1, 0.1];
/// Room left beyond the road edge before ties start again, metres.
const MARGIN: f64 = 0.5;

/// Reads a description: `{"treatment": "level" | "gated" | "road over" | "track over"}` (empty is level). Returns the treatment.
pub fn parse_treatment(v: &Value) -> Result<String, String> {
    let Some(obj) = v.as_object() else { return Err("the level crossing description must be a JSON object".into()) };
    if let Some(bad) = obj.keys().find(|k| k.as_str() != "treatment") {
        return Err(format!("unknown level crossing field `{bad}`; the only field is treatment ({})", TREATMENTS.join(" or ")));
    }
    let t = obj.get("treatment").and_then(Value::as_str).unwrap_or("level");
    if TREATMENTS.contains(&t) { Ok(t.to_string()) } else { Err(format!("treatment must be {} (\"{t}\" is not)", TREATMENTS.join(" or "))) }
}

/// Where two lines (document points) first cross, and the direction of each there.
pub struct Found {
    pub at: P,
    pub road_dir: P,
    pub track_dir: P,
}

pub fn cross_point(road: &[P], track: &[P]) -> Option<Found> {
    let unit = |a: P, b: P| {
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let l = (dx * dx + dy * dy).sqrt();
        if l > 0.0 { [dx / l, dy / l] } else { [1.0, 0.0] }
    };
    for r in road.windows(2) {
        for t in track.windows(2) {
            let (d1, d2) = ([r[1][0] - r[0][0], r[1][1] - r[0][1]], [t[1][0] - t[0][0], t[1][1] - t[0][1]]);
            let den = d1[0] * d2[1] - d1[1] * d2[0];
            if den.abs() < 1e-12 {
                continue;
            }
            let e = [t[0][0] - r[0][0], t[0][1] - r[0][1]];
            let (u, v) = ((e[0] * d2[1] - e[1] * d2[0]) / den, (e[0] * d1[1] - e[1] * d1[0]) / den);
            if (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v) {
                return Some(Found { at: [r[0][0] + u * d1[0], r[0][1] + u * d1[1]], road_dir: unit(r[0], r[1]), track_dir: unit(t[0], t[1]) });
            }
        }
    }
    None
}

/// Where a road's cross-section lies across its line, metres: the lowest and highest offset, measured to the right of travel along the line (a road
/// anchored at one edge lies wholly on one side). `line` is the piece in document points; its general heading decides how a compass-facing
/// cross-section is read, as the road code does.
fn road_extent(spec: &Spec, road: &str, line: &[P]) -> Option<(f64, f64, Vec<(f64, f64, [f64; 3])>)> {
    let (text, anchor) = spec.segments.get(road)?;
    let parsed = parse(text, &spec.library).ok()?;
    let (a, b) = (line.first()?, line.last()?);
    let (dx, dy) = (b[0] - a[0], a[1] - b[1]); // world frame: y up
    let l = (dx * dx + dy * dy).sqrt().max(1e-9);
    let placed = layout(&parsed, &spec.library, [dx / l, dy / l], anchor.clone()).ok()?;
    let lo = placed.iter().map(|e| e.left).fold(f64::INFINITY, f64::min);
    let hi = placed.iter().map(|e| e.right).fold(f64::NEG_INFINITY, f64::max);
    (lo.is_finite() && hi.is_finite()).then(|| (lo, hi, placed.iter().map(|e| (e.left, e.right, e.fill)).collect()))
}

/// The line moved sideways by `d` (to the right of travel, document points), vertex by vertex along the mean of the neighbouring normals.
fn shifted(line: &[P], d: f64) -> Vec<P> {
    let normal = |a: P, b: P| {
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let l = (dx * dx + dy * dy).sqrt().max(1e-9);
        [-dy / l, dx / l]
    };
    (0..line.len())
        .map(|i| {
            let (prev, next) = (line[i.saturating_sub(1)], line[(i + 1).min(line.len() - 1)]);
            let n = normal(if i == 0 { line[0] } else { prev }, if i + 1 == line.len() { line[i] } else { next });
            [line[i][0] + n[0] * d, line[i][1] + n[1] * d]
        })
        .collect()
}

struct Job {
    id: String,
    track: String,
    treatment: String,
    road_line: Vec<P>,
    found: Found,
    /// The road's extent across its line, metres to the right of travel.
    lo: f64,
    hi: f64,
    /// The road's elements across its line (left, right, colour), for the deck of a road bridge.
    strips: Vec<(f64, f64, [f64; 3])>,
}

fn lines<'a>(items: &'a [Item], pick: impl Fn(&str) -> bool) -> Vec<&'a Vec<P>> {
    items.iter().filter(|i| pick(&i.name)).filter_map(|i| i.subs.first()).filter(|s| s.len() >= 2).collect()
}

/// The first piece of the road and line of the track that cross (a road may be drawn as several pieces).
pub fn find_crossing<'a>(roads: &[&'a Vec<P>], tracks: &[&Vec<P>]) -> Option<(&'a Vec<P>, Found)> {
    roads.iter().find_map(|r| tracks.iter().find_map(|t| cross_point(r, t)).map(|f| (*r, f)))
}

fn jobs(items: &[Item], spec: &Spec, notes: &mut Vec<String>) -> Vec<Job> {
    let mut out = Vec::new();
    for (id, rec) in spec.level_crossings() {
        let (road, track) = (rec["road"].as_str().unwrap_or(""), rec["track"].as_str().unwrap_or(""));
        let (rls, tls) = (lines(items, |n| segment_id(n).as_deref() == Some(road)), lines(items, |n| track_id(n).as_deref() == Some(track)));
        if rls.is_empty() || tls.is_empty() {
            notes.push(format!("Level crossing {id}: road {road} or track {track} is not in the document; left out."));
            continue;
        }
        let Some((rl, found)) = find_crossing(&rls, &tls) else {
            notes.push(format!("Level crossing {id}: road {road} and track {track} no longer cross; left out."));
            continue;
        };
        let Some((lo, hi, strips)) = road_extent(spec, road, rl) else {
            notes.push(format!("Level crossing {id}: road {road} has no cross-section yet; left out."));
            continue;
        };
        out.push(Job { id, track: track.to_string(), treatment: rec["treatment"].as_str().unwrap_or("level").to_string(), road_line: rl.clone(), found, lo, hi, strips });
    }
    out
}

/// For each track, the strips (model millimetres, as the rail code works) where its ties are left out because a road crosses it.
pub fn avoid_bands(items: &[Item], spec: &Spec) -> HashMap<String, Vec<Band>> {
    let mut bands: HashMap<String, Vec<Band>> = HashMap::new();
    for job in jobs(items, spec, &mut Vec::new()).into_iter().filter(|j| j.treatment == "level" || j.treatment == "gated") {
        // The strip is the road itself: its middle may lie off the drawn line (a road anchored at one edge).
        let pts: Vec<P> = shifted(&job.road_line, (job.lo + job.hi) / 2.0).iter().map(|&p| to_mm(p)).collect();
        bands.entry(job.track).or_default().push(Band::new(pts, ((job.hi - job.lo) / 2.0 + MARGIN) / PT_PER_MM));
    }
    bands
}

/// Draws every stored level crossing (after the tracks: the panel goes on the ties stack, above every ballast, below the rails). Returns notes.
pub fn build(items: &[Item], spec: &Spec, data: &Map<String, Value>, out: &mut Out) -> Vec<String> {
    let mut notes = Vec::new();
    for job in jobs(items, spec, &mut notes) {
        let (rd, td) = (job.found.road_dir, job.found.track_dir);
        let (nr, nt) = ([-rd[1], rd[0]], [-td[1], td[0]]);
        let det = nr[0] * nt[1] - nr[1] * nt[0];
        if det.abs() < 0.17 {
            notes.push(format!("Level crossing {}: the track runs nearly along the road; left out.", job.id));
            continue;
        }
        let (near, far, ht) = (job.lo - MARGIN, job.hi + MARGIN, crossing_half_mm(data, &job.track) * PT_PER_MM + 0.3);
        let x = job.found.at;
        // The panel is the parallelogram where the road strip and the track strip overlap.
        let corner = |a: f64, b: f64| [x[0] + (a * nt[1] - nr[1] * b) / det, x[1] + (nr[0] * b - a * nt[0]) / det];
        let ring = vec![corner(far, ht), corner(far, -ht), corner(near, -ht), corner(near, ht)];
        // Named under the track, so the panel lives in the track layer (between its ballast and its rails).
        let base = format!("gen.track.{}.level.{}", job.track, job.id);
        let line = |a: (f64, f64), b: (f64, f64)| vec![corner(a.0, a.1), corner(b.0, b.1)];
        if job.treatment == "road over" {
            // Above everything on the track: the road's own surface, element by element, then a parapet along each edge.
            out.layer(Layer::Rails);
            let hb = ht + 1.0;
            for (k, (l, r, fill)) in job.strips.iter().enumerate() {
                out.polygon(&format!("{base}.deck.{k}"), &[corner(*r, hb), corner(*r, -hb), corner(*l, -hb), corner(*l, hb)], |p| p, *fill);
            }
            for (tag, a) in [("left", job.lo - 0.3), ("right", job.hi + 0.3)] {
                out.path(&format!("{base}.bridge.{tag}"), &[line((a, -hb), (a, hb))], |p| p, SIGN, 0.3, false);
            }
            out.layer(Layer::Roadbed);
            continue;
        }
        if job.treatment == "track over" {
            // The track is already drawn over the road with every tie; a parapet runs along each side of it across the road.
            out.layer(Layer::Rails);
            for (tag, b) in [("left", -(ht + 0.2)), ("right", ht + 0.2)] {
                out.path(&format!("{base}.bridge.{tag}"), &[line((near - 1.0, b), (far + 1.0, b))], |p| p, SIGN, 0.3, false);
            }
            out.layer(Layer::Roadbed);
            continue;
        }
        out.layer(Layer::Ties);
        out.polygon(&format!("{base}.panel"), &ring, |p| p, PANEL);
        if job.treatment == "gated" {
            out.layer(Layer::Rails); // signs and gates stand above everything on the track
            let clear = ht / det.abs() + 3.0;
            // Right-hand traffic: a driver approaching from behind the crossing (heading along the road line) has the gate on the right of
            // travel (`hi`); one approaching from ahead has it on the other side. The arm reaches across that driver's lanes to the road's line.
            for (sign, edge, tag) in [(-1.0, job.hi + 1.0, "before"), (1.0, job.lo - 1.0, "after")] {
                let foot = [x[0] + rd[0] * sign * clear, x[1] + rd[1] * sign * clear];
                let pole = [foot[0] + nr[0] * edge, foot[1] + nr[1] * edge];
                let arm: Vec<P> = [0.7_f64, -0.7].iter().map(|a| a.sin_cos()).flat_map(|(s, c)| [[pole[0] - 0.7 * s, pole[1] - 0.7 * c], [pole[0] + 0.7 * s, pole[1] + 0.7 * c]]).collect();
                out.path(&format!("{base}.crossbuck.{tag}"), &[arm[0..2].to_vec(), arm[2..4].to_vec()], |p| p, SIGN, 0.25, false);
                out.path(&format!("{base}.gate.{tag}"), &[vec![pole, foot]], |p| p, RED, 0.2, false);
            }
        }
        out.layer(Layer::Roadbed);
    }
    notes
}

/// The stored record for a level crossing.
pub fn record(road: &str, track: &str, treatment: &str) -> Value {
    json!({"road": road, "track": track, "treatment": treatment})
}

/// Notes for every road line and track line that cross with no level crossing stored for them (the track is then simply drawn over the road).
pub fn missing_notes(items: &[Item], spec: &Spec) -> Vec<String> {
    let mut roads: Vec<String> = items.iter().filter_map(|i| segment_id(&i.name)).collect();
    let mut tracks: Vec<String> = items.iter().filter_map(|i| track_id(&i.name)).collect();
    roads.sort();
    roads.dedup();
    tracks.sort();
    tracks.dedup();
    let stored = spec.level_crossings();
    let mut notes = Vec::new();
    for t in &tracks {
        for r in &roads {
            if stored.iter().any(|(_, v)| v["road"] == r.as_str() && v["track"] == t.as_str()) {
                continue;
            }
            let (rls, tls) = (lines(items, |n| segment_id(n).as_deref() == Some(r.as_str())), lines(items, |n| track_id(n).as_deref() == Some(t.as_str())));
            if find_crossing(&rls, &tls).is_some() {
                notes.push(format!("track {t} crosses road {r} with no level crossing: select both and make one (Make Level Crossing), or the track is drawn over the road."));
            }
        }
    }
    notes
}

