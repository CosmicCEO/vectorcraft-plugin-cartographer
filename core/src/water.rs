//! Water: lakes and ponds (a closed shape), creeks and rivers (an open line with a width) and shores (an open line with water on one side).
//! A line named `water.<id>` is a feature; its description is stored in the document's plug-in data under `waters.<id>`:
//! `{"kind": "flowing", "name": "Cedar Creek", "width": 8, "side": "right", "flow": "fwd", "speed": 0.6, "bank_left": 1.5, "bank_right": 1.5, "slope": 2}`.
//! - `still`: the shape is the water's edge; a bank band of `height x slope` metres lies outside it.
//! - `flowing`: a strip `width` wide along the line, banks each side, arrows along the flow (with the line, against it, or none).
//! - `shore`: the line is the water's edge and the water fills everything on `side` of it (left or right as drawn) out to far beyond the page;
//!   the bank lies on the other side.
//! Bank height and slope (horizontal metres per metre of height) are facts for the simulation as well as the drawing.

use crate::geom::{P, Poly, add, dot, scale, sub};
use crate::offset::{MITRE_LIMIT, offset_polyline};
use crate::output::Out;
use crate::roads::{Spec, flip};
use crate::vcdoc::Item;
use serde_json::{Value, json};

pub const WATER: [f64; 3] = [0.62, 0.78, 0.92];
pub const BANK: [f64; 3] = [0.78, 0.74, 0.58];
const EDGE: [f64; 3] = [0.35, 0.5, 0.65];
const ARROW: [f64; 3] = [0.9, 0.96, 1.0];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Still,
    Flowing,
    Shore,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Flow {
    Fwd,
    Back,
    None,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WaterSpec {
    pub kind: Kind,
    pub name: Option<String>,
    /// Width of a creek or river in metres.
    pub width: f64,
    /// For a shore: the side of the line the water is on, as seen along the drawn direction (-1 left, +1 right).
    pub side: f64,
    pub flow: Flow,
    /// Flow speed in metres per second, if known.
    pub speed: Option<f64>,
    /// Height of each bank above the water, and the slope (metres across per metre of height, H:V).
    pub bank_left: f64,
    pub bank_right: f64,
    pub slope: f64,
}

/// The id of a water line (`water.x` is `x`, plain `water` is `main`), or None.
pub fn water_id(name: &str) -> Option<String> {
    if name == "water" {
        return Some("main".into());
    }
    let rest = name.strip_prefix("water.")?;
    Some(rest.split('.').next().filter(|s| !s.is_empty()).unwrap_or("main").to_string())
}

/// Is this subpath a closed shape (it ends where it began)?
pub fn is_closed(sub: &[P]) -> bool {
    sub.len() >= 4 && (sub[0][0] - sub[sub.len() - 1][0]).hypot(sub[0][1] - sub[sub.len() - 1][1]) < 1e-6
}

impl WaterSpec {
    /// A description for a line when none is stored: a closed shape is still water, an open line a creek.
    pub fn default_for(closed: bool) -> WaterSpec {
        WaterSpec { kind: if closed { Kind::Still } else { Kind::Flowing }, name: None, width: 8.0, side: 1.0, flow: if closed { Flow::None } else { Flow::Fwd }, speed: None, bank_left: 1.5, bank_right: 1.5, slope: 2.0 }
    }

    pub fn from_json(v: &Value) -> Result<WaterSpec, String> {
        let word = |key: &str| v[key].as_str().map(|w| w.trim().to_ascii_lowercase().replace([' ', '_'], "-"));
        let kind = match word("kind").as_deref() {
            Some("still") | Some("lake") | Some("pond") | Some("reservoir") | Some("body") | Some("water-body") => Kind::Still,
            Some("flowing") | Some("river") | Some("creek") | Some("stream") | Some("brook") | Some("canal") | Some("drain") | Some("ditch") | Some("channel") => Kind::Flowing,
            Some("shore") | Some("coast") | Some("coastline") | Some("shoreline") | Some("bank") | Some("beach") => Kind::Shore,
            Some(other) => return Err(format!("kind must be still (lake, pond), flowing (creek, river) or shore (got \"{other}\")")),
            None => return Err("a water feature needs \"kind\": still, flowing or shore".into()),
        };
        let flow = match word("flow").as_deref() {
            None | Some("") => if kind == Kind::Flowing { Flow::Fwd } else { Flow::None },
            Some("fwd") | Some("forward") | Some("with") | Some("along") | Some("down") => Flow::Fwd,
            Some("back") | Some("backward") | Some("against") | Some("reverse") | Some("up") => Flow::Back,
            Some("none") | Some("still") | Some("no") => Flow::None,
            Some(other) => return Err(format!("flow must be fwd, back or none (got \"{other}\")")),
        };
        let side = match word("side").as_deref() {
            None | Some("") | Some("right") => 1.0,
            Some("left") => -1.0,
            Some(other) => return Err(format!("side must be left or right (got \"{other}\")")),
        };
        let both = v["bank"].as_f64();
        let name = v["name"].as_str().map(|n| n.chars().filter(|c| !c.is_control()).take(40).collect::<String>()).filter(|n| !n.trim().is_empty());
        Ok(WaterSpec {
            kind,
            name,
            width: v["width"].as_f64().unwrap_or(8.0).clamp(1.0, 300.0),
            side,
            flow,
            speed: v["speed"].as_f64().map(|s| s.clamp(0.0, 15.0)),
            bank_left: v["bank_left"].as_f64().or(both).unwrap_or(1.5).clamp(0.0, 15.0),
            bank_right: v["bank_right"].as_f64().or(both).unwrap_or(1.5).clamp(0.0, 15.0),
            slope: v["slope"].as_f64().unwrap_or(2.0).clamp(1.0, 6.0),
        })
    }

    pub fn to_json(&self) -> Value {
        json!({
            "kind": match self.kind { Kind::Still => "still", Kind::Flowing => "flowing", Kind::Shore => "shore" },
            "name": self.name, "width": self.width, "side": if self.side < 0.0 { "left" } else { "right" },
            "flow": match self.flow { Flow::Fwd => "fwd", Flow::Back => "back", Flow::None => "none" },
            "speed": self.speed, "bank_left": self.bank_left, "bank_right": self.bank_right, "slope": self.slope
        })
    }

    pub fn summary(&self) -> String {
        let name = self.name.as_ref().map_or(String::new(), |n| format!("{n}: "));
        let banks = if self.bank_left == self.bank_right { format!("banks {} m high at {}:1", self.bank_left, self.slope) } else { format!("banks {} m left and {} m right at {}:1", self.bank_left, self.bank_right, self.slope) };
        match self.kind {
            Kind::Still => format!("{name}lake or pond, {banks}"),
            Kind::Flowing => {
                let flow = match self.flow { Flow::Fwd => "flowing with the line", Flow::Back => "flowing against the line", Flow::None => "no flow shown" };
                format!("{name}creek or river {} m wide, {flow}{}, {banks}", self.width, self.speed.map_or(String::new(), |s| format!(" at {s} m/s")))
            }
            Kind::Shore => format!("{name}shore with the water on the {} of the line, {}", if self.side < 0.0 { "left" } else { "right" }, banks),
        }
    }

    /// The width of the bank band on one side (+1 right of the line, -1 left): bank height times slope.
    pub fn bank_width(&self, side: f64) -> f64 {
        (if side < 0.0 { self.bank_left } else { self.bank_right }) * self.slope
    }

    /// Checks that this kind suits the line (a lake is a closed shape, a river or shore an open line).
    pub fn check_shape(&self, closed: bool) -> Result<(), String> {
        match (self.kind, closed) {
            (Kind::Still, false) => Err("A lake or pond needs a closed shape: close the path (or draw it as a closed shape) first.".into()),
            (Kind::Flowing | Kind::Shore, true) => Err("A creek, river or shore is an open line; this shape is closed.".into()),
            _ => Ok(()),
        }
    }
}

fn unit(v: P) -> P {
    let m = v[0].hypot(v[1]).max(1e-12);
    [v[0] / m, v[1] / m]
}

fn signed_area(r: &[P]) -> f64 {
    (0..r.len()).map(|i| r[i][0] * r[(i + 1) % r.len()][1] - r[(i + 1) % r.len()][0] * r[i][1]).sum::<f64>() / 2.0
}

/// A ring moved outward by `d` metres (inward if negative): each corner is mitred (to at most 3 x `d`).
pub fn offset_ring(ring: &[P], d: f64) -> Vec<P> {
    let n = ring.len();
    if n < 3 {
        return ring.to_vec();
    }
    // The inside of a counter-clockwise ring (y up) is on its left, so outward is to the right; for a clockwise ring it is the other way round.
    let out_sign = if signed_area(ring) > 0.0 { 1.0 } else { -1.0 };
    (0..n)
        .map(|i| {
            let (a, b, c) = (ring[(i + n - 1) % n], ring[i], ring[(i + 1) % n]);
            let (t1, t2) = (unit(sub(b, a)), unit(sub(c, b)));
            let (n1, n2) = (Poly::right(t1), Poly::right(t2));
            let bis = [n1[0] + n2[0], n1[1] + n2[1]];
            let k = 1.0 + dot(n1, n2);
            let m = if k < 0.1 { n1 } else { scale(bis, 1.0 / k) };
            let mag = m[0].hypot(m[1]).min(3.0);
            let dir = if m[0].hypot(m[1]) > 1e-12 { scale(unit(m), mag) } else { n1 };
            add(b, scale(dir, d * out_sign))
        })
        .collect()
}

/// Flow arrows along a centreline: a chevron every `gap` metres pointing with the flow.
fn arrows(path: &[P], flow: Flow, size: f64, gap: f64) -> Vec<Vec<P>> {
    let poly = Poly::new(path.to_vec());
    let mut out = Vec::new();
    let mut s = gap / 2.0;
    while s < poly.len() - size {
        let (p, t) = poly.at(s);
        let d = if flow == Flow::Back { scale(t, -1.0) } else { t };
        let n = Poly::right(d);
        let tip = add(p, scale(d, size));
        out.push(vec![add(sub(p, scale(d, size * 0.4)), scale(n, size * 0.8)), tip, sub(sub(p, scale(d, size * 0.4)), scale(n, size * 0.8))]);
        s += gap;
    }
    out
}

/// Builds every `water.*` line in `items`. `artboard` is the page `[x, y, width, height]` if the plug-in was told it. Returns notes.
/// What a neighbouring flowing line looks like where it meets this one end to end.
#[derive(Clone, Copy)]
struct Joint {
    /// The other line's direction at the joint, turned to run the way this line runs (before its start, or on from its end).
    along: P,
    width: f64,
    bank_left: f64,
    bank_right: f64,
}

struct Job {
    base: String,
    path: Vec<P>,
    ws: WaterSpec,
}

const JOIN_TOLERANCE: f64 = 2.0;

/// Where two flowing lines meet end to end their widths and banks conform: each takes the average of the two at the joint and eases to its own
/// value over `ramp` metres. A line that merely ends in the middle of another (a tributary) is left alone: the water of both is drawn together.
fn joint_at(me: usize, end: bool, jobs: &[Job]) -> Option<Joint> {
    let job = &jobs[me];
    let pt = if end { *job.path.last()? } else { *job.path.first()? };
    for (i, other) in jobs.iter().enumerate() {
        if i == me || other.ws.kind != Kind::Flowing || other.path.len() < 2 {
            continue;
        }
        for (their_end, q) in [(false, other.path[0]), (true, *other.path.last()?)] {
            if (q[0] - pt[0]).hypot(q[1] - pt[1]) > JOIN_TOLERANCE {
                continue;
            }
            // Drawn the same way (my end meets their start, or my start their end) keeps left as left; against each other swaps the sides.
            let swap = their_end == end;
            let n = other.path.len();
            let t_other = if their_end { unit(sub(other.path[n - 1], other.path[n - 2])) } else { unit(sub(other.path[1], other.path[0])) };
            // Same-way joint: my end meets their start (their direction on), or my start their end (their direction before): already my way. Otherwise reversed.
            let along = if swap { scale(t_other, -1.0) } else { t_other };
            let (l, r) = if swap { (other.ws.bank_right, other.ws.bank_left) } else { (other.ws.bank_left, other.ws.bank_right) };
            return Some(Joint { along, width: (job.ws.width + other.ws.width) / 2.0, bank_left: (job.ws.bank_left + l) / 2.0, bank_right: (job.ws.bank_right + r) / 2.0 });
        }
    }
    None
}

/// Draws every water feature in four passes (banks, edges, water, arrows) so that where two meet the water of one is never covered by the bank or
/// edge of the other.
pub fn build(items: &[Item], spec: &Spec, artboard: Option<[f64; 4]>, out: &mut Out) -> Vec<String> {
    let mut notes = Vec::new();
    let mut jobs: Vec<Job> = Vec::new();
    for it in items {
        let Some(id) = water_id(&it.name) else { continue };
        let Some(sub) = it.subs.first() else { continue };
        let closed = is_closed(sub);
        let mut ws = spec.water(&id).unwrap_or_else(|| WaterSpec::default_for(closed));
        // A target print scale gives a creek or river the same minimum print width as a road band, about its own line.
        ws.width = ws.width.max(spec.library.min_width());
        if let Err(e) = ws.check_shape(closed) {
            notes.push(format!("water {id}: {e}"));
            continue;
        }
        jobs.push(Job { base: format!("gen.water.{id}"), path: sub.iter().map(|&p| flip(p)).collect(), ws });
    }
    let joints: Vec<(Option<Joint>, Option<Joint>)> = (0..jobs.len()).map(|i| if jobs[i].ws.kind == Kind::Flowing { (joint_at(i, false, &jobs), joint_at(i, true, &jobs)) } else { (None, None) }).collect();
    for pass in 0..4 {
        for (job, (j0, j1)) in jobs.iter().zip(&joints) {
            match job.ws.kind {
                Kind::Still => still(out, &job.base, &job.path, &job.ws, pass),
                Kind::Flowing => flowing(out, &job.base, &job.path, &job.ws, [*j0, *j1], pass),
                Kind::Shore => shore(out, &job.base, &job.path, &job.ws, artboard, pass),
            }
        }
    }
    notes
}

fn still(out: &mut Out, base: &str, path: &[P], ws: &WaterSpec, pass: u8) {
    let ring: Vec<P> = path[..path.len() - 1].to_vec();
    let bank = ws.bank_width(-1.0).max(ws.bank_width(1.0));
    match pass {
        0 if bank > 0.0 => out.polygon(&format!("{base}.bank"), &offset_ring(&ring, bank), flip, BANK),
        1 => {
            let mut edge = ring.clone();
            edge.push(ring[0]);
            out.path(&format!("{base}.edge"), &[edge], flip, EDGE, EDGE_WIDTH, false);
        }
        2 => out.polygon(&format!("{base}.water"), &ring, flip, WATER),
        _ => {}
    }
}

const EDGE_WIDTH: f64 = 0.4;

/// The strip along `path` with per-station width and banks (used where a joint changes them); `at` gives (water width, left bank width, right bank width)
/// for a station at distance s. Returns (left edge, right edge, left outer, right outer).
fn varying(path: &[P], ends: [Option<P>; 2], at: &dyn Fn(f64) -> (f64, f64, f64)) -> (Vec<P>, Vec<P>, Vec<P>, Vec<P>) {
    let poly = Poly::new(path.to_vec());
    let len = poly.len();
    let steps = (len / 2.0).ceil().max(2.0) as usize;
    let mut stations: Vec<f64> = (0..=steps).map(|i| len * i as f64 / steps as f64).collect();
    let mut acc = 0.0;
    for w in path.windows(2) {
        acc += (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]);
        if acc > 0.0 && acc < len {
            stations.push(acc);
        }
    }
    stations.sort_by(f64::total_cmp);
    stations.dedup_by(|a, b| (*a - *b).abs() < 0.05);
    let (mut l, mut r, mut lo, mut ro) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for s in stations {
        // At a bend the offset follows the bisector, lengthened (up to 2.5 x) so the edge stays parallel to both sides: no gap on the inside.
        // At a joint the cut is the bisector of this line's direction and the other line's, so both lines end on the same cut.
        let (p, _) = poly.at(s);
        let (mut n1, mut n2) = (Poly::right(poly.at((s - 0.01).max(0.0)).1), Poly::right(poly.at((s + 0.01).min(len)).1));
        if s <= 0.0 {
            if let Some(t) = ends[0] {
                n1 = Poly::right(t);
            }
        }
        if s >= len {
            if let Some(t) = ends[1] {
                n2 = Poly::right(t);
            }
        }
        let bis = unit(add(n1, n2));
        let n = scale(bis, (1.0 / dot(bis, if s >= len { n1 } else { n2 }).max(0.4)).min(2.5));
        let (w, bl, br) = at(s);
        l.push(add(p, scale(n, -w / 2.0)));
        r.push(add(p, scale(n, w / 2.0)));
        lo.push(add(p, scale(n, -(w / 2.0 + bl))));
        ro.push(add(p, scale(n, w / 2.0 + br)));
    }
    (l, r, lo, ro)
}

