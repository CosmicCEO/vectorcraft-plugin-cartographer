//! "Build Track": regenerates the rails, ties and other track parts from the lines you drew and their descriptions.
//! Everything it creates is named `gen.*`, so a rebuild replaces the previous result and never touches the paths you edit.

use crate::geom::P;
use crate::branch;
use crate::joint::{self, Band, Found, JointSpec};
use crate::yard::Tap;
use crate::output::Out;
use crate::rail_standards::{PT_PER_MM, TRACK_CENTERS};
use crate::tracks::{self, TrackSpec};
use crate::yard::{self, YardSpec};
use crate::vcdoc::{Item, items_from_filter_input};
use std::collections::HashMap;
use serde_json::{Map, Value, json};

pub const GEN_PREFIX: &str = "gen.";
/// The line you drew stays on top of what is built from it, drawn bright so it is easy to find and click (stroke width in points).
const SPINE: [f64; 3] = [0.85, 0.1, 0.1];
/// 0.75 points when 1 mm of model was 72/25.4 points (Track Layer): the same 0.26 mm of model at any scale.
const SPINE_WIDTH_MM: f64 = 0.75 / (72.0 / 25.4);

/// `track.<id>` -> `<id>`.
pub fn track_id(name: &str) -> Option<String> {
    name.strip_prefix("track.").filter(|s| !s.is_empty()).map(str::to_string)
}

pub fn to_mm(p: P) -> P {
    [p[0] / PT_PER_MM, -p[1] / PT_PER_MM]
}

fn to_doc(p: P) -> P {
    [p[0] * PT_PER_MM, -p[1] * PT_PER_MM]
}

/// What the plug-in keeps with the document (an object; everything it does not know is carried over).
fn data_of(params: &Value) -> Map<String, Value> {
    // The rail code works in model millimetres; the record is stored in metres (`stored`).
    let mut data = crate::rail_units::model_data(&params["_context"]["documentData"]);
    data.insert("schema".into(), json!(crate::storage::SCHEMA));
    data
}

fn spec_for(data: &Map<String, Value>, id: &str) -> Result<TrackSpec, String> {
    match data.get("tracks").and_then(|t| t.get(id)) {
        Some(v) => TrackSpec::from_json(v),
        None => Ok(TrackSpec::default()),
    }
}

/// Filter output text: every source object kept by id, the old `gen.*` paths dropped, fresh `gen.*` paths added.
pub fn build_text(input: &Value, params: &Value) -> Result<String, String> {
    let items = items_from_filter_input(input);
    if !items.iter().any(|i| track_id(&i.name).is_some()) {
        return Err("Nothing to build. Draw a line, name it track.<id>, then Select All so the track is included.".into());
    }
    let mut data = data_of(params);
    let mut out = Out::new();
    let mut notes: Vec<String> = Vec::new();
    for it in items.iter().filter(|i| !i.name.starts_with(GEN_PREFIX)) {
        let Some(iid) = it.id else { continue };
        if track_id(&it.name).is_some() {
            out.keep_styled(iid, SPINE, SPINE_WIDTH_MM * PT_PER_MM);
        } else {
            out.keep(iid);
        }
    }
    notes.extend(build_into(&items, &params["_context"]["documentData"], &mut out)?);
    data.insert("notes".into(), json!(notes));
    Ok(out.finish_with(Some(&Value::Object(crate::rail_units::stored_data(&data)))))
}

/// The drawn tracks' output settings: rail dimensions are model millimetres at 0.16 points each, so numbers are written with 4 decimals (0.0006 mm of
/// model; 5 decimals cost 3% more fuel and change nothing that can be seen) and curve thinning is 0.00677 points (0.04 mm of model).
const RAIL_DECIMALS: u8 = 4;
const RAIL_THINNING: f64 = 0.00677;

/// The highlight colour and stroke width of a `track.*` line (the line you drew stays on top of what is built from it).
pub fn spine_style() -> ([f64; 3], f64) {
    (SPINE, SPINE_WIDTH_MM * PT_PER_MM)
}

