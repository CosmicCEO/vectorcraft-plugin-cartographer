//! A yard from a description: mains along the drawn line, a ladder of turnouts off the outer main on each side, spurs running parallel,
//! and optionally a mirrored ladder that merges the spurs back to the mains at the far end.
//!
//! Geometry is worked out in a local frame (x along the drawn line, y to its left, mm) with the left side's formulas; the right side is the
//! same picture reflected. Each spur is one centre line; its rails are offset from it. The outermost spur is the ladder itself.

use crate::geom::Len2;
use crate::joint::Band;
use crate::geom::{P, Poly};
use crate::offset::{MITRE_LIMIT, offset_polyline_raw as offset_polyline};
use crate::output::{Layer, Out};
use crate::rail_standards::*;
use crate::ties;
use crate::tracks::{min_radius, parse_length, parse_spacing};
use crate::turnout::{Frog, Pen};
use serde_json::{Value, json};

pub const MAX_MAINS: usize = 4;
pub const MAX_SPURS: usize = 16;
pub const MAX_CROSSOVERS: usize = 8;
const BALLAST_MARGIN: f64 = 1.0;
/// Gap between two neighbouring ties' ends where a spur's own ties take over from the shared ones.
const TIE_GAP: f64 = 0.8;
const RAIL_RGB: [f64; 3] = [0.17, 0.17, 0.2];
const TIE_RGB: [f64; 3] = [0.42, 0.29, 0.18];
const BALLAST_RGB: [f64; 3] = [0.82, 0.8, 0.76];

#[derive(Clone, Debug, PartialEq)]
pub struct YardSpec {
    pub mains: usize,
    /// Spurs on the left of the drawn line, as seen travelling from its start to its end.
    pub left: usize,
    pub right: usize,
    /// true: a mirrored ladder merges the spurs back to the mains; false: the spurs end at the end of the yard.
    pub merge: bool,
    pub frog: u32,
    pub spacing: f64,
    /// Where the first turnout's toe is, mm along the drawn line.
    pub at: f64,
    /// How far the yard runs from `at`, mm; None: to the end of the drawn line.
    pub length: Option<f64>,
    /// How far before the end of the drawn line the yard stops, mm (0: at the end). Not with `length`.
    pub before_end: f64,
    /// Crossovers between neighbouring mains.
    pub crossovers: Vec<Crossover>,
    pub ballast: bool,
    pub ties: bool,
}

/// A crossover: two turnouts joined by a straight run, taking trains from main `from` to the neighbouring main `to` as the line is followed.
#[derive(Clone, Debug, PartialEq)]
pub struct Crossover {
    /// Where its first point of switch is, mm along the drawn line; None: in the middle of the yard.
    pub at: Option<f64>,
    /// Mains are numbered 1 from the left (as seen travelling from the line's start to its end).
    pub from: usize,
    pub to: usize,
}

impl Default for YardSpec {
    fn default() -> YardSpec {
        YardSpec { mains: 1, left: 0, right: 0, merge: false, frog: 6, spacing: TRACK_CENTERS, at: 0.0, length: None, before_end: 0.0, crossovers: vec![], ballast: true, ties: true }
    }
}

impl YardSpec {
    /// Half the width of the strip the mains (and their ties) cover, mm. A crossing is expected on the mains, not on a spur.
    pub fn footprint_half(&self) -> f64 {
        (self.mains as f64 - 1.0) / 2.0 * self.spacing + TIE_LENGTH / 2.0
    }

