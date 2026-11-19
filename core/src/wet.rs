//! A road meets water: a bridge over it, a culvert under it, or a causeway (the road on fill across it).
//! Stored under `wets.<id>` as `{"road": "a", "water": "w1", "treatment": "bridge", "deck_height": 3, "abutment": 4, "shape": "pipe", "count": 2, "size": 1.2,
//! "fill_height": 2.5, "slope": 2}`. Where the road crosses the water is found from the two lines; for a creek or river the road spans the water's width
//! (slanted if it crosses at an angle) plus its banks, for a pond or lake from where the road enters to where it leaves, and for a shore from where it
//! enters to the end of the line. Drawn above the roads, in a layer of its own (`gen.wet.<id>.*`). Heights are facts for the simulation.
//! - bridge: a deck over the span with parapets and an abutment at each end, set back from the water by the bank. `deck_height` is the clearance above the water.
//! - culvert (creeks and rivers only): `count` pipes or boxes of `size` metres under the road, dashed since they are hidden, with a headwall at each end,
//!   and the road on `fill_height` of fill with `slope` side slopes across the valley.
//! - causeway: the road on `fill_height` of fill with `slope` side slopes from where it enters the water to where it leaves, a little beyond.

use crate::crosssection::{Anchor, layout};
use crate::crossing::{cross2, pieces, slice_path, unit};
use crate::embank::{self, EmbankmentSpec, Sides};
use crate::geom::{P, Poly, add, dash_polyline, scale, sub};
use crate::offset::{MITRE_LIMIT, offset_polyline};
use crate::output::Out;
use crate::roads::{CURB, Spec, emit, flip, heading};
use crate::vcdoc::Item;
use crate::water::{self, Kind, WaterSpec, is_closed, water_id};
use serde_json::{Value, json};

