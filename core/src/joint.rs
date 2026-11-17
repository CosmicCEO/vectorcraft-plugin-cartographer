//! Joining two drawn lines. A joint is a record naming two lines and the intent (a crossing, later a branch or a join); the meeting point is
//! found again from the drawn lines on every build, so moving a line moves the joint. See docs/superpowers/specs/2026-11-13-joining-lines-design.md.

use crate::geom::{Len2, P, Poly};
use crate::turnout::Frog;
use crate::turnout_table::ROWS;
use serde_json::{Value, json};

/// How close to a line an end must come to count as meeting it, mm (the road builder's value, scaled: 30 m there, 30 mm here).
pub const TOLERANCE: f64 = 30.0;

/// How two drawn lines meet, from their geometry alone (millimetres, y up).
#[derive(Clone, Debug, PartialEq)]
pub enum Found {
    /// The lines cross in the middle of both. `s_a` and `s_b` are the distances along each line to the crossing.
    Crossing { point: P, s_a: f64, s_b: f64, angle_deg: f64 },
    /// One line ends on the other (within `TOLERANCE`): `branch` is 0 if line a is the one that ends, 1 if line b.
    Branch { branch: usize, point: P, s_main: f64, gap: f64, angle_deg: f64 },
    /// The ends of the two lines meet.
    Join { point: P, gap: f64, angle_deg: f64 },
}

/// The point where segments ab and cd cross, as (t along ab, u along cd), if they do.
fn cross_at(a: P, b: P, c: P, d: P) -> Option<(f64, f64)> {
    let (r, s) = ([b[0] - a[0], b[1] - a[1]], [d[0] - c[0], d[1] - c[1]]);
    let den = r[0] * s[1] - r[1] * s[0];
    if den.abs() < 1e-12 {
        return None;
    }
    let ca = [c[0] - a[0], c[1] - a[1]];
    let t = (ca[0] * s[1] - ca[1] * s[0]) / den;
    let u = (ca[0] * r[1] - ca[1] * r[0]) / den;
    ((0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)).then_some((t, u))
}

/// The angle between two directions as lines (0 to 90 degrees).
fn angle_between(t1: P, t2: P) -> f64 {
    let c = (t1[0] * t2[0] + t1[1] * t2[1]).abs().clamp(0.0, 1.0);
    c.acos().to_degrees()
}

/// How the two lines meet, or None if they do not (within `TOLERANCE`).
pub fn find(a: &[P], b: &[P]) -> Option<Found> {
    let (pa, pb) = (Poly::new(a.to_vec()), Poly::new(b.to_vec()));
    if pa.is_empty() || pb.is_empty() {
        return None;
    }
    let (la, lb) = (pa.len(), pb.len());
    // Where they cross, nearest the start of a. A crossing within TOLERANCE of an end of either line is a meeting at an end, not a crossing.
    let mut cum_a = 0.0;
    let mut best: Option<(f64, f64, P)> = None;
    for wa in pa.pts.windows(2) {
        let sa = wa[0].len2_to(wa[1]);
        let mut cum_b = 0.0;
        for wb in pb.pts.windows(2) {
            let sb = wb[0].len2_to(wb[1]);
            if let Some((t, u)) = cross_at(wa[0], wa[1], wb[0], wb[1]) {
                let (s_a, s_b) = (cum_a + t * sa, cum_b + u * sb);
                if best.is_none_or(|(x, _, _)| s_a < x) {
                    best = Some((s_a, s_b, [wa[0][0] + t * (wa[1][0] - wa[0][0]), wa[0][1] + t * (wa[1][1] - wa[0][1])]));
                }
            }
            cum_b += sb;
        }
        cum_a += sa;
    }
    if let Some((s_a, s_b, point)) = best {
        if s_a > TOLERANCE && la - s_a > TOLERANCE && s_b > TOLERANCE && lb - s_b > TOLERANCE {
            let angle_deg = angle_between(pa.at(s_a).1, pb.at(s_b).1);
            return Some(Found::Crossing { point, s_a, s_b, angle_deg });
        }
    }
    // An end of one line on, or at an end of, the other.
    let mut cand: Option<(f64, Found)> = None;
    for (which, x, y) in [(0usize, &pa, &pb), (1, &pb, &pa)] {
        for at_end in [false, true] {
            let e = if at_end { *x.pts.last().unwrap() } else { x.pts[0] };
            let (s, _, dist) = y.project(e);
            if dist > TOLERANCE {
                continue;
            }
            let t_end = if at_end { x.at(x.len()).1 } else { x.at(0.0).1 };
            let angle_deg = angle_between(t_end, y.at(s).1);
            let ends = [y.pts[0], *y.pts.last().unwrap()];
            let to_end = ends.iter().map(|q| e.len2_to(*q)).fold(f64::INFINITY, f64::min);
            let found = if to_end <= TOLERANCE {
                Found::Join { point: e, gap: to_end, angle_deg }
            } else {
                Found::Branch { branch: which, point: y.at(s).0, s_main: s, gap: dist, angle_deg }
            };
            let key = if matches!(found, Found::Join { .. }) { to_end } else { dist };
            if cand.as_ref().is_none_or(|(k, _)| key < *k) {
                cand = Some((key, found));
            }
        }
    }
    cand.map(|(_, f)| f)
}