    pub fn from_json(v: &Value) -> Result<YardSpec, String> {
        let obj = v.as_object().ok_or("The yard description must be a JSON object, for example {\"mains\": 2, \"left\": 8, \"right\": 4, \"ends\": \"merge\"}.")?;
        let mut s = YardSpec::default();
        let count = |val: &Value, name: &str, lo: usize, hi: usize| -> Result<usize, String> {
            val.as_u64().filter(|n| (lo as u64..=hi as u64).contains(n)).map(|n| n as usize).ok_or(format!("`{name}` must be a whole number from {lo} to {hi}."))
        };
        for (k, val) in obj {
            match k.as_str() {
                "mains" => s.mains = count(val, "mains", 1, MAX_MAINS)?,
                "left" => s.left = count(val, "left", 0, MAX_SPURS)?,
                "right" => s.right = count(val, "right", 0, MAX_SPURS)?,
                "ends" => {
                    s.merge = match val.as_str().map(str::to_lowercase).as_deref() {
                        Some("merge") => true,
                        Some("stub") => false,
                        _ => return Err("`ends` must be \"merge\" (the spurs join the mains again) or \"stub\" (the spurs end).".into()),
                    }
                }
                "frog" => {
                    s.frog = val.as_u64().filter(|n| (Frog::MIN as u64..=Frog::MAX as u64).contains(n)).ok_or("`frog` is the frog number, a whole number from 4 to 20 (the numbers NMRA RP-12.35 lists; 6 is typical for a yard).")? as u32;
                }
                "spacing" => {
                    s.spacing = parse_spacing(val)?;
                    if !(s.spacing > TIE_LENGTH + TIE_GAP) {
                        return Err(format!("`spacing` must be more than {:.1} mm, or neighbouring tracks overlap.", TIE_LENGTH + TIE_GAP));
                    }
                }
                "at" => s.at = parse_length(val, "at").and_then(|x| if x >= 0.0 { Ok(x) } else { Err("`at` cannot be negative.".to_string()) })?,
                "length" => {
                    let l = parse_length(val, "length")?;
                    if l <= 0.0 {
                        return Err("`length` must be more than 0.".into());
                    }
                    s.length = Some(l);
                }
                "crossovers" => {
                    let list = val.as_array().ok_or("`crossovers` must be a list like [{\"from\": 1, \"to\": 2}].")?;
                    if list.len() > MAX_CROSSOVERS {
                        return Err(format!("At most {MAX_CROSSOVERS} crossovers."));
                    }
                    s.crossovers = list.iter().map(Crossover::from_json).collect::<Result<_, _>>()?;
                }
                "ballast" => s.ballast = val.as_bool().ok_or("`ballast` must be true or false.")?,
                "ties" => s.ties = val.as_bool().ok_or("`ties` must be true or false.")?,
                "before_end" => s.before_end = parse_length(val, "before_end").and_then(|x| if x >= 0.0 { Ok(x) } else { Err("`before_end` cannot be negative.".to_string()) })?,
                other => return Err(format!("Unknown yard field `{other}`. Use mains, left, right, ends, frog, spacing, at, length, before_end, crossovers, ballast, ties.")),
            }
        }
        for c in &s.crossovers {
            if c.from > s.mains || c.to > s.mains {
                return Err(format!("A crossover joins mains {} and {}, but there {} only {} main{}.", c.from, c.to, if s.mains == 1 { "is" } else { "are" }, s.mains, if s.mains == 1 { "" } else { "s" }));
            }
        }
        if s.length.is_some() && s.before_end > 0.0 {
            return Err("Give the yard's `length` or `before_end` (how far before the end of the line it stops), not both.".into());
        }
        if !s.crossovers.is_empty() && Frog::new(s.frog).crossover_run(s.spacing).is_none() {
            return Err(format!("A No. {} crossover needs more than {:.1} mm between tracks (its two leads would overlap).", s.frog, 2.0 * Frog::new(s.frog).end()[1] + 1.0));
        }
        Ok(s)
    }

    pub fn to_json(&self) -> Value {
        let r = |x: f64| (x * 1000.0).round() / 1000.0;
        let mut v = json!({"mains": self.mains, "left": self.left, "right": self.right, "ends": if self.merge { "merge" } else { "stub" }, "frog": self.frog, "spacing": r(self.spacing), "at": r(self.at), "ballast": self.ballast, "ties": self.ties});
        if let Some(l) = self.length {
            v["length"] = json!(r(l));
        }
        if self.before_end > 0.0 {
            v["before_end"] = json!(r(self.before_end));
        }
        if !self.crossovers.is_empty() {
            v["crossovers"] = json!(self.crossovers.iter().map(|c| match c.at {
                Some(at) => json!({"from": c.from, "to": c.to, "at": r(at)}),
                None => json!({"from": c.from, "to": c.to}),
            }).collect::<Vec<_>>());
        }
        v
    }

    pub fn turnouts(&self) -> usize {
        let one = |n: usize| if n == 0 { 0 } else if self.merge { 2 * n } else { n };
        one(self.left) + one(self.right) + 2 * self.crossovers.len()
    }

    pub fn summary(&self) -> String {
        let cross = match self.crossovers.len() {
            0 => String::new(),
            1 => ", 1 crossover".to_string(),
            n => format!(", {n} crossovers"),
        };
        format!(
            "{} main{}, {} spur{} left, {} spur{} right, {}{cross}; {} turnouts (No. {})",
            self.mains, if self.mains == 1 { "" } else { "s" },
            self.left, if self.left == 1 { "" } else { "s" },
            self.right, if self.right == 1 { "" } else { "s" },
            if self.merge { "merging back at the far end" } else { "ending in the yard" },
            self.turnouts(), self.frog
        )
    }

    /// How much line one crossover takes along the line, mm (0 if there are none).
    pub fn crossover_extent(&self) -> f64 {
        let f = Frog::new(self.frog);
        f.crossover(0.0, 0.0, 1.0, self.spacing).and_then(|p| p.last().map(|e| e[0])).unwrap_or(0.0)
    }

    /// Why a crossover cannot go where it is asked to on a line `total` mm long, or None if they all fit.
    pub fn crossover_problem(&self, total: f64) -> Option<String> {
        let ext = self.crossover_extent();
        for c in &self.crossovers {
            if ext > total || c.at.is_some_and(|at| at + ext > total) {
                return Some(format!("A crossover between mains {} and {} needs {:.0} mm of line{}, and the line is {total:.0} mm long.", c.from, c.to, ext + c.at.unwrap_or(0.0), c.at.map_or(String::new(), |at| format!(" counting from its start (the crossover begins {at:.0} mm along)"))));
            }
        }
        None
    }