const DARK: [f64; 3] = [0.2, 0.2, 0.22];
const CONCRETE: [f64; 3] = [0.35, 0.35, 0.38];
const GUARD: [f64; 3] = [0.5, 0.5, 0.55];
const RIPRAP: [f64; 3] = [0.72, 0.72, 0.7];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Treatment {
    Bridge,
    Culvert,
    Causeway,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    Pipe,
    Box,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WetSpec {
    pub road: String,
    pub water: String,
    pub treatment: Treatment,
    pub deck_height: f64,
    pub abutment: f64,
    pub shape: Shape,
    pub count: usize,
    pub size: f64,
    pub fill_height: f64,
    pub slope: f64,
}

fn clean_id(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').take(16).collect()
}

impl WetSpec {
    pub fn from_json(v: &Value) -> Result<WetSpec, String> {
        let id = |key: &str| v[key].as_str().map(clean_id).filter(|s| !s.is_empty());
        let (Some(road), Some(water)) = (id("road"), id("water")) else { return Err("a crossing needs the ids of a road and a water feature".into()) };
        let word = v["treatment"].as_str().map(|w| w.trim().to_ascii_lowercase().replace([' ', '_'], "-"));
        let treatment = match word.as_deref() {
            Some("bridge") | Some("span") | Some("viaduct") => Treatment::Bridge,
            Some("culvert") | Some("pipe") | Some("box-culvert") | Some("pipes") => Treatment::Culvert,
            Some("causeway") | Some("embankment") | Some("fill") | Some("dam") | Some("levee") => Treatment::Causeway,
            Some(other) => return Err(format!("treatment must be bridge, culvert or causeway (got \"{other}\")")),
            None => return Err("a crossing needs \"treatment\": bridge, culvert or causeway".into()),
        };
        let shape = match v["shape"].as_str().map(|w| w.trim().to_ascii_lowercase()).as_deref() {
            None | Some("") | Some("pipe") | Some("round") | Some("circular") => Shape::Pipe,
            Some("box") | Some("square") | Some("rectangular") => Shape::Box,
            Some(other) => return Err(format!("shape must be pipe or box (got \"{other}\")")),
        };
        let default_size = if shape == Shape::Pipe { 1.2 } else { 2.4 };
        Ok(WetSpec {
            road,
            water,
            treatment,
            deck_height: v["deck_height"].as_f64().unwrap_or(3.0).clamp(0.0, 60.0),
            abutment: v["abutment"].as_f64().unwrap_or(4.0).clamp(0.0, 30.0),
            shape,
            count: v["count"].as_u64().unwrap_or(2).clamp(1, 6) as usize,
            size: v["size"].as_f64().unwrap_or(default_size).clamp(0.3, 6.0),
            fill_height: v["fill_height"].as_f64().unwrap_or(if treatment == Treatment::Culvert { 2.5 } else { 2.0 }).clamp(0.3, 30.0),
            slope: v["slope"].as_f64().unwrap_or(2.0).clamp(1.0, 6.0),
        })
    }

    pub fn to_json(&self) -> Value {
        json!({
            "road": self.road, "water": self.water,
            "treatment": match self.treatment { Treatment::Bridge => "bridge", Treatment::Culvert => "culvert", Treatment::Causeway => "causeway" },
            "deck_height": self.deck_height, "abutment": self.abutment, "shape": if self.shape == Shape::Pipe { "pipe" } else { "box" },
            "count": self.count, "size": self.size, "fill_height": self.fill_height, "slope": self.slope
        })
    }

    pub fn summary(&self) -> String {
        let what = match self.treatment {
            Treatment::Bridge => format!("bridge, {} m clearance above the water, abutments {} m beyond the banks", self.deck_height, self.abutment),
            Treatment::Culvert => format!("culvert of {} {} {} m {}, road on {} m of fill at {}:1", self.count, if self.shape == Shape::Pipe { "pipes" } else { "boxes" }, self.size, if self.shape == Shape::Pipe { "across" } else { "wide" }, self.fill_height, self.slope),
            Treatment::Causeway => format!("causeway, road on {} m of fill at {}:1", self.fill_height, self.slope),
        };
        format!("road {} over water {}: {what}", self.road, self.water)
    }
}

/// Where a road meets water, as distances along the road.
struct Span {
    s_in: f64,
    s_out: f64,
    /// How far the banks add at each end, along the road.
    bank: f64,
    /// For a creek or river: the middle of the crossing, the water's direction there and the sine of the angle between road and water.
    flow: Option<(f64, P, f64)>,
}

/// Every place `road` crosses `line`, as (distance along the road, point, road direction, line direction), in order along the road.
fn hits(road: &[P], line: &[P]) -> Vec<(f64, P, P, P)> {
    let mut out: Vec<(f64, P, P, P)> = Vec::new();
    let mut base = 0.0;
    for wa in road.windows(2) {
        let (r, len) = (sub(wa[1], wa[0]), (wa[1][0] - wa[0][0]).hypot(wa[1][1] - wa[0][1]));
        for wb in line.windows(2) {
            let sdir = sub(wb[1], wb[0]);
            let den = cross2(r, sdir);
            if den.abs() < 1e-12 {
                continue;
            }
            let q = sub(wb[0], wa[0]);
            let (t, u) = (cross2(q, sdir) / den, cross2(q, r) / den);
            if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
                out.push((base + t * len, add(wa[0], scale(r, t)), unit(r), unit(sdir)));
            }
        }
        base += len;
    }
    out.sort_by(|a, b| a.0.total_cmp(&b.0));
    out.dedup_by(|a, b| (a.0 - b.0).abs() < 0.5);
    out
}

fn span_of(road: &[P], line: &[P], ws: &WaterSpec) -> Result<Span, String> {
    let h = hits(road, line);
    let road_len = Poly::new(road.to_vec()).len();
    let bank_w = ws.bank_width(-1.0).max(ws.bank_width(1.0));
    match ws.kind {
        Kind::Flowing => {
            let &(s, p, rd, wd) = h.first().ok_or("the road does not cross the water line")?;
            let sin = cross2(rd, wd).abs();
            if sin < 0.17 {
                return Err("the road runs nearly along the water, so it does not cross it".into());
            }
            let l = ws.width / sin;
            Ok(Span { s_in: s - l / 2.0, s_out: s + l / 2.0, bank: (ws.bank_width(-1.0) + ws.bank_width(1.0)) / 2.0 / sin, flow: Some((s, wd, sin)) }).map(|sp| {
                let _ = p;
                sp
            })
        }
        Kind::Still => {
            if h.len() < 2 {
                return Err("the road does not go in and out of the water: it needs to cross the shape twice".into());
            }
            Ok(Span { s_in: h[0].0, s_out: h[1].0, bank: bank_w, flow: None })
        }
        Kind::Shore => {
            let first = h.first().ok_or("the road does not cross the shore")?;
            Ok(Span { s_in: first.0, s_out: road_len, bank: bank_w, flow: None })
        }
    }
}

