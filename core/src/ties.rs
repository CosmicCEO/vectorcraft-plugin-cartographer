//! Ties along a route. Plug-ins run under an instruction budget and a yard has thousands of ties, so they are placed by one walk along the
//! route (no search per tie) and written as plain two-point segments.

use crate::geom::Len2;
use crate::geom::P;
use crate::offset::{MITRE_LIMIT, offset_polyline};
use crate::rail_standards::*;

pub fn arc_lengths(pts: &[P]) -> Vec<f64> {
    let mut cum = vec![0.0];
    for w in pts.windows(2) {
        cum.push(cum.last().unwrap() + (w[1][0] - w[0][0]).len2(w[1][1] - w[0][1]));
    }
    cum
}

/// Index range of the tie centres between arc lengths `from` and `to`: one per pitch, half a pitch in from each end.
/// Tie `k` is centred at `from + TIE_PITCH / 2 + k * TIE_PITCH`.
pub fn count(from: f64, to: f64) -> usize {
    if to - from >= TIE_PITCH - 1e-9 { ((to - from - TIE_PITCH) / TIE_PITCH + 1e-9).floor() as usize + 1 } else { 0 }
}

pub fn centres(from: f64, to: f64) -> impl Iterator<Item = f64> {
    (0..count(from, to)).map(move |k| from + TIE_PITCH / 2.0 + k as f64 * TIE_PITCH)
}

/// One segment per tie, `TIE_LENGTH` long and square to the route, centred on it (computed one at a time, in order).
pub fn marks<'a>(pts: &'a [P], from: f64, to: f64) -> impl Iterator<Item = [P; 2]> + 'a {
    let cum = arc_lengths(pts);
    let half = TIE_LENGTH / 2.0;
    let mut i = 1;
    centres(from, to).filter_map(move |s| {
        while i < pts.len() - 1 && cum[i] < s {
            i += 1;
        }
        let (a, b) = (pts[i - 1], pts[i]);
        let span = cum[i] - cum[i - 1];
        if span <= 0.0 {
            return None;
        }
        let t = (s - cum[i - 1]) / span;
        let (dx, dy) = ((b[0] - a[0]) / span, (b[1] - a[1]) / span);
        let p = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
        Some([[p[0] + dy * half, p[1] - dx * half], [p[0] - dy * half, p[1] + dx * half]])
    })
}

/// Most ties one build can afford to write one by one. Measured with a fuel-counting interpreter against the host's limit (about 21 M
/// of its units): a tie costs about 1,150, a yard without ties 6 to 9 M. Routes beyond this get `band`.
pub const MAX_TIES: usize = 9000;

/// The same for a layout on a curved line, where the rest of the build is dearer and every tie end is mapped through the curve.
pub const MAX_TIES_CURVED: usize = 7000;

/// The part of `pts` between arc lengths `a` and `b`.
pub fn slice(pts: &[P], a: f64, b: f64) -> Vec<P> {
    let cum = arc_lengths(pts);
    let at = |s: f64| -> P {
        let i = cum.partition_point(|&c| c < s).clamp(1, pts.len() - 1);
        let span = cum[i] - cum[i - 1];
        let t = if span > 0.0 { (s - cum[i - 1]) / span } else { 0.0 };
        [pts[i - 1][0] + (pts[i][0] - pts[i - 1][0]) * t, pts[i - 1][1] + (pts[i][1] - pts[i - 1][1]) * t]
    };
    let mut out = vec![at(a)];
    out.extend(pts.iter().zip(&cum).filter(|(_, c)| **c > a && **c < b).map(|(p, _)| *p));
    out.push(at(b));
    out
}

/// The strip the ties of a route cover, as one polygon (used instead of the ties when a build has too many to write).
pub fn band(pts: &[P], from: f64, to: f64) -> Option<Vec<P>> {
    let n = count(from, to);
    if n == 0 {
        return None;
    }
    let first = from + TIE_PITCH / 2.0;
    let mid = slice(pts, first - TIE_WIDTH / 2.0, first + (n - 1) as f64 * TIE_PITCH + TIE_WIDTH / 2.0);
    let (l, r) = (offset_polyline(&mid, -TIE_LENGTH / 2.0, MITRE_LIMIT), offset_polyline(&mid, TIE_LENGTH / 2.0, MITRE_LIMIT));
    let ring: Vec<P> = l.into_iter().chain(r.into_iter().rev()).collect();
    (ring.len() >= 3).then_some(ring)
}