    /// The most line any part needs from the start of the line, mm: the ladders from `at`, and each crossover where it was put.
    pub fn min_line(&self) -> f64 {
        let ext = self.crossover_extent();
        let crossings = self.crossovers.iter().map(|c| c.at.unwrap_or(0.0) + ext).fold(0.0, f64::max);
        crossings.max(if self.left + self.right > 0 { self.at + self.min_length() } else { 0.0 })
    }

    /// Length of line the yard needs from `at`, mm: the ladders (twice, if they merge back) with no body between them.
    pub fn min_length(&self) -> f64 {
        let f = Frog::new(self.frog);
        let e = f.end();
        let ladder = |n: usize| if n == 0 { 0.0 } else { e[0] + (n as f64 * self.spacing - f.q() - e[1]) / f.alpha.tan() + e[0] * f.alpha.cos() + e[1] * f.alpha.sin() };
        let x = ladder(self.left).max(ladder(self.right));
        if self.merge { 2.0 * x } else { x }
    }
}

impl Crossover {
    fn from_json(v: &Value) -> Result<Crossover, String> {
        let obj = v.as_object().ok_or("Each crossover is an object like {\"from\": 1, \"to\": 2, \"at\": \"1m\"}.")?;
        let (mut at, mut from, mut to) = (None, None, None);
        for (k, val) in obj {
            let main = |name: &str| val.as_u64().filter(|n| (1..=MAX_MAINS as u64).contains(n)).map(|n| n as usize).ok_or(format!("a crossover's `{name}` is a main track number, 1 to {MAX_MAINS} counting from the left."));
            match k.as_str() {
                "at" => at = Some(parse_length(val, "at").and_then(|x| if x >= 0.0 { Ok(x) } else { Err("a crossover's `at` cannot be negative.".to_string()) })?),
                "from" => from = Some(main("from")?),
                "to" => to = Some(main("to")?),
                other => return Err(format!("Unknown crossover field `{other}`. Use from, to, at.")),
            }
        }
        let (from, to) = (from.ok_or("A crossover needs `from` and `to` (neighbouring mains, counting from the left).")?, to.ok_or("A crossover needs `from` and `to` (neighbouring mains, counting from the left).")?);
        if from.abs_diff(to) != 1 {
            return Err(format!("A crossover joins neighbouring mains; {from} and {to} are not neighbours."));
        }
        Ok(Crossover { at, from, to })
    }
}

/// A turnout that leaves one of the line's mains at a place (a branch to another line, see `branch`): the main gets lengthened switch ties and
/// the roadbed between it and the branch for the stretch where they are close.
#[derive(Clone, Copy, Debug)]
pub struct Tap {
    /// Where the point of switch is, mm along the line.
    pub x0: f64,
    /// +1 if the branch runs the way the line runs, -1 if it runs back along it.
    pub dir: f64,
    /// Which main it leaves, counting from 0 at the left.
    pub main: usize,
    /// Which side of the line's direction of travel the branch goes: +1 left, -1 right.
    pub side: f64,
    pub frog: Frog,
}

/// A turnout off a parent route: its ties are lengthened over the stretch where the branch is closer than a tie length.
#[derive(Clone, Copy)]
struct Branch {
    /// Arc length along the parent where the toe is.
    s_toe: f64,
    /// +1 if the branch runs the same way as the parent, -1 if it runs back along it.
    dir: f64,
    /// Which side of the parent's direction of travel the branch is on: +1 left, -1 right.
    side: f64,
    frog: Frog,
}

struct Route {
    name: String,
    pts: Vec<P>,
    /// Arc lengths between which the route's own ties go (the rest is covered by its parent's lengthened ties).
    tie_from: f64,
    tie_to: f64,
    branches: Vec<Branch>,
}

/// Maps the local frame onto the drawn line (mm, y up).
pub(crate) struct Mapper {
    pub(crate) straight: bool,
    /// For a straight line: origin and the x and y axes of the local frame in drawing coordinates (worked out once; this runs per point).
    o: P,
    ex: P,
    ey: P,
    /// For a curved line: the point and left-hand normal every `STEP` mm along it, interpolated per point (a search per point is too dear).
    table: Vec<(P, P)>,
    len: f64,
    /// How far apart `dense` places points on a curved line, mm: a tenth of the tightest radius, 4 to 80 (the Bezier fit joins them).
    step: f64,
}

/// Spacing of the lookup table along a curved line, mm.
const STEP: f64 = 2.0;