/// Builds every stored road-meets-water treatment. Returns notes.
pub fn build(items: &[Item], spec: &Spec, out: &mut Out) -> Vec<String> {
    let mut notes = Vec::new();
    let mut wets = spec.wets();
    wets.sort_by(|a, b| a.0.cmp(&b.0));
    for (id, v) in wets {
        match WetSpec::from_json(&v).and_then(|w| emit_one(items, spec, &id, &w, out)) {
            Ok(ns) => notes.extend(ns.into_iter().map(|n| format!("crossing {id}: {n}"))),
            Err(e) => notes.push(format!("crossing {id}: {e}")),
        }
    }
    notes
}

fn emit_one(items: &[Item], spec: &Spec, id: &str, w: &WetSpec, out: &mut Out) -> Result<Vec<String>, String> {
    let wline = items.iter().find(|i| water_id(&i.name).as_deref() == Some(w.water.as_str()) && i.subs.first().is_some_and(|s| s.len() >= 2)).ok_or_else(|| format!("water {} is not in the document", w.water))?;
    let wsub = &wline.subs[0];
    let ws = spec.water_drawn(&w.water, is_closed(wsub));
    let wpath: Vec<P> = wsub.iter().map(|&p| flip(p)).collect();
    // The shore's line is its water's edge; a lake's ring and a river's centreline likewise; all are crossed as lines.
    let mut last_err = format!("road {} is not in the document", w.road);
    for (_, rpath) in pieces(items, &w.road) {
        match span_of(&rpath, &wpath, &ws) {
            Ok(span) => return draw(spec, id, w, &ws, &rpath, &span, out),
            Err(e) => last_err = e,
        }
    }
    Err(last_err)
}