fn flowing(out: &mut Out, base: &str, path: &[P], ws: &WaterSpec, joints: [Option<Joint>; 2], pass: u8) {
    let (l, r, lo, ro) = if joints.iter().any(Option::is_some) {
        let len = Poly::new(path.to_vec()).len();
        let ramp = (len / 2.0).min(40.0).max(1.0);
        let (bl0, br0) = (ws.bank_width(-1.0), ws.bank_width(1.0));
        let blend = |s: f64| -> (f64, f64, f64) {
            let (mut w, mut bl, mut br) = (ws.width, bl0, br0);
            for (j, d) in [(joints[0], s), (joints[1], len - s)] {
                if let Some(j) = j {
                    let f = (1.0 - d / ramp).clamp(0.0, 1.0);
                    w += (j.width - ws.width) * f;
                    bl += (j.bank_left * ws.slope - bl0) * f;
                    br += (j.bank_right * ws.slope - br0) * f;
                }
            }
            (w, bl.max(0.0), br.max(0.0))
        };
        varying(path, [joints[0].map(|j| j.along), joints[1].map(|j| j.along)], &blend)
    } else {
        let half = ws.width / 2.0;
        let (l, r) = (offset_polyline(path, -half, MITRE_LIMIT), offset_polyline(path, half, MITRE_LIMIT));
        let lo = if ws.bank_width(-1.0) > 0.0 { offset_polyline(path, -(half + ws.bank_width(-1.0)), MITRE_LIMIT) } else { Vec::new() };
        let ro = if ws.bank_width(1.0) > 0.0 { offset_polyline(path, half + ws.bank_width(1.0), MITRE_LIMIT) } else { Vec::new() };
        (l, r, lo, ro)
    };
    if l.len() < 2 || r.len() < 2 {
        return;
    }
    match pass {
        0 => {
            for (edge, outer, name) in [(&l, &lo, "bank.left"), (&r, &ro, "bank.right")] {
                if outer.len() >= 2 && (ws.bank_width(if name == "bank.left" { -1.0 } else { 1.0 }) > 0.0 || joints.iter().any(Option::is_some)) {
                    let mut ring = edge.clone();
                    ring.extend(outer.iter().rev());
                    out.polygon(&format!("{base}.{name}"), &ring, flip, BANK);
                }
            }
        }
        1 => out.path(&format!("{base}.edge"), &[l, r], flip, EDGE, EDGE_WIDTH, false),
        2 => {
            let mut ring = l.clone();
            ring.extend(r.iter().rev());
            out.polygon(&format!("{base}.water"), &ring, flip, WATER);
        }
        _ => {
            if ws.flow != Flow::None {
                let a = arrows(path, ws.flow, (ws.width * 0.35).clamp(1.0, 6.0), (ws.width * 6.0).clamp(30.0, 120.0));
                if !a.is_empty() {
                    out.path(&format!("{base}.flow"), &a, flip, ARROW, 0.4, false);
                }
            }
        }
    }
}