impl Mapper {
    pub(crate) fn new(line: &[P]) -> Mapper {
        let poly = Poly::new(line.to_vec());
        let len = poly.len();
        let straight = poly.pts.len() == 2;
        if straight {
            let (a, b) = (poly.pts[0], poly.pts[1]);
            let ex = [(b[0] - a[0]) / len, (b[1] - a[1]) / len];
            // Local y is to the left of the line.
            return Mapper { straight, o: a, ex, ey: [-ex[1], ex[0]], table: vec![], len, step: 1e9 };
        }
        // The drawn curve arrives flattened to a polyline, and a plain offset of a polyline opens a gap at every vertex (3 mm and more on a
        // spur 70 mm out). So the normal is the bisector at each vertex, turned smoothly between vertices.
        let (pts, cum) = (&poly.pts, crate::ties::arc_lengths(&poly.pts));
        let seg_normal = |i: usize| -> P {
            let (a, b) = (pts[i], pts[i + 1]);
            let l = (b[0] - a[0]).len2(b[1] - a[1]).max(1e-12);
            [-(b[1] - a[1]) / l, (b[0] - a[0]) / l]
        };
        let unit = |v: P| -> P {
            let l = v[0].len2(v[1]).max(1e-12);
            [v[0] / l, v[1] / l]
        };
        let last = pts.len() - 1;
        let vn: Vec<P> = (0..=last)
            .map(|i| match (i.checked_sub(1), i < last) {
                (Some(j), true) => { let (a, b) = (seg_normal(j), seg_normal(i)); unit([a[0] + b[0], a[1] + b[1]]) }
                (None, _) => seg_normal(0),
                (Some(j), false) => seg_normal(j),
            })
            .collect();
        let n = (len / STEP).ceil() as usize + 1;
        let mut i = 0;
        let table = (0..=n)
            .map(|k| {
                let s = (k as f64 * STEP).min(len);
                while i + 1 < last && cum[i + 1] < s {
                    i += 1;
                }
                let span = (cum[i + 1] - cum[i]).max(1e-12);
                let f = ((s - cum[i]) / span).clamp(0.0, 1.0);
                let q = [pts[i][0] + (pts[i + 1][0] - pts[i][0]) * f, pts[i][1] + (pts[i + 1][1] - pts[i][1]) * f];
                (q, unit([vn[i][0] + (vn[i + 1][0] - vn[i][0]) * f, vn[i][1] + (vn[i + 1][1] - vn[i][1]) * f]))
            })
            .collect();
        let step = (min_radius(line).unwrap_or(1e9) * 0.1).clamp(4.0, 80.0);
        Mapper { straight, o: [0.0, 0.0], ex: [1.0, 0.0], ey: [0.0, 1.0], table, len, step }
    }

    pub(crate) fn map(&self, p: P) -> P {
        if self.straight {
            [self.o[0] + self.ex[0] * p[0] + self.ey[0] * p[1], self.o[1] + self.ex[1] * p[0] + self.ey[1] * p[1]]
        } else {
            let s = p[0].clamp(0.0, self.len) / STEP;
            let k = (s as usize).min(self.table.len() - 2);
            let f = s - k as f64;
            let ((a, na), (b, nb)) = (self.table[k], self.table[k + 1]);
            let (q, n) = ([a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f], [na[0] + (nb[0] - na[0]) * f, na[1] + (nb[1] - na[1]) * f]);
            [q[0] + n[0] * p[1], q[1] + n[1] * p[1]]
        }
    }

    /// Breaks long segments so that a bend in the drawn line shows in the result (not needed on a straight line).
    pub(crate) fn dense(&self, pts: &[P]) -> Vec<P> {
        if self.straight {
            return pts.to_vec();
        }
        let mut out: Vec<P> = Vec::with_capacity(pts.len());
        for &p in pts {
            if let Some(&q) = out.last() {
                let n = ((p[0] - q[0]).len2(p[1] - q[1]) / self.step).ceil() as usize;
                for k in 1..n {
                    let t = k as f64 / n as f64;
                    out.push([q[0] + (p[0] - q[0]) * t, q[1] + (p[1] - q[1]) * t]);
                }
            }
            out.push(p);
        }
        out
    }
}

fn arc_len(pts: &[P]) -> Vec<f64> {
    let mut cum = vec![0.0];
    for w in pts.windows(2) {
        cum.push(cum.last().unwrap() + (w[1][0] - w[0][0]).len2(w[1][1] - w[0][1]));
    }
    cum
}

/// Arc length at which `pts` first lies `dist` or more from the line through `t0` with unit direction `dir`.
fn leaves(pts: &[P], t0: P, dir: P, dist: f64) -> f64 {
    let cum = arc_len(pts);
    let off = |p: &P| ((p[0] - t0[0]) * dir[1] - (p[1] - t0[1]) * dir[0]).abs();
    for (i, p) in pts.iter().enumerate() {
        let d = off(p);
        if d >= dist {
            // Inside the segment that crosses the distance (a ladder's straight run is one long segment): the exact place, not its far end.
            if i > 0 {
                let d0 = off(&pts[i - 1]);
                if d > d0 {
                    return cum[i - 1] + (cum[i] - cum[i - 1]) * (dist - d0) / (d - d0);
                }
            }
            return cum[i];
        }
    }
    *cum.last().unwrap_or(&0.0)
}

