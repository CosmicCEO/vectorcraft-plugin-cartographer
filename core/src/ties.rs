//! Ties along a route. Plug-ins run under an instruction budget and a yard has thousands of ties, so they are placed by one walk along the
//! route (no search per tie). Since 2.0 a route's ties are written as one line that the panel dashes (`runs`), so a build never runs out of them.

use crate::geom::Len2;
use crate::geom::P;
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

/// The ties of a route as lines to be dashed: each run of kept ties (`keep` looks at a tie's centre) is the part of the route from the first
/// tie's near edge to the last tie's far edge. Drawn `TIE_LENGTH` wide with the dash `[TIE_WIDTH, TIE_PITCH - TIE_WIDTH]` from its start, dash k
/// covers tie k exactly; the panel sets that dash after a build (VectorCraft ignores dashes in a plug-in's output). Thousands of ties cost a
/// handful of points, so a build never runs out of them.
pub fn runs(pts: &[P], from: f64, to: f64, keep: Option<&dyn Fn(P) -> bool>) -> Vec<Vec<P>> {
    let n = count(from, to);
    if n == 0 || pts.len() < 2 {
        return Vec::new();
    }
    let (first, last) = (from + TIE_PITCH / 2.0, from + TIE_PITCH / 2.0 + (n - 1) as f64 * TIE_PITCH);
    // No strip to give way to (the usual case): one line, without looking at each tie.
    let Some(keep) = keep else { return vec![slice(pts, first - TIE_WIDTH / 2.0, last + TIE_WIDTH / 2.0)] };
    let cum = arc_lengths(pts);
    let mut i = 1;
    let mut at = |s: f64| -> P {
        while i < pts.len() - 1 && cum[i] < s {
            i += 1;
        }
        let span = cum[i] - cum[i - 1];
        let t = if span > 0.0 { (s - cum[i - 1]) / span } else { 0.0 };
        [pts[i - 1][0] + (pts[i][0] - pts[i - 1][0]) * t, pts[i - 1][1] + (pts[i][1] - pts[i - 1][1]) * t]
    };
    let mut out = Vec::new();
    let mut run: Option<(f64, f64)> = None;
    for c in centres(from, to) {
        if keep(at(c)) {
            run = Some(run.map_or((c, c), |(a, _)| (a, c)));
        } else if let Some((a, b)) = run.take() {
            out.push(slice(pts, a - TIE_WIDTH / 2.0, b + TIE_WIDTH / 2.0));
        }
    }
    if let Some((a, b)) = run {
        out.push(slice(pts, a - TIE_WIDTH / 2.0, b + TIE_WIDTH / 2.0));
    }
    out
}

/// The ties a dashed tie line shows, as segments: the inverse of `runs`, for checks and tests. `run` is one subpath of a tie line, `scale`
/// the drawing's units per model mm (document points: `PT_PER_MM`).
pub fn expand(run: &[P], scale: f64) -> Vec<[P; 2]> {
    let cum = arc_lengths(run);
    let len = cum.last().copied().unwrap_or(0.0);
    let half = TIE_LENGTH / 2.0 * scale;
    let mut out = Vec::new();
    let (mut s, mut i) = (TIE_WIDTH / 2.0 * scale, 1);
    while s <= len - TIE_WIDTH / 2.0 * scale + 1e-6 && run.len() >= 2 {
        while i < run.len() - 1 && cum[i] < s {
            i += 1;
        }
        let (a, b) = (run[i - 1], run[i]);
        let span = (cum[i] - cum[i - 1]).max(1e-12);
        let t = (s - cum[i - 1]) / span;
        let (dx, dy) = ((b[0] - a[0]) / span, (b[1] - a[1]) / span);
        let p = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
        out.push([[p[0] + dy * half, p[1] - dx * half], [p[0] - dy * half, p[1] + dx * half]]);
        s += TIE_PITCH * scale;
    }
    out
}


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

