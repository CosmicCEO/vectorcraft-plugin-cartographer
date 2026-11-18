//! Branches and joins between two drawn lines (see docs/superpowers/specs/2026-11-13-joining-lines-design.md).
//!
//! A **branch**: line B ends on line A. A turnout (the NMRA RP-12.35 shape, `turnout.rs`) is put on A, on the side B comes from, on A's outer
//! track there; B's own drawing is kept except for its last stretch, which is replaced by the turnout's diverging leg and a connector curve
//! (`connector.rs`) tangent to both. The turnout's frog number is the owner's, or the one (of the nearest three) that leaves the gentlest
//! connector. Nothing in the document is changed: the replacement is a drawn copy.
//!
//! A **join**: the ends of lines A and B meet. A connector between the two ends, tangent to both lines.

use crate::connector::{Shape, connect, connect_shape};
use crate::geom::{Len2, P, Poly};
use crate::joint::{Band, nearest_frog};
use crate::offset::{MITRE_LIMIT, offset_polyline};
use crate::rail_standards::*;
use crate::ties::arc_lengths;
use crate::turnout::{Frog, Pen};
use crate::yard::{Mapper, Tap};

/// Mixed into the connector's radius: a connector never turns tighter than the turnout's own closure curve if there is room, and falls back
/// toward the NMRA minimum radius only when there is not.
const PREFERRED_RADIUS: f64 = 450.0;
/// How far along the branch line (from where it meets the main) the connector may join it, mm.
const JOIN_AT: [f64; 2] = [40.0, 100.0];

pub struct BranchPlan {
    /// The branch line as it is drawn: the turnout's diverging leg, the connector, then the owner's line from where the connector joins it.
    pub line: Vec<P>,
    /// The turnout on the main, for `yard::build`.
    pub tap: Tap,
    /// The branch's ties give way within a tie length of the main's track, where the main's lengthened switch ties cover them.
    pub avoid: Band,
    pub frog: u32,
    pub radius: f64,
    pub turn_deg: f64,
}

fn heading(t: P) -> f64 {
    t[1].atan2(t[0])
}

fn reversed(p: &[P]) -> Vec<P> {
    p.iter().rev().copied().collect()
}