trait Dist {
    fn len2_to(self, other: P) -> f64;
}

impl Dist for P {
    fn len2_to(self, other: P) -> f64 {
        (other[0] - self[0]).len2(other[1] - self[1])
    }
}

/// The NMRA frog number whose angle is nearest `angle_deg`, with that angle.
pub fn nearest_frog(angle_deg: f64) -> (u32, f64) {
    let r = ROWS.iter().min_by(|x, y| (x.frog_angle_deg - angle_deg).abs().total_cmp(&(y.frog_angle_deg - angle_deg).abs())).unwrap();
    (r.frog, r.frog_angle_deg)
}

impl Found {
    /// What was found, in words, for lines called `a` and `b`.
    pub fn describe(&self, a: &str, b: &str) -> String {
        let name = |i: usize| if i == 0 { a } else { b };
        let frog = |ang: f64| {
            let (n, deg) = nearest_frog(ang);
            if ang > ROWS[0].frog_angle_deg + 1.0 {
                format!("{ang:.1} degrees, wider than any NMRA N frog (No. {n} is {deg:.2})")
            } else {
                format!("{ang:.1} degrees; the nearest NMRA frog is No. {n} ({deg:.2})")
            }
        };
        match self {
            Found::Crossing { s_a, s_b, angle_deg, .. } => format!("lines {a} and {b} cross, {s_a:.0} mm along {a} and {s_b:.0} mm along {b}, at {}", frog(*angle_deg)),
            Found::Branch { branch, s_main, gap, angle_deg, .. } => format!("line {} ends on line {}, {s_main:.0} mm along it ({gap:.1} mm off), at {}", name(*branch), name(1 - branch), frog(*angle_deg)),
            Found::Join { gap, angle_deg, .. } => format!("the ends of lines {a} and {b} meet ({gap:.1} mm apart), {angle_deg:.1} degrees off straight on"),
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Found::Crossing { .. } => "crossing",
            Found::Branch { .. } => "branch",
            Found::Join { .. } => "join",
        }
    }

    pub fn to_json(&self) -> Value {
        match self {
            Found::Crossing { point, s_a, s_b, angle_deg } => json!({"kind": "crossing", "point": point, "s_a": s_a, "s_b": s_b, "angle_deg": angle_deg}),
            Found::Branch { branch, point, s_main, gap, angle_deg } => json!({"kind": "branch", "branch": branch, "point": point, "s_main": s_main, "gap": gap, "angle_deg": angle_deg}),
            Found::Join { point, gap, angle_deg } => json!({"kind": "join", "point": point, "gap": gap, "angle_deg": angle_deg}),
        }
    }
}

/// What kind of joint the owner asked for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Crossing,
    Branch,
    Join,
}

impl Kind {
    pub fn word(self) -> &'static str {
        match self {
            Kind::Crossing => "crossing",
            Kind::Branch => "branch",
            Kind::Join => "join",
        }
    }
}

/// What the owner asked for at a meeting.
#[derive(Clone, Debug, PartialEq)]
pub struct JointSpec {
    pub kind: Kind,
    /// Crossing: the line whose track runs through, the other gives way (its ties stop at the crossing); None: the wider one, else the first
    /// by name. Branch: the line the other leaves (the program works this out from the drawing, so this only has to agree with it).
    pub main: Option<String>,
    /// Branch: the turnout's frog number (4 to 20); None: the one that makes the gentlest connection.
    pub frog: Option<u32>,
}

