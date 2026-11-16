//! Filter output written directly as JSON text (not as a JSON tree) because plug-ins run in an interpreter under an instruction budget.

use crate::geom::Len2;
use crate::geom::P;
use serde_json::Value;
use std::fmt::Write;

/// How many ties one output may write one by one by default (Track Layer's `ties::MAX_TIES`); the rail module sets its own with `limit_ties`.
const DEFAULT_MAX_TIES: usize = 9_000;

/// A stroke needs a finite width above zero; VectorCraft would draw nothing, or refuse it.
fn visible_width(width: f64) -> bool {
    width.is_finite() && width > 0.0
}

pub struct Out {
    /// New objects (they are listed first, so VectorCraft puts them just below the objects that are kept: the lines you drew stay on top).
    texts: [String; 3],
    firsts: [bool; 3],
    /// Which of the three stacks the next object goes on (see `Layer`).
    layer: usize,
    /// Existing objects kept, in their original order.
    kept: String,
    first_kept: bool,
    /// How many more ties this output may write one by one (see `ties::MAX_TIES`).
    ties_left: usize,
}

/// The stack an object goes on: new objects are listed roadbed first, then ties, then rails, whatever order they were written in.
#[derive(Clone, Copy)]
pub enum Layer {
    Roadbed = 0,
    Ties = 1,
    Rails = 2,
}

impl Default for Out {
    fn default() -> Out {
        Out::new()
    }
}

impl Out {
    pub fn new() -> Out {
        Out { texts: [String::new(), String::new(), String::new()], firsts: [true; 3], layer: 0, kept: String::new(), first_kept: true, ties_left: DEFAULT_MAX_TIES }
    }

    /// Puts the objects written from now on on the given stack.
    pub fn layer(&mut self, l: Layer) {
        self.layer = l as usize;
    }

    /// Lowers the number of ties that may still be written one by one (never raises it).
    pub fn limit_ties(&mut self, n: usize) {
        self.ties_left = self.ties_left.min(n);
    }

    /// Asks to write `n` ties one by one; false if that would pass the budget (the caller draws a strip instead).
    pub fn take_ties(&mut self, n: usize) -> bool {
        if n <= self.ties_left {
            self.ties_left -= n;
            // About 100 bytes a tie: reserve now, so the text is not copied again and again as it grows.
            self.texts[self.layer].reserve(n * 100);
            true
        } else {
            false
        }
    }

    fn sep(&mut self) {
        if !self.firsts[self.layer] {
            self.texts[self.layer].push(',');
        }
        self.firsts[self.layer] = false;
    }

    fn sep_kept(&mut self) {
        if !self.first_kept {
            self.kept.push(',');
        }
        self.first_kept = false;
    }

    /// Keep an existing object as it is.
    pub fn keep(&mut self, id: i64) {
        self.sep_kept();
        let _ = write!(self.kept, "{{\"id\":{id}}}");
    }

    /// Keep an existing object and give it a new name.
    pub fn rename(&mut self, id: i64, name: &str) {
        self.sep_kept();
        let _ = write!(self.kept, "{{\"id\":{id},\"name\":{}}}", json_string(name));
    }

    /// Keep an existing line and give it a bold, bright stroke of the given colour and width (so it is easy to see and to click).
    pub fn keep_styled(&mut self, id: i64, rgb: [f64; 3], width: f64) {
        if !visible_width(width) {
            return self.keep(id);
        }
        self.sep_kept();
        let _ = write!(self.kept, "{{\"id\":{id},\"strokes\":[{{\"paint\":");
        let mut paint = String::new();
        paint.push_str("{\"type\":\"solid\",\"color\":{\"model\":\"rgb\",\"r\":");
        num(&mut paint, rgb[0]);
        paint.push_str(",\"g\":");
        num(&mut paint, rgb[1]);
        paint.push_str(",\"b\":");
        num(&mut paint, rgb[2]);
        paint.push_str("}}");
        self.kept.push_str(&paint);
        self.kept.push_str(",\"width\":");
        num(&mut self.kept, width);
        self.kept.push_str(",\"cap\":\"round\",\"join\":\"round\",\"miterLimit\":4}]}");
    }

    /// Keep an existing line or shape, drawn as a bold bright outline with no fill (so a lake's own line never covers the water drawn from it).
    pub fn keep_outline(&mut self, id: i64, rgb: [f64; 3], width: f64) {
        self.keep_styled(id, rgb, width);
        // keep_styled wrote `...}]}` as the end of the object: put the empty fills before the closing brace.
        if self.kept.ends_with("}]}") {
            self.kept.truncate(self.kept.len() - 1);
            self.kept.push_str(",\"fills\":[]}");
        }
    }

