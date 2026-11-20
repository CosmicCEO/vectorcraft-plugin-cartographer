//! Parallel track along a drawn line: rails, ties and roadbed. Lengths here are millimetres at 1:160, y up (see `builder` for the conversion).

use crate::geom::Len2;
use crate::geom::{P, Poly};
use crate::joint::Band;
use crate::offset::{MITRE_LIMIT, offset_polyline};
use crate::output::{Layer, Out};
use crate::rail_standards::*;
use crate::ties;
use serde_json::{Value, json};

pub const MAX_TRACKS: usize = 12;
const BALLAST_MARGIN: f64 = 1.0;
const RAIL_RGB: [f64; 3] = [0.17, 0.17, 0.2];
const TIE_RGB: [f64; 3] = [0.42, 0.29, 0.18];
const BALLAST_RGB: [f64; 3] = [0.82, 0.8, 0.76];

#[derive(Clone, Debug, PartialEq)]
pub struct TrackSpec {
    /// Parallel tracks, 1 to `MAX_TRACKS`, centred on the drawn line.
    pub count: usize,
    /// Distance between neighbouring track centre lines, mm.
    pub spacing: f64,
    pub ballast: bool,
    pub ties: bool,
}

impl Default for TrackSpec {
    fn default() -> TrackSpec {
        TrackSpec { count: 1, spacing: TRACK_CENTERS, ballast: true, ties: true }
    }
}

/// A length as a number (model mm: descriptions in metres are converted first, see `rail_units`) or text: `24mm`, `1in` (model inches) or
/// `13ft` (feet of ground, scaled to 1:160).
pub fn parse_length(v: &Value, name: &str) -> Result<f64, String> {
    if let Some(x) = v.as_f64() {
        return Ok(x);
    }
    let bad = || format!("`{name}` must be a number of metres or text like 4m or 13ft, not {v}.");
    let t = v.as_str().ok_or_else(bad)?.trim().to_lowercase();
    let split = t.find(|c: char| !(c.is_ascii_digit() || c == '.')).unwrap_or(t.len());
    let n: f64 = t[..split].parse().map_err(|_| bad())?;
    match t[split..].trim() {
        "" | "mm" => Ok(n),
        "in" | "\"" => Ok(n * 25.4),
        "ft" | "'" => Ok(n * FOOT),
        _ => Err(bad()),
    }
}

/// A spacing: any length (see `parse_length`), or one of NMRA RP-7.1's modeling eras, which name the minimum tangent track centres:
/// `old-time` (12 ft), `classic` (13 ft), `modern` (14 ft, also `early-modern`).
pub fn parse_spacing(v: &Value) -> Result<f64, String> {
    let era = v.as_str().map(|t| t.trim().to_lowercase().replace([' ', '_'], "-"));
    match era.as_deref() {
        Some("old-time" | "narrow-gauge" | "old-time/narrow-gauge") => Ok(12.0 * FOOT),
        Some("classic") => Ok(13.0 * FOOT),
        Some("early-modern" | "modern") => Ok(14.0 * FOOT),
        _ => parse_length(v, "spacing").map_err(|e| format!("{e} Or name an era: old-time (12 ft), classic (13 ft), modern (14 ft).")),
    }
}

impl TrackSpec {
    /// The same tracks as a yard of that many mains (a yard can take branches to other lines).
    pub fn as_yard(&self) -> crate::yard::YardSpec {
        crate::yard::YardSpec { mains: self.count, spacing: self.spacing, ballast: self.ballast, ties: self.ties, ..Default::default() }
    }

    /// Half the width of the strip this group of tracks (and its ties) covers, mm.
    pub fn footprint_half(&self) -> f64 {
        (self.count as f64 - 1.0) / 2.0 * self.spacing + TIE_LENGTH / 2.0
    }

    pub fn from_json(v: &Value) -> Result<TrackSpec, String> {
        let obj = v.as_object().ok_or("The track description must be a JSON object, for example {\"tracks\": 2}.")?;
        let mut s = TrackSpec::default();
        for (k, val) in obj {
            match k.as_str() {
                "tracks" => {
                    let n = val.as_u64().filter(|n| (1..=MAX_TRACKS as u64).contains(n)).ok_or(format!("`tracks` must be a whole number from 1 to {MAX_TRACKS}."))?;
                    s.count = n as usize;
                }
                "spacing" => {
                    s.spacing = parse_spacing(val)?;
                    if !(s.spacing > GAUGE + 2.0 * RAIL_HEAD) {
                        return Err(format!("`spacing` must be more than {:.1} mm, or the tracks touch.", GAUGE + 2.0 * RAIL_HEAD));
                    }
                }
                "ballast" => s.ballast = val.as_bool().ok_or("`ballast` must be true or false.")?,
                "ties" => s.ties = val.as_bool().ok_or("`ties` must be true or false.")?,
                other => return Err(format!("Unknown track field `{other}`. Use tracks, spacing, ballast, ties.")),
            }
        }
        Ok(s)
    }