/// Builds every `track.*` line in `items` into `out` (rails, ties, roadbed, joints, branches) and returns what the build had to say. `data` is the
/// document's plug-in data. The output is written with the rail settings and put back to the defaults afterwards, so a road written next keeps its own.
/// A stored description that cannot be read is an error (the whole build fails, as in Track Layer).
pub fn build_into(items: &[Item], data: &Value, out: &mut Out) -> Result<Vec<String>, String> {
    build_into_with(items, data, out, &HashMap::new())
}

/// `build_into`, with extra strips per track where its ties are left out (a road crossing it at a level crossing).
pub fn build_into_with(items: &[Item], data: &Value, out: &mut Out, extra: &HashMap<String, Vec<crate::joint::Band>>) -> Result<Vec<String>, String> {
    let data = crate::rail_units::model_data(data);
    let mut notes: Vec<String> = Vec::new();
    out.precision(RAIL_DECIMALS);
    out.thinning(RAIL_THINNING);
    out.displace(false); // a model at true scale: the style may colour it but never move it
    let result = build_lines(items, &data, out, &mut notes, extra);
    out.displace(true);
    out.layer(crate::output::Layer::Roadbed); // the stack a road writes to
    out.precision(crate::output::DEFAULT_DECIMALS);
    out.thinning(crate::output::DEFAULT_THINNING);
    result.map(|()| notes)
}

fn build_lines(items: &[Item], data: &Map<String, Value>, out: &mut Out, notes: &mut Vec<String>, extra: &HashMap<String, Vec<crate::joint::Band>>) -> Result<(), String> {
    // The drawn lines (millimetres, y up) and how wide a strip each one's track covers.
    let mut lines: Vec<(String, Vec<P>)> = Vec::new();
    for it in items {
        if let (Some(id), Some(sub)) = (track_id(&it.name), it.subs.first()) {
            lines.push((id, sub.iter().map(|&p| to_mm(p)).collect()));
        }
    }
    let applied = apply_joints(data, &lines, notes);
    for (id, line) in &lines {
        let line = applied.replace.get(id).unwrap_or(line);
        let mut joined: Vec<crate::joint::Band> = applied.avoid.get(id).cloned().unwrap_or_default();
        joined.extend(extra.get(id).cloned().unwrap_or_default());
        let bands = joined.as_slice();
        let taps = applied.taps.get(id).map_or(&[][..], Vec::as_slice);
        match data.get("yards").and_then(|y| y.get(id)) {
            Some(y) => notes.extend(yard::build(id, line, &YardSpec::from_json(y)?, out, to_doc, bands, taps)),
            None => {
                let spec = spec_for(data, id)?;
                // A line with branches leaving it is drawn as a yard of that many mains, which knows about turnouts.
                if taps.is_empty() {
                    notes.extend(tracks::build(id, line, &spec, out, to_doc, bands));
                } else {
                    notes.extend(yard::build(id, line, &spec.as_yard(), out, to_doc, bands, taps));
                }
            }
        }
    }
    for (tag, line, count, spacing) in &applied.extra {
        let spec = TrackSpec { count: *count, spacing: *spacing, ..TrackSpec::default() };
        notes.extend(tracks::build(tag, line, &spec, out, to_doc, &[]));
    }
    Ok(())
}

/// Half the width of the strip the track described for line `id` covers (its mains, for a yard).
/// The half width of everything a track line may carry where a road crosses it, model millimetres: a yard's spurs too (see `level`).
pub(crate) fn crossing_half_mm(data: &Map<String, Value>, id: &str) -> f64 {
    match data.get("yards").and_then(|y| y.get(id)).map(YardSpec::from_json) {
        Some(Ok(y)) => ((y.mains as f64 - 1.0) / 2.0 + y.left.max(y.right) as f64) * y.spacing + crate::rail_standards::TIE_LENGTH / 2.0,
        _ => footprint_half(data, id),
    }
}

fn footprint_half(data: &Map<String, Value>, id: &str) -> f64 {
    match (data.get("yards").and_then(|y| y.get(id)), spec_for(data, id)) {
        (Some(y), _) => YardSpec::from_json(y).map_or(0.0, |s| s.footprint_half()),
        (None, Ok(t)) => t.footprint_half(),
        _ => 0.0,
    }
}