/// Plans a branch of `branch` off `main`. `count` and `spacing` describe the main's group of tracks (1 and any for a single track): the
/// turnout leaves the outermost track on the branch's side. `frog` is the owner's choice, if any.
pub fn plan_branch(main: &[P], branch: &[P], count: usize, spacing: f64, frog: Option<u32>) -> Result<BranchPlan, String> {
    let pm = Poly::new(main.to_vec());
    let pbr = Poly::new(branch.to_vec());
    if pm.is_empty() || pbr.is_empty() {
        return Err("a line has no length".into());
    }
    // The branch, drawn so that it starts where it meets the main and runs away from it.
    let (first, last) = (pbr.pts[0], *pbr.pts.last().unwrap());
    let (d_first, d_last) = (pm.project(first).2, pm.project(last).2);
    let b_pts = if d_first <= d_last { pbr.pts.clone() } else { reversed(&pbr.pts) };
    let pb = Poly::new(b_pts);
    let s_main = pm.project(pb.pts[0]).0;
    let b_len = pb.len();
    let b_dir = {
        let q = pb.at(25.0_f64.min(b_len)).0;
        let v = [q[0] - pb.pts[0][0], q[1] - pb.pts[0][1]];
        let l = v[0].len2(v[1]).max(1e-9);
        [v[0] / l, v[1] / l]
    };
    // The main, oriented so the branch runs forward along it (a turnout faces the way its diverging leg goes).
    let t_here = pm.at(s_main).1;
    let rev = t_here[0] * b_dir[0] + t_here[1] * b_dir[1] < 0.0;
    let main_o = if rev { reversed(&pm.pts) } else { pm.pts.clone() };
    let po = Poly::new(main_o.clone());
    let (len_o, s_o) = (po.len(), if rev { pm.len() - s_main } else { s_main });
    let t_o = po.at(s_o).1;
    let cross = t_o[0] * b_dir[1] - t_o[1] * b_dir[0];
    let side_o = if cross >= 0.0 { 1.0 } else { -1.0 };
    let half = (count as f64 - 1.0) / 2.0 * spacing;
    let y_main = side_o * half;
    let mapper = Mapper::new(&main_o);
    let angle = (t_o[0] * b_dir[0] + t_o[1] * b_dir[1]).clamp(-1.0, 1.0).acos().to_degrees();

    let frogs: Vec<u32> = match frog {
        Some(f) => vec![f],
        None => {
            let (n, _) = nearest_frog(angle);
            (n.saturating_sub(1).max(Frog::MIN)..=(n + 1).min(Frog::MAX)).collect()
        }
    };
    let cum_b = arc_lengths(&pb.pts);
    // Try the point of switch at `x0` with frog `f`, the connector joining the branch line `t` mm along it: the score and the connector.
    let try_at = |f: &Frog, x0: f64, t: f64| -> Option<(f64, Shape)> {
        let e = f.end();
        let q = mapper.map([x0 + e[0], y_main + side_o * e[1]]);
        let psi = heading(po.at(x0 + e[0]).1) + side_o * f.alpha;
        let (tp, tt) = pb.at(t);
        if q.len2_to_p(tp) < 10.0 {
            return None;
        }
        let c = connect_shape(q, psi, tp, heading(tt), f.radius.max(PREFERRED_RADIUS), (MIN_RADIUS + half).min(f.radius.max(MIN_RADIUS + half)))?;
        Some((c.turn + 0.0004 * c.length, c))
    };
    let ts: Vec<f64> = JOIN_AT.iter().copied().filter(|t| *t < b_len - 5.0).collect();
    // Coarse sweep along the main for every candidate frog, then a fine search round the best.
    let mut best: Option<(f64, Frog, f64, f64, Shape)> = None;
    let consider = |best: &mut Option<(f64, Frog, f64, f64, Shape)>, f: Frog, x0: f64, t: f64| {
        if let Some((score, c)) = try_at(&f, x0, t) {
            if best.as_ref().is_none_or(|b| score < b.0 - 1e-9) {
                *best = Some((score, f, x0, t, c));
            }
        }
    };
    for &n in &frogs {
        let f = Frog::new(n);
        let e = f.end();
        let (lo, hi) = ((s_o - e[0] - 260.0).max(0.0), (s_o + 15.0).min(len_o - e[0]));
        let mut x0 = lo;
        while x0 <= hi {
            for &t in &ts {
                consider(&mut best, f, x0, t);
            }
            x0 += 18.0;
        }
    }
    if let Some((_, f, x_best, _, _)) = best {
        let e = f.end();
        let (lo, hi) = ((x_best - 18.0).max(0.0), (x_best + 18.0).min(len_o - e[0]));
        let mut x0 = lo;
        while x0 <= hi {
            for &t in &ts {
                consider(&mut best, f, x0, t);
            }
            x0 += 3.0;
        }
    }
    let Some((_, f, x0, t, shape)) = best else {
        return Err(format!("the branch line leaves the main at {angle:.0} degrees and cannot be joined smoothly with a turnout and curves no tighter than the NMRA minimum radius; draw it to meet the main more gently or with more room"));
    };
    // Build the winning connector's points now (only the winner's).
    let e = f.end();
    let q = mapper.map([x0 + e[0], y_main + side_o * e[1]]);
    let conn = shape.path(q, heading(po.at(x0 + e[0]).1) + side_o * f.alpha);
    // The turnout's own leg, from the point of switch to the end of the closure arc, on the drawn main.
    let mut local: Vec<P> = vec![[x0, y_main]];
    f.diverge(&mut Pen::new([x0, y_main], 0.0), side_o, &mut local);
    let mut line: Vec<P> = local.iter().map(|&p| mapper.map(p)).collect();
    line.extend(conn.pts.iter().skip(1).copied());
    line.extend(pb.pts.iter().zip(&cum_b).filter(|(_, c)| **c > t + 1e-6).map(|(p, _)| *p));
    // The tap, in the main's own (not the oriented) frame.
    let (x0_main, dir, side) = if rev { (pm.len() - x0, -1.0, -side_o) } else { (x0, 1.0, side_o) };
    let main_index = if side > 0.0 { 0 } else { count.saturating_sub(1) };
    let tap = Tap { x0: x0_main, dir, main: main_index, side, frog: f };
    // The main's outer track on that side, as a drawn line, for the band the branch's ties keep out of.
    let y_orig = side * half;
    let track = offset_polyline(&pm.pts, -y_orig, MITRE_LIMIT);
    let avoid = Band::new(track, TIE_LENGTH + 0.8);
    Ok(BranchPlan { line, tap, avoid, frog: f.n, radius: conn.radius, turn_deg: conn.turn.to_degrees() })
}