    pub fn to_json(&self) -> Value {
        json!({"tracks": self.count, "spacing": (self.spacing * 1000.0).round() / 1000.0, "ballast": self.ballast, "ties": self.ties})
    }

    pub fn summary(&self) -> String {
        format!("{} track{}, {:.1} mm between centres{}", self.count, if self.count == 1 { "" } else { "s" }, self.spacing, if self.ballast { ", with roadbed" } else { "" })
    }

    /// Offset of track `k` (0-based) from the drawn line, mm, positive to the right of travel.
    pub fn offset(&self, k: usize) -> f64 {
        (k as f64 - (self.count as f64 - 1.0) / 2.0) * self.spacing
    }
}

/// The smallest radius the polyline turns on (circle through every three neighbouring points), mm.
pub fn min_radius(pts: &[P]) -> Option<f64> {
    let mut clean: Vec<P> = Vec::with_capacity(pts.len());
    for &p in pts {
        if clean.last().is_none_or(|l| (l[0] - p[0]).len2(l[1] - p[1]) > 0.05) {
            clean.push(p);
        }
    }
    let mut best: Option<f64> = None;
    for w in clean.windows(3) {
        let (ab, bc, ca) = ((w[1][0] - w[0][0]).len2(w[1][1] - w[0][1]), (w[2][0] - w[1][0]).len2(w[2][1] - w[1][1]), (w[0][0] - w[2][0]).len2(w[0][1] - w[2][1]));
        let cross = ((w[1][0] - w[0][0]) * (w[2][1] - w[0][1]) - (w[1][1] - w[0][1]) * (w[2][0] - w[0][0])).abs();
        if cross > 1e-9 {
            let r = ab * bc * ca / (2.0 * cross);
            best = Some(best.map_or(r, |b| b.min(r)));
        }
    }
    best
}

/// Draws `line` as `spec.count` tracks (named `gen.track.<id>.*`); returns plain-words notes about anything unusual.
/// `avoid`: strips where another track runs through, in which this one's ties are left out (see `joint`).
pub fn build(id: &str, line: &[P], spec: &TrackSpec, out: &mut Out, to_doc: impl Fn(P) -> P + Copy, avoid: &[Band]) -> Vec<String> {
    let mut notes = Vec::new();
    if Poly::new(line.to_vec()).is_empty() {
        notes.push(format!("Track {id}: the line has fewer than two points that differ, so nothing was drawn for it."));
        return notes;
    }
    let half_span = (spec.count as f64 - 1.0) / 2.0 * spec.spacing;
    if spec.ballast {
        out.layer(Layer::Roadbed);
        let reach = half_span + TIE_LENGTH / 2.0 + BALLAST_MARGIN;
        let (l, r) = (offset_polyline(line, -reach, MITRE_LIMIT), offset_polyline(line, reach, MITRE_LIMIT));
        let ring: Vec<P> = l.into_iter().chain(r.into_iter().rev()).collect();
        if ring.len() >= 3 {
            out.curve_fills(&format!("gen.track.{id}.ballast"), std::slice::from_ref(&ring), to_doc, BALLAST_RGB);
        }
    }
    for k in 0..spec.count {
        let cl = offset_polyline(line, spec.offset(k), MITRE_LIMIT);
        if cl.len() < 2 {
            continue;
        }
        let tag = format!("gen.track.{id}.t{}", k + 1);
        if spec.ties {
            out.layer(Layer::Ties);
            let len = ties::arc_lengths(&cl).last().copied().unwrap_or(0.0);
            let keep = |c: P| !avoid.iter().any(|b| b.covers(c));
            let runs = ties::runs(&cl, 0.0, len, (!avoid.is_empty()).then_some(&keep as &dyn Fn(P) -> bool));
            if !runs.is_empty() {
                out.path(&format!("{tag}.ties"), &runs, to_doc, TIE_RGB, TIE_LENGTH * PT_PER_MM, false);
            }
        }
        out.layer(Layer::Rails);
        let rails = vec![offset_polyline(&cl, -GAUGE / 2.0, MITRE_LIMIT), offset_polyline(&cl, GAUGE / 2.0, MITRE_LIMIT)];
        out.curve(&format!("{tag}.rails"), &rails, to_doc, RAIL_RGB, RAIL_HEAD * PT_PER_MM);
    }
    if spec.count > 1 && spec.spacing < 12.0 * FOOT - 1e-6 {
        notes.push(format!("Track {id}: {:.1} mm between centres is closer than the smallest NMRA tangent spacing (12 ft prototype = {:.1} mm).", spec.spacing, 12.0 * FOOT));
    }
    if let Some(r) = min_radius(line) {
        let inner = r - half_span;
        if inner < MIN_RADIUS {
            notes.push(format!("Track {id}: the tightest curve is {r:.0} mm radius on the drawn line ({inner:.0} mm on the inner track), under the {MIN_RADIUS:.0} mm that NMRA RP-11 lists as the smallest. Many locomotives will not run it."));
        }
    }
    notes
}