/// What the joints in the document do to the drawing.
#[derive(Default)]
struct Applied {
    /// Strips in which a line's ties give way, per line.
    avoid: std::collections::HashMap<String, Vec<Band>>,
    /// Lines whose drawn path is replaced (a branch, from its turnout through the connector, then as drawn).
    replace: std::collections::HashMap<String, Vec<P>>,
    /// Turnouts leaving a line's mains.
    taps: std::collections::HashMap<String, Vec<Tap>>,
    /// Extra tracks: a connector between two lines (name tag, centre line, track count, spacing).
    extra: Vec<(String, Vec<P>, usize, f64)>,
}

/// How many tracks the line `id` has side by side, and how far apart (a yard's mains).
fn group_of(data: &Map<String, Value>, id: &str) -> (usize, f64) {
    match (data.get("yards").and_then(|y| y.get(id)), spec_for(data, id)) {
        (Some(y), _) => YardSpec::from_json(y).map_or((1, TRACK_CENTERS), |s| (s.mains, s.spacing)),
        (None, Ok(t)) => (t.count, t.spacing),
        _ => (1, TRACK_CENTERS),
    }
}

/// The path with each end pushed out `by` mm along its own direction, so a connector's roadbed overlaps the lines it meets (two shapes that
/// only touch show a hairline seam between them).
fn overlap_ends(mut pts: Vec<P>, by: f64) -> Vec<P> {
    let n = pts.len();
    if n < 2 {
        return pts;
    }
    let push = |from: P, to: P| -> P {
        let d = [to[0] - from[0], to[1] - from[1]];
        let l = (d[0] * d[0] + d[1] * d[1]).sqrt().max(1e-9);
        [to[0] + d[0] / l * by, to[1] + d[1] / l * by]
    };
    let (first, last) = (push(pts[1], pts[0]), push(pts[n - 2], pts[n - 1]));
    pts[0] = first;
    pts[n - 1] = last;
    pts
}