    /// Starts an object. Returns false (and writes nothing) when no subpath is usable: VectorCraft refuses an object with no points, and a
    /// line needs 2 points, a shape 3, all finite.
    fn open_object(&mut self, name: &str, subs: &[Vec<P>], map: &impl Fn(P) -> P, closed: bool, smooth: bool) -> bool {
        let min = if closed { 3 } else { 2 };
        let usable: Vec<Vec<P>> = subs
            .iter()
            .filter(|s| s.len() >= min)
            .map(|s| {
                let mut m: Vec<P> = s.iter().map(|&w| map(w)).collect();
                // A smooth closed ring that repeats its first point (how a drawn shape is read) is one point shorter: the repeat would be a zero-length
                // chord and `thin` would drop every corner. A plain polygon keeps its points exactly as given.
                if smooth && closed && m.len() > 2 && m.first() == m.last() {
                    m.pop();
                }
                m
            })
            .filter(|m| m.len() >= min && m.iter().all(|p| p[0].is_finite() && p[1].is_finite()) && (closed || m.iter().any(|p| *p != m[0])))
            .collect();
        if usable.is_empty() {
            return false;
        }
        self.sep();
        self.texts[self.layer].push_str("{\"name\":");
        self.texts[self.layer].push_str(&json_string(name));
        self.texts[self.layer].push_str(",\"path\":{\"subpaths\":[");
        for (i, mapped) in usable.iter().enumerate() {
            if i > 0 {
                self.texts[self.layer].push(',');
            }
            self.texts[self.layer].push_str("{\"anchors\":[");
            if smooth {
                write_curve(&mut self.texts[self.layer], mapped, closed);
            } else {
                for (k, p) in mapped.iter().enumerate() {
                    if k > 0 {
                        self.texts[self.layer].push(',');
                    }
                    self.texts[self.layer].push_str("{\"p\":[");
                    num(&mut self.texts[self.layer], p[0]);
                    self.texts[self.layer].push(',');
                    num(&mut self.texts[self.layer], p[1]);
                    self.texts[self.layer].push_str("]}");
                }
            }
            self.texts[self.layer].push_str(if closed { "],\"closed\":true}" } else { "],\"closed\":false}" });
        }
        self.texts[self.layer].push_str("]}");
        true
    }

    fn paint(&mut self, rgb: [f64; 3]) {
        self.texts[self.layer].push_str("{\"type\":\"solid\",\"color\":{\"model\":\"rgb\",\"r\":");
        num(&mut self.texts[self.layer], rgb[0]);
        self.texts[self.layer].push_str(",\"g\":");
        num(&mut self.texts[self.layer], rgb[1]);
        self.texts[self.layer].push_str(",\"b\":");
        num(&mut self.texts[self.layer], rgb[2]);
        self.texts[self.layer].push_str("}}");
    }

    /// A new stroked path; `map` converts each point to a document point.
    pub fn path(&mut self, name: &str, subs: &[Vec<P>], map: impl Fn(P) -> P, rgb: [f64; 3], width: f64, closed: bool) {
        if !visible_width(width) {
            return;
        }
        if !self.open_object(name, subs, &map, closed, false) {
            return;
        }
        self.texts[self.layer].push_str(",\"strokes\":[{\"paint\":");
        self.paint(rgb);
        self.texts[self.layer].push_str(",\"width\":");
        num(&mut self.texts[self.layer], width);
        self.texts[self.layer].push_str(",\"cap\":\"butt\",\"join\":\"miter\",\"miterLimit\":4}]}");
    }