fn draw(spec: &Spec, id: &str, w: &WetSpec, ws: &WaterSpec, road: &[P], span: &Span, out: &mut Out) -> Result<Vec<String>, String> {
    let (parsed, anchor, _) = spec.for_segment(&w.road);
    let placed = layout(&parsed, &spec.library, heading(road), anchor.clone())?;
    let poly = Poly::new(road.to_vec());
    let len = poly.len();
    let base = format!("gen.wet.{id}");
    let road_width = placed.last().map_or(7.2, |p| p.right) - placed.first().map_or(-3.6, |p| p.left);
    let notes = Vec::new();
    match w.treatment {
        Treatment::Bridge => {
            let (a, b) = ((span.s_in - span.bank - w.abutment).max(0.0), (span.s_out + span.bank + w.abutment).min(len));
            let deck = slice_path(road, a, b);
            emit(out, &format!("{base}.deck"), &deck, &placed);
            for (k, off) in [placed.first().map_or(-3.6, |p| p.left), placed.last().map_or(3.6, |p| p.right)].into_iter().enumerate() {
                let edge = offset_polyline(&deck, off, MITRE_LIMIT);
                if edge.len() >= 2 {
                    out.path(&format!("{base}.parapet.{}", k + 1), &[edge], flip, DARK, 0.5, false);
                }
            }
            for (k, s) in [a, b].into_iter().enumerate() {
                let (p, t) = poly.at(s);
                let n = Poly::right(t);
                out.path(&format!("{base}.abutment.{}", k + 1), &[vec![add(p, scale(n, placed.first().map_or(-3.6, |q| q.left))), add(p, scale(n, placed.last().map_or(3.6, |q| q.right)))]], flip, DARK, 0.6, false);
            }
            // Wing walls (guardrail flares): from each corner of each abutment, 10 m back along the road and 2.5 m outward.
            let (el, er) = (placed.first().map_or(-3.6, |p| p.left), placed.last().map_or(3.6, |p| p.right));
            for (k, (s, dir)) in [(a, -1.0), (b, 1.0)].into_iter().enumerate() {
                for (side, edge, name) in [(-1.0, el, "left"), (1.0, er, "right")] {
                    let (p, t) = poly.at(s);
                    let (p2, t2) = poly.at((s + dir * 10.0).clamp(0.0, len));
                    let (from, to) = (add(p, scale(Poly::right(t), edge)), add(p2, scale(Poly::right(t2), edge + side * 2.5)));
                    out.path(&format!("{base}.wing.{}.{name}", k + 1), &[vec![from, to]], flip, DARK, 0.3, false);
                    // Guardrail: from where the wall ends, 30 m further back along the road, 2.5 m off the pavement edge.
                    let (r0, r1) = ((s + dir * 10.0).clamp(0.0, len), (s + dir * 40.0).clamp(0.0, len));
                    if (r1 - r0).abs() > 1.0 {
                        let (q0, u0) = poly.at(r0);
                        let (q1, u1) = poly.at(r1);
                        out.path(&format!("{base}.rail.{}.{name}", k + 1), &[vec![add(q0, scale(Poly::right(u0), edge + side * 2.5)), add(q1, scale(Poly::right(u1), edge + side * 2.5))]], flip, GUARD, 0.3, false);
                    }
                    // Rock slope protection (riprap) on the bank beside the abutment: 6 m along the road, from 0.5 to 5 m off the pavement edge.
                    let (q2, u2) = poly.at((s + dir * 6.0).clamp(0.0, len));
                    let (n0, n2) = (Poly::right(t), Poly::right(u2));
                    let ring = [add(p, scale(n0, edge + side * 0.5)), add(p, scale(n0, edge + side * 5.0)), add(q2, scale(n2, edge + side * 5.0)), add(q2, scale(n2, edge + side * 0.5))];
                    out.polygon(&format!("{base}.riprap.{}.{name}", k + 1), &ring, flip, RIPRAP);
                }
            }
            // A long span over the water stands on piers, at most 45 m apart.
            let water = span.s_out - span.s_in;
            if water > 60.0 {
                let n = (water / 45.0).ceil() as usize - 1;
                for i in 1..=n {
                    let (c, t) = poly.at(span.s_in + water * i as f64 / (n + 1) as f64);
                    let (nrm, half) = (Poly::right(t), 0.3 * (er - el));
                    let mid = (el + er) / 2.0;
                    let corner = |along: f64, across: f64| add(add(c, scale(t, along)), scale(nrm, mid + across));
                    out.polygon(&format!("{base}.pier.{i}"), &[corner(-1.5, -half), corner(1.5, -half), corner(1.5, half), corner(-1.5, half)], flip, CONCRETE);
                }
            }
        }
        Treatment::Causeway => {
            let e = EmbankmentSpec { piece: 1, from: Some((span.s_in - 8.0).max(0.0)), to: Some((span.s_out + 8.0).min(len)), height: w.fill_height, slope: w.slope, sides: Sides::Both, ramp: 12.0 };
            embank::emit(out, &format!("{base}.embank"), road, &placed, &e);
        }
        Treatment::Culvert => {
            let Some((s_x, wd, sin)) = span.flow else {
                return Err("a culvert is for a creek or river (an open water line); use a bridge or causeway across a lake or the sea".into());
            };
            let reach = w.fill_height * w.slope;
            let lf = (span.s_out - span.s_in) + 2.0 * span.bank + 2.0 * reach;
            let e = EmbankmentSpec { piece: 1, from: Some((s_x - lf / 2.0).max(0.0)), to: Some((s_x + lf / 2.0).min(len)), height: w.fill_height, slope: w.slope, sides: Sides::Both, ramp: (lf / 4.0).min(12.0) };
            embank::emit(out, &format!("{base}.embank"), road, &placed, &e);
            // The pipes run along the water, under the road and through the fill each side of it, side by side across the creek.
            let (centre, _) = poly.at(s_x);
            let lp = road_width / sin + 2.0 * reach;
            let lateral = Poly::right(wd);
            let (gap, d) = (0.6 * w.size, w.size);
            let group = w.count as f64 * d + (w.count as f64 - 1.0) * gap;
            let along = |t: f64, side: f64| add(add(centre, scale(wd, t)), scale(lateral, side));
            let mut dashes: Vec<Vec<P>> = Vec::new();
            for k in 0..w.count {
                let c = (k as f64 - (w.count as f64 - 1.0) / 2.0) * (d + gap);
                for edge in [c - d / 2.0, c + d / 2.0] {
                    dashes.extend(dash_polyline(&[along(-lp / 2.0, edge), along(lp / 2.0, edge)], 1.0, 0.8));
                }
            }
            out.path(&format!("{base}.pipe"), &dashes, flip, CONCRETE, if w.shape == Shape::Box { 0.3 } else { 0.18 }, false);
            for (k, t) in [-lp / 2.0, lp / 2.0].into_iter().enumerate() {
                out.path(&format!("{base}.headwall.{}", k + 1), &[vec![along(t, -group / 2.0 - 0.4), along(t, group / 2.0 + 0.4)]], flip, CONCRETE, 0.6, false);
            }
        }
    }
    let _ = (CURB, Anchor::Auto, water::WATER, ws);
    Ok(notes)
}
