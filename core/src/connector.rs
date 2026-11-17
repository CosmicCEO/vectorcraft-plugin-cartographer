//! The connector: a smooth path joining two oriented ends (a point and the direction of travel there), made of an arc, a straight and an
//! arc, never turning tighter than a given radius. It is what a branch's last stretch and a join between two lines are made of, so the
//! track is always tangent at both ends and never kinks. See docs/superpowers/specs/2026-11-13-joining-lines-design.md.

use crate::geom::P;
use crate::turnout::Pen;
use std::f64::consts::TAU;

/// A connecting path: its points (the first is the start, the last the end) and how much it turns.
#[derive(Clone, Debug)]
pub struct Conn {
    pub pts: Vec<P>,
    /// The radius of its arcs, mm.
    pub radius: f64,
    /// Total turning, radians (the two arcs added; straight pieces do not turn).
    pub turn: f64,
    pub length: f64,
}

/// The angle as 0 up to a full turn; an angle that is zero but for rounding is zero, not a full turn.
fn positive(a: f64) -> f64 {
    let t = a.rem_euclid(TAU);
    if t > TAU - 1e-7 { 0.0 } else { t }
}

/// The numbers of an arc-straight-arc path, without its points (cheap enough to try hundreds of: plug-ins run under an instruction budget).
#[derive(Clone, Copy, Debug)]
pub struct Shape {
    pub radius: f64,
    pub turn: f64,
    pub length: f64,
    sa: f64,
    sb: f64,
    a1: f64,
    k: f64,
    a2: f64,
}

impl Shape {
    /// The path's points from `p0` heading `h0`.
    pub fn path(&self, p0: P, h0: f64) -> Conn {
        let mut pts = vec![p0];
        let mut pen = Pen::new(p0, h0);
        pen.arc(self.radius, self.sa * self.a1, &mut pts);
        pen.line(self.k, &mut pts);
        pen.arc(self.radius, self.sb * self.a2, &mut pts);
        Conn { pts, radius: self.radius, turn: self.turn, length: self.length }
    }
}

/// The shortest arc-straight-arc shape of radius `r` from `p0` heading `h0` to `p1` heading `h1` (radians, counter-clockwise from +x), or
/// None if there is none (the ends are closer than the arcs need). Each arc turns left or right; the four combinations are tried.
pub fn dubins_shape(p0: P, h0: f64, p1: P, h1: f64, r: f64) -> Option<Shape> {
    let rot = |h: f64| [-h.sin(), h.cos()]; // the left-hand normal
    let (na, nb) = (rot(h0), rot(h1));
    let mut best: Option<Shape> = None;
    for sa in [1.0, -1.0] {
        for sb in [1.0, -1.0] {
            let ca = [p0[0] + sa * r * na[0], p0[1] + sa * r * na[1]];
            let cb = [p1[0] + sb * r * nb[0], p1[1] + sb * r * nb[1]];
            let d = [cb[0] - ca[0], cb[1] - ca[1]];
            let dist = (d[0] * d[0] + d[1] * d[1]).sqrt();
            let q = (sb - sa) * r / dist.max(1e-12);
            if q.abs() > 1.0 || dist < 1e-9 && sa != sb {
                continue;
            }
            // The tangent heading: sin(theta - psi) = q, with the straight running forward.
            let theta = d[1].atan2(d[0]);
            let psi = theta - q.asin();
            let k = (dist * dist - ((sb - sa) * r).powi(2)).max(0.0).sqrt();
            let a1 = positive(sa * (psi - h0));
            let a2 = positive(sb * (h1 - psi));
            let length = r * (a1 + a2) + k;
            if best.is_none_or(|b| length < b.length) {
                best = Some(Shape { radius: r, turn: a1 + a2, length, sa, sb, a1, k, a2 });
            }
        }
    }
    best
}

/// `dubins_shape` with its points.
pub fn dubins(p0: P, h0: f64, p1: P, h1: f64, r: f64) -> Option<Conn> {
    dubins_shape(p0, h0, p1, h1, r).map(|s| s.path(p0, h0))
}

/// The shape of a connector between the ends with the largest radius that fits, from `preferred` down to `min` in four steps (a bigger
/// radius is a gentler curve, so it is tried first; few steps for the instruction budget). None if even `min` does not fit.
pub fn connect_shape(p0: P, h0: f64, p1: P, h1: f64, preferred: f64, min: f64) -> Option<Shape> {
    let top = preferred.max(min);
    for f in [1.0, 0.6, 0.3, 0.0] {
        let r = min + (top - min) * f;
        // A path that loops round (more than a turn and a half) is not what anyone wants from a connection.
        if let Some(c) = dubins_shape(p0, h0, p1, h1, r).filter(|c| c.turn <= std::f64::consts::PI * 1.5) {
            return Some(c);
        }
    }
    None
}

/// `connect_shape` with its points.
pub fn connect(p0: P, h0: f64, p1: P, h1: f64, preferred: f64, min: f64) -> Option<Conn> {
    connect_shape(p0, h0, p1, h1, preferred, min).map(|s| s.path(p0, h0))
}