    /// A new stroked object made of many two-point segments (ties), written straight out with no allocation per segment.
    pub fn segments(&mut self, name: &str, segs: impl Iterator<Item = [P; 2]>, map: impl Fn(P) -> P, rgb: [f64; 3], width: f64) {
        let mut segs = segs.peekable();
        if segs.peek().is_none() || !visible_width(width) {
            return; // VectorCraft refuses an object with no points
        }
        let (mark, was_first) = (self.texts[self.layer].len(), self.firsts[self.layer]);
        self.sep();
        self.texts[self.layer].push_str("{\"name\":");
        self.texts[self.layer].push_str(&json_string(name));
        self.texts[self.layer].push_str(",\"path\":{\"subpaths\":[");
        let mut first = true;
        for [a, b] in segs {
            let (a, b) = (map(a), map(b));
            if ![a[0], a[1], b[0], b[1]].iter().all(|c| c.is_finite()) {
                continue; // a non-finite point would be written as 0 and draw a line to the origin
            }
            if !first {
                self.texts[self.layer].push(',');
            }
            first = false;
            self.texts[self.layer].push_str("{\"anchors\":[{\"p\":[");
            num(&mut self.texts[self.layer], a[0]);
            self.texts[self.layer].push(',');
            num(&mut self.texts[self.layer], a[1]);
            self.texts[self.layer].push_str("]},{\"p\":[");
            num(&mut self.texts[self.layer], b[0]);
            self.texts[self.layer].push(',');
            num(&mut self.texts[self.layer], b[1]);
            self.texts[self.layer].push_str("]}],\"closed\":false}");
        }
        if first {
            // Every segment was skipped: take the half-written object back.
            self.texts[self.layer].truncate(mark);
            self.firsts[self.layer] = was_first;
            return;
        }
        self.texts[self.layer].push_str("]},\"strokes\":[{\"paint\":");
        self.paint(rgb);
        self.texts[self.layer].push_str(",\"width\":");
        num(&mut self.texts[self.layer], width);
        self.texts[self.layer].push_str(",\"cap\":\"butt\",\"join\":\"miter\",\"miterLimit\":4}]}");
    }

    /// Several closed filled shapes in one object (a row of triangles, say), with no outline.
    pub fn fills(&mut self, name: &str, rings: &[Vec<P>], map: impl Fn(P) -> P, rgb: [f64; 3]) {
        if !self.open_object(name, rings, &map, true, false) {
            return;
        }
        self.texts[self.layer].push_str(",\"fills\":[");
        self.paint(rgb);
        self.texts[self.layer].push_str("]}");
    }

    /// A new filled shape with no outline.
    pub fn polygon(&mut self, name: &str, ring: &[P], map: impl Fn(P) -> P, rgb: [f64; 3]) {
        if !self.open_object(name, std::slice::from_ref(&ring.to_vec()), &map, true, false) {
            return;
        }
        // A fill is the paint itself (a stroke wraps its paint in `paint` next to its width).
        self.texts[self.layer].push_str(",\"fills\":[");
        self.paint(rgb);
        self.texts[self.layer].push_str("]}");
    }

    /// Like `path`, but the points are thinned and joined with Bezier handles, so a curve is a curve and not a run of chords. A sharp corner
    /// (a turn of more than `CORNER_DEG` at one point) stays sharp.
    pub fn curve(&mut self, name: &str, subs: &[Vec<P>], map: impl Fn(P) -> P, rgb: [f64; 3], width: f64) {
        if !visible_width(width) {
            return;
        }
        if !self.open_object(name, subs, &map, false, true) {
            return;
        }
        self.texts[self.layer].push_str(",\"strokes\":[{\"paint\":");
        self.paint(rgb);
        self.texts[self.layer].push_str(",\"width\":");
        num(&mut self.texts[self.layer], width);
        self.texts[self.layer].push_str(",\"cap\":\"butt\",\"join\":\"miter\",\"miterLimit\":4}]}");
    }

    /// Like `fills`, with smooth outlines (see `curve`).
    pub fn curve_fills(&mut self, name: &str, rings: &[Vec<P>], map: impl Fn(P) -> P, rgb: [f64; 3]) {
        if !self.open_object(name, rings, &map, true, true) {
            return;
        }
        self.texts[self.layer].push_str(",\"fills\":[");
        self.paint(rgb);
        self.texts[self.layer].push_str("]}");
    }

    pub fn finish(self) -> String {
        self.finish_with(None)
    }

    /// Ends the output, optionally replacing the data the plug-in keeps with the document. New objects come first, then the kept ones.
    pub fn finish_with(self, document_data: Option<&Value>) -> String {
        let mut out = String::from("{\"objects\":[");
        // Roadbed first (lowest), then ties, then rails, across every line, so no line's roadbed hides another line's ties.
        let mut any = false;
        for t in self.texts.iter().filter(|t| !t.is_empty()) {
            if any {
                out.push(',');
            }
            out.push_str(t);
            any = true;
        }
        if any && !self.kept.is_empty() {
            out.push(',');
        }
        out.push_str(&self.kept);
        out.push(']');
        if let Some(d) = document_data {
            out.push_str(",\"documentData\":");
            out.push_str(&serde_json::to_string(d).unwrap_or_else(|_| "null".into()));
        }
        out.push('}');
        out
    }
}

fn json_string(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into())
}