fn shore(out: &mut Out, base: &str, path: &[P], ws: &WaterSpec, artboard: Option<[f64; 4]>, pass: u8) {
    let n = path.len();
    if n < 2 {
        return;
    }
    // The water side: right of the drawn direction for +1, left for -1. Far enough to cover the page whatever its size.
    let far = artboard.map_or(5000.0, |a| (a[2] + a[3]) * 3.0 + 500.0).max(500.0);
    let wn = |t: P| scale(Poly::right(t), ws.side);
    let (t0, t1) = (unit(sub(path[1], path[0])), unit(sub(path[n - 1], path[n - 2])));
    // The bank is on the land side: opposite the water.
    let land = -ws.side;
    match pass {
        0 => {
            let w = ws.bank_width(land);
            if w > 0.0 {
                let outer = offset_polyline(path, land * w, MITRE_LIMIT);
                if outer.len() >= 2 {
                    let mut band = path.to_vec();
                    band.extend(outer.iter().rev());
                    out.polygon(&format!("{base}.bank"), &band, flip, BANK);
                }
            }
        }
        1 => out.path(&format!("{base}.edge"), &[path.to_vec()], flip, EDGE, EDGE_WIDTH, false),
        2 => {
            let mut ring: Vec<P> = vec![sub(path[0], scale(t0, far))];
            ring.extend_from_slice(path);
            ring.push(add(path[n - 1], scale(t1, far)));
            ring.push(add(add(path[n - 1], scale(t1, far)), scale(wn(t1), far)));
            ring.push(add(sub(path[0], scale(t0, far)), scale(wn(t0), far)));
            out.polygon(&format!("{base}.water"), &ring, flip, WATER);
        }
        _ => {}
    }
}