/// Twice the signed area of a ring (positive: counter-clockwise).
fn winding(ring: &[P]) -> f64 {
    ring.iter().zip(ring.iter().cycle().skip(1)).map(|(a, b)| a[0] * b[1] - b[0] * a[1]).sum()
}

fn reflect(sigma: f64, p: P) -> P {
    [p[0], p[1] * sigma]
}

/// The lengthened part of the ties near each branch: for every tie close to a branch, the piece beyond a normal tie's end on the branch's
/// side (the normal ties are the dashed tie line, `ties::runs`). Only the ties inside a branch's zone are looked at.
/// A route laid on a curved line: its unrolled (model) arc lengths and its drawn ones at the same points, to go from one to the other. The
/// dash of a tie line runs along the drawn curve, which is longer or shorter than the model by the curve, so ties are placed along it.
struct Along {
    u: Vec<f64>,
    m: Vec<f64>,
    pts: Vec<P>,
}

impl Along {
    fn new(mapper: &Mapper, route: &[P]) -> Along {
        let d = mapper.dense(route);
        let pts: Vec<P> = d.iter().map(|&p| mapper.map(p)).collect();
        Along { u: ties::arc_lengths(&d), m: ties::arc_lengths(&pts), pts }
    }

    fn lerp(xs: &[f64], ys: &[f64], x: f64) -> f64 {
        let i = xs.partition_point(|&v| v < x).clamp(1, xs.len() - 1);
        let span = xs[i] - xs[i - 1];
        ys[i - 1] + if span > 0.0 { (x - xs[i - 1]) / span * (ys[i] - ys[i - 1]) } else { 0.0 }
    }

    fn drawn(&self, u: f64) -> f64 {
        Along::lerp(&self.u, &self.m, u)
    }

    fn model(&self, m: f64) -> f64 {
        Along::lerp(&self.m, &self.u, m)
    }

    /// A run of ties (unrolled edges `a`..`b`) laid along the drawn curve as a whole number of ties from the same first edge.
    fn run(&self, (a, b): (f64, f64)) -> (f64, f64) {
        let (ma, mb) = (self.drawn(a), self.drawn(b));
        // The nearest whole number of ties, but never a tie past the end of the route.
        let end = *self.m.last().unwrap_or(&mb);
        let mut n = ((mb - ma - TIE_WIDTH) / TIE_PITCH).round().max(0.0);
        if n > 0.0 && ma + TIE_WIDTH + n * TIE_PITCH > end + 1e-9 {
            n -= 1.0;
        }
        (ma, ma + TIE_WIDTH + n * TIE_PITCH)
    }

    /// The model arc length of the tie a lengthened switch tie at `s` belongs to: the dash nearest to it along the drawn curve.
    fn snap(&self, spans: &[(f64, f64)], s: f64) -> f64 {
        let Some(&(a, b)) = spans.iter().find(|(a, b)| s >= *a && s <= *b) else { return s };
        let (ma, mb) = self.run((a, b));
        let last = ((mb - ma - TIE_WIDTH) / TIE_PITCH).round();
        let k = ((self.drawn(s) - ma - TIE_WIDTH / 2.0) / TIE_PITCH).round().clamp(0.0, last);
        self.model(ma + TIE_WIDTH / 2.0 + k * TIE_PITCH)
    }
}

fn extensions(r: &Route) -> Vec<Vec<P>> {
    extensions_at(r, &|s| s)
}

/// `extensions`, each lengthened tie put at `place(s)` along the route (on a curved line: on its dash).
fn extensions_at(r: &Route, place: &dyn Fn(f64) -> f64) -> Vec<Vec<P>> {
    let poly = Poly::new(r.pts.clone());
    let half = TIE_LENGTH / 2.0;
    let n = ties::count(r.tie_from, r.tie_to) as f64;
    let mut out = Vec::new();
    for b in &r.branches {
        let zone = b.frog.u_at(TIE_LENGTH + TIE_GAP);
        let (lo_s, hi_s) = if b.dir > 0.0 { (b.s_toe, b.s_toe + zone) } else { (b.s_toe - zone, b.s_toe) };
        let k_of = |s: f64| (s - r.tie_from - TIE_PITCH / 2.0) / TIE_PITCH;
        let (k0, k1) = (k_of(lo_s).ceil().max(0.0), k_of(hi_s).floor().min(n - 1.0));
        let mut k = k0;
        while k <= k1 {
            let s = r.tie_from + TIE_PITCH / 2.0 + k * TIE_PITCH;
            let u = b.dir * (s - b.s_toe);
            let lat = b.frog.lateral(u);
            if u > 0.0 && lat < TIE_LENGTH + TIE_GAP && lat > 0.0 {
                let (p, t) = poly.at(place(s));
                let nrm = [-t[1], t[0]];
                let (from, to) = if b.side > 0.0 { (half, lat + half) } else { (-half, -(lat + half)) };
                out.push(vec![[p[0] + nrm[0] * from, p[1] + nrm[1] * from], [p[0] + nrm[0] * to, p[1] + nrm[1] * to]]);
            }
            k += 1.0;
        }
    }
    out
}