/// For every joint in the document: finds where its two lines meet again and works out what it does to the drawing. A joint that cannot be
/// applied is left out with a note; nothing is guessed.
fn apply_joints(data: &Map<String, Value>, lines: &[(String, Vec<P>)], notes: &mut Vec<String>) -> Applied {
    let mut ap = Applied::default();
    let Some(joints) = data.get("joints").and_then(Value::as_object) else { return ap };
    let mut ids: Vec<&String> = joints.keys().collect();
    ids.sort();
    for jid in ids {
        let j = &joints[jid];
        let names: Vec<String> = j["lines"].as_array().map(|l| l.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default();
        let [a, b] = names.as_slice() else {
            notes.push(format!("Joint {jid}: it does not name two lines; left out."));
            continue;
        };
        let (Some(la), Some(lb)) = (lines.iter().find(|l| &l.0 == a), lines.iter().find(|l| &l.0 == b)) else {
            notes.push(format!("Joint {jid} joins lines {a} and {b}, but one of them is not in the document; left out."));
            continue;
        };
        let spec = match JointSpec::from_json(j) {
            Ok(s) => s,
            Err(e) => {
                notes.push(format!("Joint {jid}: {e} Left out."));
                continue;
            }
        };
        let Some(found) = joint::find(&la.1, &lb.1) else {
            notes.push(format!("Joint {jid}: lines {a} and {b} do not meet within {:.0} mm; left out.", joint::TOLERANCE));
            continue;
        };
        if found.kind() != spec.kind.word() {
            notes.push(format!("Joint {jid} is a {} but {}; left out.", spec.kind.word(), found.describe(a, b)));
            continue;
        }
        match found {
            Found::Crossing { .. } => {
                // The line that runs through keeps its ties; the other's stop at its edge.
                let (ha, hb) = (footprint_half(data, a), footprint_half(data, b));
                let a_wins = match spec.main.as_deref() {
                    Some(m) if m == a => true,
                    Some(m) if m == b => false,
                    _ => ha > hb + 1e-9 || ((ha - hb).abs() <= 1e-9 && a <= b),
                };
                let (win, lose, half) = if a_wins { (la, lb, ha) } else { (lb, la, hb) };
                ap.avoid.entry(lose.0.clone()).or_default().push(Band::new(win.1.clone(), half));
            }
            Found::Branch { branch, .. } => {
                let (main, br) = if branch == 0 { (lb, la) } else { (la, lb) };
                if spec.main.as_deref().is_some_and(|m| m != main.0) {
                    notes.push(format!("Joint {jid}: line {} ends on line {}, so {} is the main, not {}; left out.", br.0, main.0, main.0, spec.main.as_deref().unwrap_or("")));
                    continue;
                }
                let (count, spacing) = group_of(data, &main.0);
                if group_of(data, &br.0).0 != 1 {
                    notes.push(format!("Joint {jid}: line {} has more than one track; a branch with several tracks is not built yet; left out.", br.0));
                    continue;
                }
                match branch::plan_branch(&main.1, &br.1, count, spacing, spec.frog) {
                    Ok(plan) => {
                        ap.replace.insert(br.0.clone(), plan.line);
                        ap.taps.entry(main.0.clone()).or_default().push(plan.tap);
                        ap.avoid.entry(br.0.clone()).or_default().push(plan.avoid);
                    }
                    Err(e) => notes.push(format!("Joint {jid}: {e}; left out.")),
                }
            }
            Found::Join { .. } => {
                let ((ca, sa), (cb, sb)) = (group_of(data, a), group_of(data, b));
                if ca != cb || (sa - sb).abs() > 1e-6 {
                    notes.push(format!("Joint {jid}: lines {a} and {b} have different numbers or spacings of tracks; joining them is not built yet; left out."));
                    continue;
                }
                match branch::plan_join(&la.1, &lb.1, ca, sa) {
                    Ok(plan) => {
                        ap.replace.insert(a.clone(), plan.a_line);
                        ap.replace.insert(b.clone(), plan.b_line);
                        ap.extra.push((format!("{a}.{jid}"), overlap_ends(plan.connector, 1.2), ca, sa));
                    }
                    Err(e) => notes.push(format!("Joint {jid}: {e}; left out.")),
                }
            }
        }
    }
    ap
}

/// The same as a JSON value (for tests and callers that want one).
pub fn build(input: &Value, params: &Value) -> Result<Value, String> {
    serde_json::from_str(&build_text(input, params)?).map_err(|e| e.to_string())
}

/// Names that already mean something else, so a plain line must not be mistaken for them.
fn is_reserved_name(name: &str) -> bool {
    // The same rule as Describe Road (centreline, profile, flag, arrow and legacy junction names too), plus road lines.
    name.starts_with(GEN_PREFIX) || crate::builder::is_reserved_name(name) || crate::roads::segment_id(name).is_some()
}

fn clean_id(t: &str) -> String {
    t.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').take(16).collect()
}

/// Command "Set Tracks": stores the description (see `tracks`) in the document's plug-in data. Applies to the selected `track.*` lines,
/// or to the track named in `segment`; one selected plain line is renamed `track.<id>` with a fresh id.
pub fn set_tracks_text(input: &Value, params: &Value) -> Result<String, String> {
    set_text(input, params, "tracks", "yards", |v, _| Ok(TrackSpec::from_json(v)?.to_json()), "Describe the track, for example {\"tracks\": 2, \"spacing\": \"13ft\"}.")
}

/// Command "Set Yard": the same for a yard (see `yard`). Checks that the selected line is long enough.
pub fn set_yard_text(input: &Value, params: &Value) -> Result<String, String> {
    set_text(
        input,
        params,
        "yards",
        "tracks",
        |v, line| {
            let spec = YardSpec::from_json(v)?;
            if let Some(len) = line {
                let avail = spec.length.unwrap_or(len - spec.before_end - spec.at);
                if spec.at + spec.min_length() > len + 1e-6 || avail < spec.min_length() - 1e-6 {
                    return Err(format!("This yard needs {:.0} mm of line from {:.0} mm on, and the selected line gives {:.0} mm. Draw a longer line, use a smaller frog number, or fewer spurs.", spec.min_length(), spec.at, avail.min(len - spec.at).max(0.0)));
                }
            }
            if let Some(len) = line {
                if let Some(problem) = spec.crossover_problem(len) {
                    return Err(problem);
                }
            }
            Ok(spec.to_json())
        },
        "Describe the yard, for example {\"mains\": 2, \"left\": 8, \"right\": 4, \"ends\": \"merge\"}.",
    )
}

/// Shared by the commands that describe a drawn line: `kind` is the store this description goes in, `other` the store it replaces.
/// `check` validates the description (with the selected line's length in mm, if one line is selected) and returns its stored form.
fn set_text(input: &Value, params: &Value, kind: &str, other: &str, check: impl Fn(&Value, Option<f64>) -> Result<Value, String>, ask: &str) -> Result<String, String> {
    let items = items_from_filter_input(input);
    let mut data = data_of(params);
    let raw = params["description"].as_str().map(str::trim).filter(|t| !t.is_empty()).ok_or(ask.to_string())?;
    let value: Value = serde_json::from_str(raw).map_err(|e| format!("the description is not valid JSON: {e}"))?;
    let value = crate::rail_units::record_to_model(&value);
    let known = |id: &str| ["tracks", "yards"].iter().any(|k| data.get(*k).and_then(|t| t.get(id)).is_some());
    let length_of = |id: &str| items.iter().find(|i| track_id(&i.name).as_deref() == Some(id)).or(None).and_then(|i| i.subs.first()).map(|s| crate::geom::Poly::new(s.iter().map(|&p| to_mm(p)).collect()).len());

    let mut renames: Vec<(i64, String)> = Vec::new();
    let mut targets: Vec<(String, Option<f64>)> = Vec::new();
    if let Some(seg) = params["segment"].as_str().map(clean_id).filter(|s| !s.is_empty()) {
        let len = length_of(&seg);
        targets.push((seg, len));
    } else {
        for it in &items {
            if let Some(id) = track_id(&it.name).filter(|id| !targets.iter().any(|t| &t.0 == id)) {
                let len = length_of(&id);
                targets.push((id, len));
            }
        }
        if targets.is_empty() {
            let plain: Vec<&Item> = items.iter().filter(|i| i.id.is_some() && !i.subs.is_empty() && !is_reserved_name(&i.name)).collect();
            match plain.as_slice() {
                [] => return Err("Select the line to describe (or give its track id).".into()),
                [one] => {
                    let id = (1..).map(|n| format!("t{n}")).find(|id| !known(id)).unwrap_or_default();
                    renames.push((one.id.unwrap_or_default(), format!("track.{id}")));
                    let len = one.subs.first().map(|s| crate::geom::Poly::new(s.iter().map(|&p| to_mm(p)).collect()).len());
                    targets.push((id, len));
                }
                _ => return Err("More than one line is selected: select the one line to describe.".into()),
            }
        }
    }
    let mut table = data.get(kind).and_then(Value::as_object).cloned().unwrap_or_default();
    let mut dropped = data.get(other).and_then(Value::as_object).cloned().unwrap_or_default();
    for (id, len) in targets {
        table.insert(id.clone(), check(&value, len)?);
        dropped.remove(&id);
    }
    data.insert(kind.into(), Value::Object(table));
    data.insert(other.into(), Value::Object(dropped));
    let mut out = Out::new();
    for it in &items {
        if let Some(iid) = it.id {
            match renames.iter().find(|r| r.0 == iid) {
                Some((_, name)) => out.rename(iid, name),
                None => out.keep(iid),
            }
        }
    }
    Ok(out.finish_with(Some(&Value::Object(crate::rail_units::stored_data(&data)))))
}

/// Command "Set Joint": the selected lines (one of each of two tracks) meet; stores the joint (see `joint`) in the document's plug-in data.
/// `description` is JSON ({"kind": "crossing", "main": "a"}); the two line ids come from the selection.
pub fn set_joint_text(input: &Value, params: &Value) -> Result<String, String> {
    let items = items_from_filter_input(input);
    let mut data = data_of(params);
    let mut ids: Vec<String> = Vec::new();
    for it in &items {
        if let Some(id) = track_id(&it.name).filter(|id| !ids.contains(id)) {
            ids.push(id);
        }
    }
    let [a, b] = <[String; 2]>::try_from(ids).map_err(|r: Vec<String>| format!("Select one line of each of the two tracks that meet; {} track{} selected.", r.len(), if r.len() == 1 { " is" } else { "s are" }))?;
    let value: Value = match params["description"].as_str().map(str::trim).filter(|t| !t.is_empty()) {
        Some(raw) => serde_json::from_str(raw).map_err(|e| format!("the joint description is not valid JSON: {e}"))?,
        None => json!({"kind": "crossing"}),
    };
    let spec = JointSpec::from_json(&value)?;
    if let Some(m) = &spec.main {
        if m != &a && m != &b {
            return Err(format!("`main` must be one of the two lines, {a} or {b}."));
        }
    }
    let geometry = |id: &str| items.iter().find(|i| track_id(&i.name).as_deref() == Some(id)).and_then(|i| i.subs.first()).map(|s| s.iter().map(|&p| to_mm(p)).collect::<Vec<P>>()).unwrap_or_default();
    let (ga, gb) = (geometry(&a), geometry(&b));
    let Some(found) = joint::find(&ga, &gb) else {
        return Err(format!("Lines {a} and {b} do not meet (nowhere do they come within {:.0} mm of each other). Draw one across or onto the other, then select both.", joint::TOLERANCE));
    };
    if found.kind() != spec.kind.word() {
        return Err(format!("You asked for a {}, but {}.", spec.kind.word(), found.describe(&a, &b)));
    }
    // Check the joint can actually be drawn, so the owner hears about it now and not as a note after the build.
    match found {
        Found::Branch { branch, .. } => {
            let (main, br) = if branch == 0 { (&b, &a) } else { (&a, &b) };
            if spec.main.as_deref().is_some_and(|m| m != main) {
                return Err(format!("Line {br} ends on line {main}, so {main} is the main, not {}.", spec.main.as_deref().unwrap_or("")));
            }
            if group_of(&data, br).0 != 1 {
                return Err(format!("Line {br} has more than one track; a branch with several tracks is not built yet."));
            }
            let (count, spacing) = group_of(&data, main);
            let (ml, bl) = if branch == 0 { (&gb, &ga) } else { (&ga, &gb) };
            branch::plan_branch(ml, bl, count, spacing, spec.frog)?;
        }
        Found::Join { .. } => {
            let ((ca, sa), (cb, sb)) = (group_of(&data, &a), group_of(&data, &b));
            if ca != cb || (sa - sb).abs() > 1e-6 {
                return Err(format!("Lines {a} and {b} have different numbers or spacings of tracks; joining them is not built yet."));
            }
            branch::plan_join(&ga, &gb, ca, sa)?;
        }
        Found::Crossing { .. } => {}
    }
    let mut table = data.get("joints").and_then(Value::as_object).cloned().unwrap_or_default();
    let same = |j: &Value| { let l: Vec<&str> = j["lines"].as_array().map(|x| x.iter().filter_map(Value::as_str).collect()).unwrap_or_default(); l.len() == 2 && l.contains(&a.as_str()) && l.contains(&b.as_str()) };
    let jid = table.iter().find(|(_, j)| same(j)).map(|(k, _)| k.clone()).unwrap_or_else(|| (1..).map(|n| format!("j{n}")).find(|k| !table.contains_key(k)).unwrap_or_default());
    let mut record = spec.to_json();
    record["lines"] = json!([a, b]);
    table.insert(jid, record);
    data.insert("joints".into(), Value::Object(table));
    let mut out = Out::new();
    for it in &items {
        if let Some(iid) = it.id {
            out.keep(iid);
        }
    }
    Ok(out.finish_with(Some(&Value::Object(crate::rail_units::stored_data(&data)))))
}