/// A number to the nearest thousandth, trailing zeros dropped, written by hand (the formatting machinery and `f64::round` are slow under the
/// plug-in's instruction budget and this runs for every coordinate). Values beyond about 4 million are clamped.
pub fn num(out: &mut String, x: f64) {
    // Round half away from zero with a truncating cast.
    let v = (x * 1000.0 + if x < 0.0 { -0.5 } else { 0.5 }) as i64;
    let mut n = v.unsigned_abs().min(u32::MAX as u64) as u32;
    let mut buf = [0u8; 16];
    let mut i = buf.len();
    let mut frac = n % 1000;
    n /= 1000;
    if frac != 0 {
        let mut digits = 3;
        while frac % 10 == 0 {
            frac /= 10;
            digits -= 1;
        }
        while digits > 0 {
            i -= 1;
            buf[i] = b'0' + (frac % 10) as u8;
            frac /= 10;
            digits -= 1;
        }
        i -= 1;
        buf[i] = b'.';
    }
    loop {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    if v < 0 {
        i -= 1;
        buf[i] = b'-';
    }
    // SAFETY: the buffer holds only ASCII digits, '.' and '-'.
    out.push_str(unsafe { std::str::from_utf8_unchecked(&buf[i..]) });
}

/// A turn bigger than this at one point is a corner, not part of a curve.
pub const CORNER_DEG: f64 = 30.0;
/// Points closer than this (document points) to the line between their kept neighbours are dropped before the curve is fitted.
const THIN: f64 = 0.12;

fn thin(pts: &[P], tol: f64) -> Vec<P> {
    if pts.len() < 3 {
        return pts.to_vec();
    }
    let mut keep = vec![false; pts.len()];
    keep[0] = true;
    keep[pts.len() - 1] = true;
    let mut stack = vec![(0, pts.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        let (p, q) = (pts[a], pts[b]);
        let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
        let len = dx.len2(dy).max(1e-9);
        let (mut far, mut at) = (0.0, a);
        for (i, r) in pts.iter().enumerate().take(b).skip(a + 1) {
            let d = ((r[0] - p[0]) * dy - (r[1] - p[1]) * dx).abs() / len;
            if d > far {
                (far, at) = (d, i);
            }
        }
        if far > tol {
            keep[at] = true;
            stack.push((a, at));
            stack.push((at, b));
        }
    }
    pts.iter().zip(keep).filter_map(|(p, k)| k.then_some(*p)).collect()
}

/// Anchors with Bezier handles through `raw` (thinned first). Handles follow the Catmull-Rom tangent, a third of the way to each neighbour.
fn write_curve(text: &mut String, raw: &[P], closed: bool) {
    let pts = thin(raw, THIN);
    let n = pts.len();
    let corner_cos = CORNER_DEG.to_radians().cos();
    let unit = |a: P, b: P| -> Option<P> {
        let m = (b[0] - a[0]).len2(b[1] - a[1]);
        (m > 1e-9).then(|| [(b[0] - a[0]) / m, (b[1] - a[1]) / m])
    };
    let at = |i: isize| -> Option<P> {
        if closed { Some(pts[i.rem_euclid(n as isize) as usize]) } else { (0..n as isize).contains(&i).then(|| pts[i as usize]) }
    };
    let handle = |text: &mut String, key: &str, p: P, q: P| {
        text.push_str(",\"");
        text.push_str(key);
        text.push_str("\":[");
        num(text, q[0]);
        text.push(',');
        num(text, q[1]);
        let _ = p;
        text.push(']');
    };
    for i in 0..n {
        if i > 0 {
            text.push(',');
        }
        let p = pts[i];
        text.push_str("{\"p\":[");
        num(text, p[0]);
        text.push(',');
        num(text, p[1]);
        text.push(']');
        let (prev, next) = (at(i as isize - 1), at(i as isize + 1));
        let d_in = prev.and_then(|q| unit(q, p));
        let d_out = next.and_then(|q| unit(p, q));
        // The tangent: along the chord between the neighbours, or along the one segment at an open end.
        let smooth = match (d_in, d_out) {
            (Some(a), Some(b)) => (a[0] * b[0] + a[1] * b[1]) >= corner_cos,
            _ => true,
        };
        if smooth {
            let t = match (prev, next) {
                (Some(a), Some(b)) => unit(a, b),
                (None, Some(b)) => unit(p, b),
                (Some(a), None) => unit(a, p),
                _ => None,
            };
            if let Some(t) = t {
                if let Some(q) = next {
                    let l = (q[0] - p[0]).len2(q[1] - p[1]) / 3.0;
                    handle(text, "out", p, [p[0] + t[0] * l, p[1] + t[1] * l]);
                }
                if let Some(q) = prev {
                    let l = (p[0] - q[0]).len2(p[1] - q[1]) / 3.0;
                    handle(text, "in", p, [p[0] - t[0] * l, p[1] - t[1] * l]);
                }
            }
        }
        text.push('}');
    }
}