/// Everything one side's ladder makes: its routes, and the branches it adds to the outer main.
struct Side {
    routes: Vec<Route>,
    /// Branch off the outer main (front, and back if merging), to add to that main's route.
    main_branches: Vec<Branch>,
}

#[allow(clippy::too_many_arguments)]
fn side(sigma: f64, n: usize, y_out: f64, a0: f64, a_end: f64, spec: &YardSpec, label: &str) -> Side {
    let f = Frog::new(spec.frog);
    let (e, q, alpha) = (f.end(), f.q(), f.alpha);
    let sp = spec.spacing;
    let mirror = |p: P| [a0 + a_end - p[0], p[1]];
    // Spur j leaves the ladder where the ladder is j spacings out, less what the closing curve itself gains (q), then runs parallel.
    let x_toe = |j: usize| a0 + e[0] + (j as f64 * sp - q - e[1]) / alpha.tan();
    let y_toe = |j: usize| y_out + j as f64 * sp - q;

    // The ladder (and the outermost spur): the lead off the main, straight at the frog angle, then the closing lead back to parallel.
    let mut ladder: Vec<P> = vec![[a0, y_out]];
    let mut pen = Pen::new([a0, y_out], 0.0);
    f.diverge(&mut pen, 1.0, &mut ladder);
    pen.line((x_toe(n) - pen.p[0]) / alpha.cos(), &mut ladder);
    f.diverge(&mut pen, -1.0, &mut ladder);

    // Spurs 1..n-1 leave the ladder with the closing arc.
    let mut fronts: Vec<(usize, Vec<P>)> = Vec::new();
    for j in 1..n {
        let mut pts = vec![[x_toe(j), y_toe(j)]];
        f.diverge(&mut Pen::new(pts[0], alpha), -1.0, &mut pts);
        fronts.push((j, pts));
    }

    let finish = |front: &[P], y_end: f64| -> Vec<P> {
        let mut pts = front.to_vec();
        if spec.merge {
            pts.extend(front.iter().rev().map(|&p| mirror(p)));
        } else {
            pts.push([a_end, y_end]);
        }
        pts
    };
    let ladder_full = finish(&ladder, y_out + n as f64 * sp);
    let ladder_poly = Poly::new(ladder_full.iter().map(|&p| reflect(sigma, p)).collect());

    let ladder_dir = [alpha.cos(), alpha.sin()];
    let mut ladder_branches: Vec<Branch> = Vec::new();
    let mut routes: Vec<Route> = Vec::new();
    for (j, front) in &fronts {
        let toe = front[0];
        let y_end = y_out + *j as f64 * sp;
        let pts = finish(front, y_end);
        // Measured along the whole spur: its closing curve gains less than a tie length on the ladder, the parallel run that follows does the rest.
        let from = leaves(&pts, toe, ladder_dir, TIE_LENGTH + TIE_GAP);
        let len = *arc_len(&pts).last().unwrap();
        let to = if spec.merge { len - from } else { len };
        let s_front = ladder_poly.project(reflect(sigma, toe)).0;
        ladder_branches.push(Branch { s_toe: s_front, dir: 1.0, side: -sigma, frog: f });
        if spec.merge {
            let mt = mirror(toe);
            ladder_branches.push(Branch { s_toe: ladder_poly.project(reflect(sigma, mt)).0, dir: -1.0, side: -sigma, frog: f });
        }
        routes.push(Route { name: format!("{label}{j}"), pts: pts.iter().map(|&p| reflect(sigma, p)).collect(), tie_from: from, tie_to: to, branches: vec![] });
    }
    // The ladder is the outermost spur: its ties start where it has left the main by a tie length.
    let from = leaves(&ladder, [a0, y_out], [1.0, 0.0], TIE_LENGTH + TIE_GAP);
    let len = *arc_len(&ladder_full).last().unwrap();
    let to = if spec.merge { len - from } else { len };
    routes.push(Route { name: format!("{label}{n}"), pts: ladder_poly.pts.clone(), tie_from: from, tie_to: to, branches: ladder_branches });
    routes.sort_by(|a, b| a.name.cmp(&b.name));

    let mut main_branches = vec![Branch { s_toe: a0, dir: 1.0, side: sigma, frog: f }];
    if spec.merge {
        main_branches.push(Branch { s_toe: a_end, dir: -1.0, side: sigma, frog: f });
    }
    Side { routes, main_branches }
}