impl JointSpec {
    /// `v` is the description, with or without `lines` (which the record adds).
    pub fn from_json(v: &Value) -> Result<JointSpec, String> {
        let obj = v.as_object().ok_or("The joint description must be a JSON object, for example {\"kind\": \"crossing\"}.")?;
        let mut s = JointSpec { kind: Kind::Crossing, main: None, frog: None };
        for (k, val) in obj {
            match k.as_str() {
                "kind" => {
                    s.kind = match val.as_str().map(str::to_lowercase).as_deref() {
                        Some("crossing" | "diamond") => Kind::Crossing,
                        Some("branch" | "turnout" | "siding") => Kind::Branch,
                        Some("join" | "connect" | "connection") => Kind::Join,
                        _ => return Err("`kind` must be \"crossing\" (two tracks cross), \"branch\" (one line leaves the other with a turnout) or \"join\" (two lines continue into each other end to end).".into()),
                    }
                }
                "main" => s.main = Some(val.as_str().filter(|t| !t.is_empty()).ok_or("`main` is the id of the line whose track runs through.")?.to_string()),
                "frog" => s.frog = Some(val.as_u64().filter(|n| (Frog::MIN as u64..=Frog::MAX as u64).contains(n)).ok_or("`frog` is the frog number, a whole number from 4 to 20.")? as u32),
                "lines" => {
                    if !val.as_array().is_some_and(|l| l.len() == 2 && l.iter().all(Value::is_string)) {
                        return Err("`lines` must name two lines.".into());
                    }
                }
                other => return Err(format!("Unknown joint field `{other}`. Use kind, main, frog.")),
            }
        }
        if s.frog.is_some() && s.kind != Kind::Branch {
            return Err("`frog` is the turnout of a branch; a crossing or a join has none.".into());
        }
        if s.main.is_some() && s.kind == Kind::Join {
            return Err("`main` does not apply to a join: the two lines are equals.".into());
        }
        Ok(s)
    }

    pub fn to_json(&self) -> Value {
        let mut v = json!({"kind": self.kind.word()});
        if let Some(m) = &self.main {
            v["main"] = json!(m);
        }
        if let Some(f) = self.frog {
            v["frog"] = json!(f);
        }
        v
    }

    pub fn summary(&self) -> String {
        match (self.kind, &self.main, self.frog) {
            (Kind::Crossing, Some(m), _) => format!("a crossing, line {m} runs through"),
            (Kind::Crossing, None, _) => "a crossing".to_string(),
            (Kind::Branch, m, f) => format!("a branch{}{}", m.as_ref().map_or(String::new(), |m| format!(" off line {m}")), f.map_or(", turnout number chosen to fit".to_string(), |f| format!(" with a No. {f} turnout"))),
            (Kind::Join, _, _) => "a join, end to end".to_string(),
        }
    }
}

/// A strip along a line where another track's ties would pile on top of this one's: ties whose centre lies inside it are left out.
#[derive(Clone, Debug)]
pub struct Band {
    pts: Vec<P>,
    half: f64,
    lo: P,
    hi: P,
}

impl Band {
    pub fn new(pts: Vec<P>, half: f64) -> Band {
        let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
        for p in &pts {
            for k in 0..2 {
                lo[k] = lo[k].min(p[k] - half);
                hi[k] = hi[k].max(p[k] + half);
            }
        }
        Band { pts, half, lo, hi }
    }

    /// Is `p` within the band (nearest distance to the line at most `half`)?
    pub fn covers(&self, p: P) -> bool {
        if p[0] < self.lo[0] || p[0] > self.hi[0] || p[1] < self.lo[1] || p[1] > self.hi[1] {
            return false;
        }
        let h2 = self.half * self.half;
        self.pts.windows(2).any(|w| {
            let (d, e) = ([w[1][0] - w[0][0], w[1][1] - w[0][1]], [p[0] - w[0][0], p[1] - w[0][1]]);
            let l2 = d[0] * d[0] + d[1] * d[1];
            let t = if l2 > 0.0 { ((e[0] * d[0] + e[1] * d[1]) / l2).clamp(0.0, 1.0) } else { 0.0 };
            let q = [e[0] - t * d[0], e[1] - t * d[1]];
            q[0] * q[0] + q[1] * q[1] <= h2
        })
    }
}
