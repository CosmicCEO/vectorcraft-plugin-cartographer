//! A road on fill: an embankment along a stretch of one line of a road, with side slopes down to the ground (or into water).
//! Stored under `embankments.<road id>` as a list of
//! `{"piece": 1, "from": 120, "to": 300, "height": 3, "slope": 2, "sides": "both", "ramp": 15}`: `from` and `to` are metres from the start of that line
//! (left out: its start and end), `height` is how high the road stands above the ground beside it (m), `slope` the side slope as horizontal metres per metre
//! of height (H:V, so 2 is "2:1"), `sides` both, left or right, and `ramp` the length over which the slope grows from nothing at each end.
//! The drawing is a shaded band on each side (width `height x slope`) with short hatch lines across it. Height and slope are also facts for the simulation.

use crate::crosssection::Placed;
use crate::geom::{P, Poly, add, scale};
use crate::output::Out;
use crate::roads::flip;
use serde_json::{Value, json};

const SLOPE: [f64; 3] = [0.74, 0.68, 0.5];
const HATCH: [f64; 3] = [0.55, 0.5, 0.36];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sides {
    Both,
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EmbankmentSpec {
    pub piece: usize,
    pub from: Option<f64>,
    pub to: Option<f64>,
    pub height: f64,
    pub slope: f64,
    pub sides: Sides,
    pub ramp: f64,
}

impl EmbankmentSpec {
    pub fn from_json(v: &Value) -> Result<EmbankmentSpec, String> {
        let sides = match v["sides"].as_str().map(|w| w.trim().to_ascii_lowercase()).as_deref() {
            None | Some("") | Some("both") | Some("two") => Sides::Both,
            Some("left") => Sides::Left,
            Some("right") => Sides::Right,
            Some(other) => return Err(format!("sides must be both, left or right (got \"{other}\")")),
        };
        let (from, to) = (v["from"].as_f64().map(|f| f.max(0.0)), v["to"].as_f64().map(|t| t.max(0.0)));
        if let (Some(f), Some(t)) = (from, to) {
            if t <= f + 1.0 {
                return Err("an embankment must end after it starts: \"to\" has to be more than a metre beyond \"from\"".into());
            }
        }
        Ok(EmbankmentSpec {
            piece: v["piece"].as_u64().unwrap_or(1).clamp(1, 99) as usize,
            from,
            to,
            height: v["height"].as_f64().unwrap_or(2.0).clamp(0.3, 30.0),
            slope: v["slope"].as_f64().unwrap_or(2.0).clamp(1.0, 6.0),
            sides,
            ramp: v["ramp"].as_f64().unwrap_or(15.0).clamp(0.0, 80.0),
        })
    }

    pub fn to_json(&self) -> Value {
        json!({"piece": self.piece, "from": self.from, "to": self.to, "height": self.height, "slope": self.slope,
               "sides": match self.sides { Sides::Both => "both", Sides::Left => "left", Sides::Right => "right" }, "ramp": self.ramp})
    }

    pub fn summary(&self) -> String {
        let span = match (self.from, self.to) {
            (Some(f), Some(t)) => format!("from {f} m to {t} m"),
            (Some(f), None) => format!("from {f} m to the end"),
            (None, Some(t)) => format!("from the start to {t} m"),
            (None, None) => "along the whole line".to_string(),
        };
        let sides = match self.sides { Sides::Both => "", Sides::Left => ", left side only", Sides::Right => ", right side only" };
        format!("road on fill {} m high with {}:1 slopes {span}{sides}", self.height, self.slope)
    }

    /// How far the slope reaches out from the road's edge at full height, in metres.
    pub fn reach(&self) -> f64 {
        self.height * self.slope
    }
}

/// Draws the embankment along `path` (world metres, y up) whose outermost elements are `placed`: a band each side from the road's edge out to the toe of the
/// slope, growing over `ramp` metres at each end, with hatch lines across. `base` is the name prefix.
pub fn emit(out: &mut Out, base: &str, path: &[P], placed: &[Placed], e: &EmbankmentSpec) {
    let poly = Poly::new(path.to_vec());
    let len = poly.len();
    let (from, to) = (e.from.unwrap_or(0.0).min(len), e.to.unwrap_or(len).min(len));
    if to - from < 1.0 || placed.is_empty() {
        return;
    }
    let (el, er) = (placed.first().map_or(-3.0, |p| p.left), placed.last().map_or(3.0, |p| p.right));
    let reach = e.reach();
    // Stations: every 2 m, and at every bend of the line inside the stretch.
    let steps = ((to - from) / 2.0).ceil().max(2.0) as usize;
    let mut stations: Vec<f64> = (0..=steps).map(|i| from + (to - from) * i as f64 / steps as f64).collect();
    let mut acc = 0.0;
    for w in poly.pts.windows(2) {
        acc += (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]);
        if acc > from && acc < to {
            stations.push(acc);
        }
    }
    stations.sort_by(f64::total_cmp);
    stations.dedup_by(|a, b| (*a - *b).abs() < 0.05);
    let width_at = |s: f64| -> f64 {
        if e.ramp <= 0.0 {
            return reach;
        }
        reach * ((s - from) / e.ramp).min((to - s) / e.ramp).clamp(0.0, 1.0)
    };
    for (side, edge, name, on) in [(-1.0, el, "left", e.sides != Sides::Right), (1.0, er, "right", e.sides != Sides::Left)] {
        if !on {
            continue;
        }
        let at = |s: f64, w: f64| {
            let (p, t) = poly.at(s);
            add(p, scale(Poly::right(t), edge + side * w))
        };
        let inner: Vec<P> = stations.iter().map(|&s| at(s, 0.0)).collect();
        let outer: Vec<P> = stations.iter().map(|&s| at(s, width_at(s))).collect();
        let mut ring = inner.clone();
        ring.extend(outer.iter().rev());
        out.polygon(&format!("{base}.slope.{name}"), &ring, flip, SLOPE);
        // Hatch lines across the slope, every 4 m, from the road's edge to the toe.
        let mut hatch: Vec<Vec<P>> = Vec::new();
        let mut s = from + 2.0;
        while s < to - 1.0 {
            let w = width_at(s);
            if w > 0.8 {
                hatch.push(vec![at(s, 0.0), at(s, w)]);
            }
            s += 4.0;
        }
        if !hatch.is_empty() {
            out.path(&format!("{base}.hatch.{name}"), &hatch, flip, HATCH, 0.15, false);
        }
    }
}