trait ToP {
    fn len2_to_p(self, other: P) -> f64;
}

impl ToP for P {
    fn len2_to_p(self, other: P) -> f64 {
        (other[0] - self[0]).len2(other[1] - self[1])
    }
}

pub struct JoinPlan {
    /// The connector track's centre line, from a point on line A to a point on line B.
    pub connector: Vec<P>,
    /// Line A and line B as they are then drawn: each trimmed back to where the connector leaves or joins it (untrimmed if the ends were
    /// already in line).
    pub a_line: Vec<P>,
    pub b_line: Vec<P>,
    pub radius: f64,
}

/// How far back from their ends the two lines may be trimmed to make the connection gentler, mm. Zero is tried first and preferred.
const TRIM: [f64; 6] = [0.0, 30.0, 60.0, 100.0, 150.0, 220.0];

fn slice_line(p: &Poly, a: f64, b: f64) -> Vec<P> {
    crate::ties::slice(&p.pts, a, b)
}

/// The connection between two lines whose ends meet, for a group of `count` tracks `spacing` apart, or why there is none. Each line is trimmed
/// back by the least that lets a smooth connector (no tighter than the NMRA minimum radius) join them tangentially.
pub fn plan_join(a: &[P], b: &[P], count: usize, spacing: f64) -> Result<JoinPlan, String> {
    let (pa, pb) = (Poly::new(a.to_vec()), Poly::new(b.to_vec()));
    if pa.is_empty() || pb.is_empty() {
        return Err("a line has no length".into());
    }
    // The pair of ends that are closest.
    let ends = |p: &Poly| [(p.pts[0], false), (*p.pts.last().unwrap(), true)];
    let mut best_ends: Option<(f64, bool, bool)> = None;
    for ea in ends(&pa) {
        for eb in ends(&pb) {
            let d = ea.0.len2_to_p(eb.0);
            if best_ends.is_none_or(|x| d < x.0) {
                best_ends = Some((d, ea.1, eb.1));
            }
        }
    }
    let (_, a_end, b_end) = best_ends.unwrap();
    let half = (count as f64 - 1.0) / 2.0 * spacing;
    let min = MIN_RADIUS + half;
    // Where along a line, counted from the meeting end, the connector leaves (A) or joins (B): the point, and the direction of travel there.
    let port_a = |ta: f64| {
        let s = if a_end { pa.len() - ta } else { ta };
        let (p, t) = pa.at(s);
        (p, heading(if a_end { t } else { [-t[0], -t[1]] }))
    };
    let port_b = |tb: f64| {
        let s = if b_end { pb.len() - tb } else { tb };
        let (p, t) = pb.at(s);
        (p, heading(if b_end { [-t[0], -t[1]] } else { t }))
    };
    let mut best: Option<(f64, f64, f64, crate::connector::Conn)> = None;
    for &ta in TRIM.iter().filter(|t| **t < pa.len() - 10.0) {
        for &tb in TRIM.iter().filter(|t| **t < pb.len() - 10.0) {
            let ((p0, h0), (p1, h1)) = (port_a(ta), port_b(tb));
            if let Some(c) = connect(p0, h0, p1, h1, PREFERRED_RADIUS.max(min), min) {
                let score = c.turn + 0.0004 * c.length + 0.0002 * (ta + tb);
                if best.as_ref().is_none_or(|x| score < x.0 - 1e-9) {
                    best = Some((score, ta, tb, c));
                }
            }
        }
    }
    let Some((_, ta, tb, c)) = best else {
        return Err("the two lines cannot be joined smoothly with curves no tighter than the NMRA minimum radius; they meet too sharply or too close together".into());
    };
    let a_line = if a_end { slice_line(&pa, 0.0, pa.len() - ta) } else { slice_line(&pa, ta, pa.len()) };
    let b_line = if b_end { slice_line(&pb, 0.0, pb.len() - tb) } else { slice_line(&pb, tb, pb.len()) };
    Ok(JoinPlan { connector: c.pts, a_line, b_line, radius: c.radius })
}