/// Draws the yard on `line` (mm, y up); returns plain-words notes (it draws only the mains if the line is too short).
/// `avoid`: strips where another track runs through, in which this yard's ties are left out (see `joint`). `taps`: turnouts to branches.
pub fn build(id: &str, line: &[P], spec: &YardSpec, out: &mut Out, to_doc: impl Fn(P) -> P + Copy, avoid: &[Band], taps: &[Tap]) -> Vec<String> {
    let mut notes = Vec::new();
    let mapper = Mapper::new(line);
    let total = mapper.len;
    if total <= 0.0 {
        return notes;
    }
    let a0 = spec.at;
    let a_end = a0 + spec.length.unwrap_or(total - spec.before_end - a0);
    let (mut spec, mut a_end) = (spec.clone(), a_end);
    let need = spec.min_length();
    if a0 + need > total + 1e-6 || a_end > total + 1e-6 || a_end - a0 < need - 1e-6 {
        notes.push(format!("Track {id}: this yard needs {need:.0} mm of line from {a0:.0} mm, but only {:.0} mm is available. Only the mains are drawn. Draw a longer line, use a smaller frog number, or fewer spurs.", (a_end.min(total) - a0).max(0.0)));
        spec.left = 0;
        spec.right = 0;
        a_end = total;
    }
    let f = Frog::new(spec.frog);
    if let Some(r) = min_radius(line) {
        if r < 3.0 * f.radius && (spec.left + spec.right + spec.crossovers.len()) > 0 {
            notes.push(format!("Track {id}: the drawn line curves on {r:.0} mm radius, under three times the {:.0} mm turnout closure radius. NMRA RP-12 says turnouts should not be on the inside of a curve like that.", f.radius));
        }
    }
    let (m, sp) = (spec.mains, spec.spacing);
    let y_out = (m as f64 - 1.0) / 2.0 * sp;
    let mut routes: Vec<Route> = (0..m)
        .map(|k| Route { name: format!("m{}", k + 1), pts: vec![[0.0, y_out - k as f64 * sp], [total, y_out - k as f64 * sp]], tie_from: 0.0, tie_to: total, branches: vec![] })
        .collect();
    for (sigma, n, label, main) in [(1.0, spec.left, "L", 0), (-1.0, spec.right, "R", m - 1)] {
        if n == 0 {
            continue;
        }
        let s = side(sigma, n, y_out, a0, a_end, &spec, label);
        routes[main].branches.extend(s.main_branches);
        routes.extend(s.routes);
    }
    // Crossovers: a route from one main to its neighbour, with the lengthened switch ties on both mains and its own ties between the two.
    let ext = spec.crossover_extent();
    for (k, c) in spec.crossovers.iter().enumerate() {
        let x0 = c.at.unwrap_or(a0 + (a_end - a0) / 2.0 - ext / 2.0);
        let (y_from, y_to) = (y_out - (c.from - 1) as f64 * sp, y_out - (c.to - 1) as f64 * sp);
        let sig = if y_to > y_from { 1.0 } else { -1.0 };
        let Some(pts) = f.crossover(x0, y_from, sig, sp) else { continue };
        if x0 < -1e-6 || x0 + ext > total + 1e-6 {
            notes.push(format!("Track {id}: the crossover between mains {} and {} does not fit on the line ({:.0} mm needed from {x0:.0} mm, the line is {total:.0} mm). It is left out.", c.from, c.to, ext));
            continue;
        }
        let end = *pts.last().unwrap();
        routes[c.from - 1].branches.push(Branch { s_toe: x0, dir: 1.0, side: sig, frog: f });
        routes[c.to - 1].branches.push(Branch { s_toe: end[0], dir: -1.0, side: -sig, frog: f });
        let len = *arc_len(&pts).last().unwrap();
        let from = leaves(&pts, pts[0], [1.0, 0.0], TIE_LENGTH + TIE_GAP);
        let back: Vec<P> = pts.iter().rev().copied().collect();
        let to = len - leaves(&back, end, [1.0, 0.0], TIE_LENGTH + TIE_GAP);
        routes.push(Route { name: format!("X{}", k + 1), pts, tie_from: from, tie_to: to, branches: vec![] });
    }

    // Branches to other lines: switch ties on the main, and a wedge of roadbed between the main and the branch.
    let mut wedges: Vec<Vec<P>> = Vec::new();
    let route_count = routes.len();
    for t in taps.iter().filter(|t| t.main < route_count) {
        let y = routes[t.main].pts[0][1];
        routes[t.main].branches.push(Branch { s_toe: t.x0, dir: t.dir, side: t.side, frog: t.frog });
        let reach = TIE_LENGTH / 2.0 + BALLAST_MARGIN;
        let u_max = t.frog.u_at(2.0 * reach);
        let at = |u: f64, lat: f64| [t.x0 + t.dir * u, y + t.side * lat];
        let steps = 12;
        let mut ring: Vec<P> = (0..=steps).map(|i| at(u_max * i as f64 / steps as f64, 0.0)).collect();
        ring.extend((0..=steps).rev().map(|i| { let u = u_max * i as f64 / steps as f64; at(u, t.frog.lateral(u)) }));
        wedges.push(ring);
    }
    // Roadbed: a band under the mains, and for a yard one outline from the outer edge of the outermost left spur to that of the outermost
    // right spur (or the outer main on a side with no spurs), closed across both ends. It is made in the straight local frame, then bent
    // onto the drawn line.
    let reach = TIE_LENGTH / 2.0 + BALLAST_MARGIN;
    if spec.ballast {
        out.layer(Layer::Roadbed);
        let mut rings: Vec<Vec<P>> = Vec::new();
        // One band across all the mains, from the outer edge of the leftmost to that of the rightmost.
        let (l, rt) = (offset_polyline(&routes[0].pts, -reach, MITRE_LIMIT), offset_polyline(&routes[m - 1].pts, reach, MITRE_LIMIT));
        rings.push(mapper.dense(&l).into_iter().chain(mapper.dense(&rt).into_iter().rev()).collect());
        if spec.left + spec.right > 0 {
            let pick = |name: String, fallback: usize, off: f64| -> Vec<P> {
                let pts = routes.iter().find(|r| r.name == name).map_or_else(|| vec![[a0, routes[fallback].pts[0][1]], [a_end, routes[fallback].pts[0][1]]], |r| r.pts.clone());
                offset_polyline(&pts, off, MITRE_LIMIT)
            };
            let left = pick(format!("L{}", spec.left), 0, -reach);
            let right = pick(format!("R{}", spec.right), m - 1, reach);
            rings.push(mapper.dense(&left).into_iter().chain(mapper.dense(&right).into_iter().rev()).collect());
        }
        rings.extend(wedges.iter().map(|w| mapper.dense(w)));
        // Overlapping shapes in one object must wind the same way, or the fill rule makes holes where they overlap.
        for r in rings.iter_mut() {
            if winding(r) < 0.0 {
                r.reverse();
            }
        }
        out.curve_fills(&format!("gen.track.{id}.ballast"), &rings, |p| to_doc(mapper.map(p)), BALLAST_RGB);
    }
    if spec.ties {
        out.layer(Layer::Ties);
        for r in &routes {
            let keep = |c: P| !avoid.iter().any(|b| b.covers(mapper.map(c)));
            let spans = ties::spans_whole(&r.pts, r.tie_from, r.tie_to, (!avoid.is_empty()).then_some(&keep as &dyn Fn(P) -> bool));
            let name = format!("gen.track.{id}.{}.ties", r.name);
            let along = (!mapper.straight && !spans.is_empty()).then(|| Along::new(&mapper, &r.pts));
            match &along {
                // Straight: the model is the drawing; each run as it is.
                None if !spans.is_empty() => {
                    let runs: Vec<Vec<P>> = spans.iter().flat_map(|&sp| ties::chunks(sp)).map(|(a, b)| ties::slice(&r.pts, a, b)).collect();
                    out.curve(&name, &runs, |p| to_doc(mapper.map(p)), TIE_RGB, TIE_LENGTH * PT_PER_MM);
                }
                None => {}
                // Curved: each run cut from the drawn route, a whole number of ties long along it.
                Some(al) => {
                    let runs: Vec<Vec<P>> = spans.iter().flat_map(|&sp| ties::chunks(al.run(sp))).map(|(a, b)| ties::slice(&al.pts, a, b)).collect();
                    out.curve(&name, &runs, to_doc, TIE_RGB, TIE_LENGTH * PT_PER_MM);
                }
            }
            // The lengthened switch ties give way where another track or a road runs through, like the ordinary ones.
            let mid = |t: &Vec<P>| mapper.map([(t[0][0] + t[1][0]) / 2.0, (t[0][1] + t[1][1]) / 2.0]);
            let placed = match &along { Some(al) => extensions_at(r, &|s| al.snap(&spans, s)), None => extensions(r) };
            let ext: Vec<Vec<P>> = placed.into_iter().filter(|t| avoid.is_empty() || !avoid.iter().any(|b| b.covers(mid(t)))).collect();
            if !ext.is_empty() {
                out.path(&format!("gen.track.{id}.{}.switch-ties", r.name), &ext, |p| to_doc(mapper.map(p)), TIE_RGB, TIE_WIDTH * PT_PER_MM, false);
            }
        }
    }
    out.layer(Layer::Rails);
    for r in &routes {
        let rails = vec![mapper.dense(&offset_polyline(&r.pts, -GAUGE / 2.0, MITRE_LIMIT)), mapper.dense(&offset_polyline(&r.pts, GAUGE / 2.0, MITRE_LIMIT))];
        out.curve(&format!("gen.track.{id}.{}.rails", r.name), &rails, |p| to_doc(mapper.map(p)), RAIL_RGB, RAIL_HEAD * PT_PER_MM);
    }
    notes
}
