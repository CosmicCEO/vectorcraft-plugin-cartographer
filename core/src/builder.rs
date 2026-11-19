//! The plug-in's commands. Each takes the filter input and parameters and returns the output text. Everything a build creates is named
//! `gen.*`, so a rebuild replaces the previous result and never touches the paths the owner edits.

use crate::crossing::{self, CrossingSpec};
use crate::embank;
use crate::hub::{self, HubSpec};
use crate::junction::{self, JunctionSpec, junction_id};
use crate::output::Out;
use crate::level;
use crate::rail_builder;
use crate::style::StyleKit;
use crate::units::{TargetScale, ground_m_for_print_mm};
use crate::roads::{self, Spec, anchor_from, segment_id};
use crate::storage::{Doc, Settings};
use crate::transition;
use crate::vcdoc::{Item, items_from_filter_input};
use crate::water;
use crate::wet;
use serde_json::{Value, json};

pub const GEN_PREFIX: &str = "gen.";

/// A note above this many input points. Measured 2026-11-19 on VectorCraft 0.24.1: a plain two-lane road builds up to about 19,450 points and is then
/// refused for its instruction budget; junctions, water and hand-drawn styles cost more per point (not measured), so the note comes well before: 12,000 is an estimate.
const NEAR_LIMIT_POINTS: usize = 12_000;

/// The stored descriptions of the document, after checking the record is one this version may read (a newer schema is refused).
fn load_spec(params: &Value) -> Result<Spec, String> {
    let data = &params["_context"]["documentData"];
    Doc::check_schema(data)?;
    Ok(Spec::from_document_data(data))
}

/// How far hand-drawn lines wander, metres: 0.4 when drawing true, 0.25 mm on paper at a target scale (never more than 3 m).
const WOBBLE_TRUE_M: f64 = 0.4;
const WOBBLE_PRINT_MM: f64 = 0.25;
const WOBBLE_MAX_M: f64 = 3.0;

/// The document's style, read from its settings. A missing, unreadable or invalid record gives the plain kit (medium-contrast, no wobble).
pub fn style_kit(params: &Value) -> StyleKit {
    let s = Settings::from_json_lenient(&params["_context"]["documentData"]["settings"]).0;
    let amplitude = match s.target_print_scale {
        TargetScale::True => WOBBLE_TRUE_M,
        TargetScale::Ratio(n) => ground_m_for_print_mm(WOBBLE_PRINT_MM, n).min(WOBBLE_MAX_M),
    };
    StyleKit::new(s.style, s.seed, amplitude)
}

/// The thinnest a road stroke may be, metres of ground: 0.12 mm on paper at the document's target print scale (0 when drawing true).
pub fn min_stroke(params: &Value) -> f64 {
    Settings::from_json_lenient(&params["_context"]["documentData"]["settings"]).0.target_print_scale.min_width_m(crate::units::MIN_STROKE_PRINT_MM)
}

/// `load_spec` without the element library, for a build that draws no road.
fn load_spec_without_library(params: &Value) -> Result<Spec, String> {
    let data = &params["_context"]["documentData"];
    Doc::check_schema(data)?;
    Ok(Spec::without_library(data))
}
/// The drawn line of a road or junction: bright and bold (metres of stroke), so it can be seen and clicked among the generated shapes.
const SPINE_ROAD: [f64; 3] = [0.85, 0.1, 0.1];
const SPINE_JUNCTION: [f64; 3] = [0.1, 0.35, 0.9];
const SPINE_WATER: [f64; 3] = [0.0, 0.55, 0.6];
const SPINE_WIDTH: f64 = 1.0;


/// A short sentence for a card: `style hand-drawn, target print scale 1:5000, seed 7, north 10 degrees`.
pub fn settings_summary(s: &Settings) -> String {
    format!("style {}, target print scale {}, seed {}, north {} degrees", s.style.id(), s.target_print_scale.to_text(), s.seed, s.north_deg)
}

/// Command "Set Settings": changes the document settings named in `description` (a JSON object; keys left out keep their value), keeps every
/// selected object, and writes the record back. A record saved by a newer Cartographer is refused.
pub fn set_settings_text(input: &Value, params: &Value) -> Result<String, String> {
    let mut doc = Doc::from_document_data_lenient(&params["_context"]["documentData"])?;
    let patch: Value = match params["description"].as_str().map(str::trim).filter(|t| !t.is_empty()) {
        Some(raw) => serde_json::from_str(raw).map_err(|e| format!("the settings description is not valid JSON: {e}"))?,
        None => json!({}),
    };
    Settings::check_keys(&patch)?;
    let mut merged = doc.settings.to_json();
    match (merged.as_object_mut(), patch.as_object()) {
        // A null means "leave this one as it is".
        (Some(m), Some(p)) => m.extend(p.iter().filter(|(_, v)| !v.is_null()).map(|(k, v)| (k.clone(), v.clone()))),
        _ => return Err("the settings description must be a JSON object".into()),
    }
    doc.settings = Settings::from_json(&merged)?;
    let mut out = Out::new();
    for item in items_from_filter_input(input) {
        if let Some(id) = item.id {
            out.keep(id);
        }
    }
    Ok(out.finish_with(Some(&doc.to_value())))
}

/// Filter output text: every source object kept by id, the old `gen.*` paths dropped, fresh `gen.*` paths added.
pub fn build_text(input: &Value, params: &Value) -> Result<String, String> {
    let items = items_from_filter_input(input);
    let has_road = items.iter().any(|i| segment_id(&i.name).is_some() || junction_id(&i.name).is_some() || water::water_id(&i.name).is_some());
    let spec = if has_road { load_spec(params)? } else { load_spec_without_library(params)? };
    let has_track = items.iter().any(|i| rail_builder::track_id(&i.name).is_some());
    if !has_road && !has_track {
        return Err("Nothing to build. Draw a line and name it road.<id> or track.<id> (or a water line), then Select All so it is included.".into());
    }

    let mut out = Out::new();
    out.set_style(style_kit(params));
    out.set_min_stroke(min_stroke(params));
    // A stored setting that cannot be read is ignored (the others still apply) and said so here.
    let mut notes: Vec<String> = Settings::from_json_lenient(&params["_context"]["documentData"]["settings"]).1;
    let points: usize = items.iter().filter(|i| !i.name.starts_with(GEN_PREFIX)).map(|i| i.subs.iter().map(Vec::len).sum::<usize>()).sum();
    if points > NEAR_LIMIT_POINTS {
        notes.push(format!("this build has {points} points; VectorCraft refuses a build that needs too many instructions (a plain two-lane road of about 19,000 points, less with junctions, water or the hand-drawn style), so split long roads"));
    }
    for it in items.iter().filter(|i| !i.name.starts_with(GEN_PREFIX)) {
        let Some(id) = it.id else { continue };
        // The line you drew stays on top of what is built from it, and is drawn bold so it is easy to find and click.
        if segment_id(&it.name).is_some() {
            out.keep_styled(id, SPINE_ROAD, SPINE_WIDTH);
        } else if junction_id(&it.name).is_some() {
            out.keep_styled(id, SPINE_JUNCTION, SPINE_WIDTH);
        } else if water::water_id(&it.name).is_some() {
            out.keep_outline(id, SPINE_WATER, SPINE_WIDTH);
        } else if rail_builder::track_id(&it.name).is_some() {
            let (rgb, width) = rail_builder::spine_style();
            out.keep_styled(id, rgb, width);
        } else {
            out.keep(id);
        }
    }
    if has_road {
        let mut trims = crossing::trims(&items, &spec);
        trims.extend(hub::trims(&items, &spec));
        notes.extend(roads::build_clipped(&items, &spec, &trims, &mut out));
        notes.extend(junction::build(&items, &spec, &mut out)); // junctions are drawn over the roads they join
        notes.extend(crossing::build(&items, &spec, &mut out)); // and so are intersections where two roads cross
        notes.extend(hub::build(&items, &spec, &mut out)); // and where three or more meet
        let artboard = params["_context"]["artboard"].as_array().filter(|a| a.len() == 4).and_then(|a| Some([a[0].as_f64()?, a[1].as_f64()?, a[2].as_f64()?, a[3].as_f64()?]));
        notes.extend(water::build(&items, &spec, artboard, &mut out));
        notes.extend(wet::build(&items, &spec, &mut out)); // a road meeting water is drawn over both
    }
    if has_track {
        // Tracks are built after the roads, so their ties and rails lie above the roads' paving; a stored description that cannot be read fails the build.
        // A road crossing a track at a level crossing leaves the track's ties out under it.
        let level_bands = if has_road { level::avoid_bands(&items, &spec) } else { Default::default() };
        let mut track_notes = rail_builder::build_into_with(&items, &params["_context"]["documentData"], &mut out, &level_bands).map_err(|e| crate::rail_units::metres_text(&e))?;
        if has_road {
            // A missing level crossing is said first: it is the note a user can act on. All of these share the track half of the 30 notes.
            let mut level_notes = level::missing_notes(&items, &spec);
            let data = crate::rail_units::model_data(&params["_context"]["documentData"]);
            out.displace(false); // the panel and signs sit on the track, a model at true scale: never wobbled or thickened
            level_notes.extend(level::build(&items, &spec, &data, &mut out));
            out.displace(true);
            level_notes.extend(track_notes);
            track_notes = level_notes;
        }
        let track_notes: Vec<String> = track_notes.iter().map(|n| crate::rail_units::metres_text(n)).collect();
        // Only 30 notes are kept: share them between roads and tracks so a busy road network cannot hide every track warning.
        notes = fair_notes(notes, track_notes, MAX_NOTES);
    }
    if has_road && !has_track && !spec.level_crossings().is_empty() {
        // No track at all: stored level crossings only have notes to give (their track is gone).
        let data = crate::rail_units::model_data(&params["_context"]["documentData"]);
        notes.extend(level::build(&items, &spec, &data, &mut out));
    }
    // What the build had to say is kept with the document so the panel can show it; the rest of the stored data is carried over unchanged.
    let mut spec = spec;
    spec.set_notes(&notes);
    Ok(out.finish_with(Some(&spec.to_value())))
}

/// The same as a JSON value (for tests and callers that want one).
pub fn build(input: &Value, params: &Value) -> Result<Value, String> {
    serde_json::from_str(&build_text(input, params)?).map_err(|e| e.to_string())
}

/// A legacy junction marker's first name segment is one capital letter and a number (`J7`, `C12`); such a name is never a plain line.
fn is_junction_name(name: &str) -> bool {
    let id = name.split('.').next().unwrap_or("");
    id.len() > 1 && id.starts_with(|c: char| c.is_ascii_uppercase()) && id[1..].chars().all(|c| c.is_ascii_digit())
}

/// How many build notes the document keeps.
const MAX_NOTES: usize = 30;

/// Up to `cap` notes from two lists, each list getting at least half when it has that many; the rest of the room goes to whichever has more. Road notes
/// come first.
pub fn fair_notes(a: Vec<String>, b: Vec<String>, cap: usize) -> Vec<String> {
    let half = cap / 2;
    let take_a = a.len().min(half.max(cap.saturating_sub(b.len())));
    let take_b = b.len().min(cap - take_a);
    a.into_iter().take(take_a).chain(b.into_iter().take(take_b)).collect()
}

/// Names that already mean something else, so a plain line must not be mistaken for them (shared by Describe Road and Describe Track).
pub(crate) fn is_reserved_name(name: &str) -> bool {
    name == "centerline" || ["gen.", "profile.", "flag.", "arrow."].iter().any(|p| name.starts_with(p)) || is_junction_name(name) || junction_id(name).is_some() || water::water_id(name).is_some() || rail_builder::track_id(name).is_some()
}

/// Command "Set Cross-section": stores the road's cross-section (see `crosssection`) in the document's plug-in data. Applies to the
/// selected `road.*` lines, or to the segment named in `segment`; a single plain line is turned into a road with a fresh id.
pub fn set_cross_section_text(input: &Value, params: &Value) -> Result<String, String> {
    let items = items_from_filter_input(input);
    let mut spec = load_spec(params)?;
    let text = params["cross_section"].as_str().map(str::trim).filter(|t| !t.is_empty()).ok_or("Write the cross-section, for example: bike-lane | lane back x2 | margin 3 | lane fwd x2 | bike-lane")?;
    let anchor = params["anchor"].as_str().and_then(anchor_from);
    let clean = |t: &str| t.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').take(16).collect::<String>();

    let mut targets: Vec<String> = Vec::new();
    let mut renames: Vec<(i64, String)> = Vec::new();
    if let Some(seg) = params["segment"].as_str().map(clean).filter(|s| !s.is_empty()) {
        targets.push(seg);
    } else {
        for it in &items {
            if let Some(id) = segment_id(&it.name).filter(|id| !targets.contains(id)) {
                targets.push(id);
            }
        }
        // "Selected pieces only": give them an id of their own, starting from a copy of the old description, so the other pieces keep theirs.
        if params["scope"].as_str() == Some("selected") && !targets.is_empty() {
            let mut fresh: Vec<String> = Vec::new();
            for old in &targets {
                let new = (1..).map(|n| format!("r{n}")).find(|id| !spec.segments.contains_key(id) && !fresh.contains(id) && !targets.contains(id)).unwrap_or_default();
                for it in items.iter().filter(|i| segment_id(&i.name).as_deref() == Some(old.as_str())) {
                    if let Some(iid) = it.id {
                        renames.push((iid, format!("road.{new}")));
                    }
                }
                if let Some(prev) = spec.segments.get(old).cloned() {
                    spec.segments.insert(new.clone(), prev);
                }
                fresh.push(new);
            }
            targets = fresh;
        }
        if targets.is_empty() {
            let plain: Vec<&Item> = items.iter().filter(|i| i.id.is_some() && !i.subs.is_empty() && !is_reserved_name(&i.name)).collect();
            match plain.as_slice() {
                [] => return Err("Select the line to describe (or give its segment id).".into()),
                [one] => {
                    let taken = |id: &str| spec.segments.contains_key(id);
                    let id = (1..).map(|n| format!("r{n}")).find(|id| !taken(id)).unwrap_or_default();
                    renames.push((one.id.unwrap_or_default(), format!("road.{id}")));
                    targets.push(id);
                }
                _ => return Err("More than one line is selected: select the one line to describe.".into()),
            }
        }
    }
    for id in &targets {
        spec.set_cross_section(id, text, anchor.clone())?;
    }
    let mut out = Out::new();
    for it in &items {
        if let Some(id) = it.id {
            match renames.iter().find(|r| r.0 == id) {
                Some((_, name)) => out.rename(id, name),
                None => out.keep(id),
            }
        }
    }
    Ok(out.finish_with(Some(&spec.to_value())))
}

/// Command "Set Junction": stores a junction description (see `junction`) in the document's plug-in data. Applies to the selected
/// `junction.*` line, or to the one selected plain line, which is renamed `junction.<id>`; `junction` names the id explicitly.
pub fn set_junction_text(input: &Value, params: &Value) -> Result<String, String> {
    let items = items_from_filter_input(input);
    let mut spec = load_spec(params)?;
    let raw = params["description"].as_str().map(str::trim).filter(|t| !t.is_empty()).ok_or("Give the junction description (JSON), for example {\"legs\": [{\"side\": \"left\", \"cross_section\": \"lane back | lane fwd\"}], \"control\": \"signal\", \"crosswalks\": true}")?;
    let value: Value = serde_json::from_str(raw).map_err(|e| format!("the junction description is not valid JSON: {e}"))?;
    let js = JunctionSpec::from_json(&value, &spec.library)?;
    let clean = |t: &str| t.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').take(16).collect::<String>();

    let mut renames: Vec<(i64, String)> = Vec::new();
    let id = if let Some(id) = params["junction"].as_str().map(clean).filter(|s| !s.is_empty()) {
        id
    } else if let Some(id) = items.iter().find_map(|i| junction_id(&i.name)) {
        id
    } else {
        let plain: Vec<&Item> = items.iter().filter(|i| i.id.is_some() && !i.subs.is_empty() && !is_reserved_name(&i.name) && segment_id(&i.name).is_none()).collect();
        match plain.as_slice() {
            [] => return Err("Select the connecting line to describe (or give its junction id).".into()),
            [one] => {
                let id = (1..).map(|n| format!("j{n}")).find(|id| spec.junction_value(id).is_none()).unwrap_or_default();
                renames.push((one.id.unwrap_or_default(), format!("junction.{id}")));
                id
            }
            _ => return Err("More than one line is selected: select the one connecting line to describe.".into()),
        }
    };
    spec.set_junction(&id, &js);
    let mut out = Out::new();
    for it in &items {
        if let Some(iid) = it.id {
            match renames.iter().find(|r| r.0 == iid) {
                Some((_, name)) => out.rename(iid, name),
                None => out.keep(iid),
            }
        }
    }
    Ok(out.finish_with(Some(&spec.to_value())))
}

/// Command "Set Crossing": the two selected roads (one line of each) cross; stores the intersection's description (see `crossing`) in the
/// document's plug-in data. `description` is JSON (control, crosswalks, radius, minor); the two road ids come from the selection.
pub fn set_crossing_text(input: &Value, params: &Value) -> Result<String, String> {
    let items = items_from_filter_input(input);
    let mut spec = load_spec(params)?;
    let mut roads: Vec<String> = Vec::new();
    for it in &items {
        if let Some(id) = segment_id(&it.name).filter(|id| !roads.contains(id)) {
            roads.push(id);
        }
    }
    let [a, b] = <[String; 2]>::try_from(roads).map_err(|r: Vec<String>| format!("Select exactly two roads (one line of each) that cross; {} road{} selected.", r.len(), if r.len() == 1 { " is" } else { "s are" }))?;
    let mut value: Value = match params["description"].as_str().map(str::trim).filter(|t| !t.is_empty()) {
        Some(raw) => serde_json::from_str(raw).map_err(|e| format!("the intersection description is not valid JSON: {e}"))?,
        None => json!({}),
    };
    value["roads"] = json!([a, b]);
    let cs = CrossingSpec::from_json(&value)?;
    if crossing::find(&items, &a, &b).is_none() {
        return Err(format!("Roads {a} and {b} do not cross within the selected lines. Draw one across the other, then select both."));
    }
    let id = spec.crossing_between(&a, &b).unwrap_or_else(|| (1..).map(|n| format!("k{n}")).find(|id| !spec.crossings().iter().any(|(k, _)| k == id)).unwrap_or_default());
    spec.set_crossing(&id, &cs);
    let mut out = Out::new();
    for it in &items {
        if let Some(iid) = it.id {
            out.keep(iid);
        }
    }
    Ok(out.finish_with(Some(&spec.to_value())))
}

/// Command "Set Hub": three or more selected roads that meet at one point; stores the hub's description (see `hub`). `description` is JSON
/// (treatment, crosswalks, radius, minor, stub); the road ids come from the selection.
pub fn set_hub_text(input: &Value, params: &Value) -> Result<String, String> {
    let items = items_from_filter_input(input);
    let mut spec = load_spec(params)?;
    let mut roads: Vec<String> = Vec::new();
    for it in &items {
        if let Some(id) = segment_id(&it.name).filter(|id| !roads.contains(id)) {
            roads.push(id);
        }
    }
    if roads.len() < 3 {
        return Err(format!("Select one line of each of three or more roads that meet at a point; {} road{} selected.", roads.len(), if roads.len() == 1 { " is" } else { "s are" }));
    }
    let mut value: Value = match params["description"].as_str().map(str::trim).filter(|t| !t.is_empty()) {
        Some(raw) => serde_json::from_str(raw).map_err(|e| format!("the hub description is not valid JSON: {e}"))?,
        None => json!({}),
    };
    value["roads"] = json!(roads);
    let hs = HubSpec::from_json(&value)?;
    let id = spec.hub_with(&roads).unwrap_or_else(|| (1..).map(|n| format!("h{n}")).find(|id| !spec.hubs().iter().any(|(k, _)| k == id)).unwrap_or_default());
    spec.set_hub(&id, &hs);
    let mut out = Out::new();
    for it in &items {
        if let Some(iid) = it.id {
            out.keep(iid);
        }
    }
    Ok(out.finish_with(Some(&spec.to_value())))
}

/// Command "Set Transition": the selected line of a road gains or loses lanes; stores the transition (see `transition`) in the document's plug-in
/// data. `description` is JSON (change, side, direction, count, at, length, speed, to, piece, remove).
pub fn set_transition_text(input: &Value, params: &Value) -> Result<String, String> {
    let items = items_from_filter_input(input);
    let mut spec = load_spec(params)?;
    let mut roads: Vec<String> = Vec::new();
    for it in &items {
        if let Some(id) = segment_id(&it.name).filter(|id| !roads.contains(id)) {
            roads.push(id);
        }
    }
    let [road] = <[String; 1]>::try_from(roads).map_err(|r: Vec<String>| format!("Select one line of one road; {} road{} selected.", r.len(), if r.len() == 1 { " is" } else { "s are" }))?;
    let value: Value = match params["description"].as_str().map(str::trim).filter(|t| !t.is_empty()) {
        Some(raw) => serde_json::from_str(raw).map_err(|e| format!("the transition description is not valid JSON: {e}"))?,
        None => return Err("Describe the transition: change (add or drop), side (left or right) and optionally count, at, length.".into()),
    };
    let t = transition::TransitionSpec::from_json(&value)?;
    // Check it can be worked out before storing it.
    let (parsed, _, _) = spec.for_segment(&road);
    if value["remove"].as_bool() != Some(true) {
        let head = items.iter().find_map(|i| i.subs.first()).map_or([0.0, 1.0], |s| roads::heading(&s.iter().map(|&p| roads::flip(p)).collect::<Vec<_>>()));
        let here = transition::cross_section_at(&parsed, &spec.library, head, &spec.transitions_for(&road), t.piece, t.at.unwrap_or(f64::MAX));
        transition::derive_to(&here, &spec.library, head, &t)?;
    }
    spec.set_transition(&road, &t, value["remove"].as_bool() == Some(true));
    let mut out = Out::new();
    for it in &items {
        if let Some(iid) = it.id {
            out.keep(iid);
        }
    }
    Ok(out.finish_with(Some(&spec.to_value())))
}

/// Command "Set Water": the selected lines or shapes become water features (a closed shape is a lake or pond, an open line a creek or river, or a shore
/// if the description says so). Each line not yet named `water.<id>` is renamed with the next free id; the description (see `water`) is stored in the
/// document's plug-in data for each.
pub fn set_water_text(input: &Value, params: &Value) -> Result<String, String> {
    let items = items_from_filter_input(input);
    let mut spec = load_spec(params)?;
    let chosen: Vec<&Item> = items.iter().filter(|i| i.subs.first().is_some_and(|s| s.len() >= 2)).collect();
    if chosen.is_empty() {
        return Err("Select the line or shape to describe as water first.".into());
    }
    if let Some(bad) = chosen.iter().find(|i| segment_id(&i.name).is_some() || junction_id(&i.name).is_some()) {
        return Err(format!("\"{}\" is a road line; select only the water line or shape.", bad.name));
    }
    let mut value: Value = match params["description"].as_str().map(str::trim).filter(|t| !t.is_empty()) {
        Some(raw) => serde_json::from_str(raw).map_err(|e| format!("the water description is not valid JSON: {e}"))?,
        None => json!({}),
    };
    let first_closed = chosen.first().and_then(|i| i.subs.first()).is_some_and(|s| water::is_closed(s));
    if value["kind"].is_null() {
        value["kind"] = json!(if first_closed { "still" } else { "flowing" });
    }
    let ws = water::WaterSpec::from_json(&value)?;
    let mut out = Out::new();
    let mut taken: Vec<String> = spec.waters().into_iter().map(|(k, _)| k).collect();
    for it in &items {
        let Some(iid) = it.id else { continue };
        if !chosen.iter().any(|c| c.id == it.id) {
            out.keep(iid);
            continue;
        }
        ws.check_shape(it.subs.first().is_some_and(|s| water::is_closed(s)))?;
        let id = match water::water_id(&it.name) {
            Some(id) => {
                out.keep(iid);
                id
            }
            None => {
                let id = (1..).map(|n| format!("w{n}")).find(|c| !taken.contains(c)).unwrap_or_default();
                out.rename(iid, &format!("water.{id}"));
                id
            }
        };
        if !taken.contains(&id) {
            taken.push(id.clone());
        }
        spec.set_water(&id, &ws);
    }
    Ok(out.finish_with(Some(&spec.to_value())))
}

/// Command "Set Embankment": the selected line of a road stands on fill along a stretch; stores the embankment (see `embank`) in the document's plug-in data.
/// `description` is JSON (height, slope, from, to, sides, ramp, piece, remove).
pub fn set_embankment_text(input: &Value, params: &Value) -> Result<String, String> {
    let items = items_from_filter_input(input);
    let mut spec = load_spec(params)?;
    let mut roads: Vec<String> = Vec::new();
    for it in &items {
        if let Some(id) = segment_id(&it.name).filter(|id| !roads.contains(id)) {
            roads.push(id);
        }
    }
    let [road] = <[String; 1]>::try_from(roads).map_err(|r: Vec<String>| format!("Select one line of one road; {} road{} selected.", r.len(), if r.len() == 1 { " is" } else { "s are" }))?;
    let value: Value = match params["description"].as_str().map(str::trim).filter(|t| !t.is_empty()) {
        Some(raw) => serde_json::from_str(raw).map_err(|e| format!("the embankment description is not valid JSON: {e}"))?,
        None => json!({}),
    };
    let e = embank::EmbankmentSpec::from_json(&value)?;
    spec.set_embankment(&road, &e, value["remove"].as_bool() == Some(true));
    let mut out = Out::new();
    for it in &items {
        if let Some(iid) = it.id {
            out.keep(iid);
        }
    }
    Ok(out.finish_with(Some(&spec.to_value())))
}

/// Command "Set Water Crossing": select one line of a road and one water line; stores how the road meets the water (see `wet`) in the document's plug-in data.
/// `description` is JSON (treatment bridge, culvert or causeway, and its numbers); the road and water ids come from the selection.
pub fn set_wet_text(input: &Value, params: &Value) -> Result<String, String> {
    let items = items_from_filter_input(input);
    let mut spec = load_spec(params)?;
    let roads: Vec<String> = items.iter().filter_map(|i| segment_id(&i.name)).fold(Vec::new(), |mut v, id| { if !v.contains(&id) { v.push(id); } v });
    let waters: Vec<String> = items.iter().filter_map(|i| water::water_id(&i.name)).fold(Vec::new(), |mut v, id| { if !v.contains(&id) { v.push(id); } v });
    if roads.len() != 1 || waters.len() != 1 {
        return Err(format!("Select one line of one road and one water line; {} road{} and {} water feature{} selected.", roads.len(), if roads.len() == 1 { " is" } else { "s are" }, waters.len(), if waters.len() == 1 { " is" } else { "s are" }));
    }
    let (road, water_id) = (roads[0].clone(), waters[0].clone());
    let mut value: Value = match params["description"].as_str().map(str::trim).filter(|t| !t.is_empty()) {
        Some(raw) => serde_json::from_str(raw).map_err(|e| format!("the description is not valid JSON: {e}"))?,
        None => json!({}),
    };
    value["road"] = json!(road);
    value["water"] = json!(water_id);
    let w = wet::WetSpec::from_json(&value)?;
    let id = spec.wet_between(&road, &water_id).unwrap_or_else(|| (1..).map(|n| format!("x{n}")).find(|id| !spec.wets().iter().any(|(k, _)| k == id)).unwrap_or_default());
    spec.set_wet(&id, &w);
    let mut out = Out::new();
    for it in &items {
        if let Some(iid) = it.id {
            out.keep(iid);
        }
    }
    Ok(out.finish_with(Some(&spec.to_value())))
}

/// Command "Set Tracks" (Describe Track): see `rail_builder::set_tracks_text`; a record from a newer schema is refused first.
pub fn set_tracks_text(input: &Value, params: &Value) -> Result<String, String> {
    Doc::check_schema(&params["_context"]["documentData"])?;
    rail_builder::set_tracks_text(input, params).map_err(|e| crate::rail_units::metres_text(&e))
}

/// Command "Set Yard" (Describe Yard): see `rail_builder::set_yard_text`; a record from a newer schema is refused first.
pub fn set_yard_text(input: &Value, params: &Value) -> Result<String, String> {
    Doc::check_schema(&params["_context"]["documentData"])?;
    rail_builder::set_yard_text(input, params).map_err(|e| crate::rail_units::metres_text(&e))
}

/// Command "Set Joint" (Join Two Lines): see `rail_builder::set_joint_text`; a record from a newer schema is refused first.
pub fn set_joint_text(input: &Value, params: &Value) -> Result<String, String> {
    Doc::check_schema(&params["_context"]["documentData"])?;
    rail_builder::set_joint_text(input, params).map_err(|e| crate::rail_units::metres_text(&e))
}

/// Command "Set Level Crossing" (Make Level Crossing): one selected road line and one selected track line that cross; stores
/// `level_crossings[id] = {road, track, treatment}` (see `level`). `description` is JSON: `{"treatment": "level" | "gated"}`.
pub fn set_level_crossing_text(input: &Value, params: &Value) -> Result<String, String> {
    let items = items_from_filter_input(input);
    let mut spec = load_spec(params)?;
    // One road (it may be drawn as several pieces) and one track.
    let mut road_ids: Vec<String> = items.iter().filter_map(|i| segment_id(&i.name)).collect();
    let mut track_ids: Vec<String> = items.iter().filter_map(|i| rail_builder::track_id(&i.name)).collect();
    road_ids.dedup();
    track_ids.dedup();
    road_ids.sort();
    road_ids.dedup();
    track_ids.sort();
    track_ids.dedup();
    let (r, t) = match (road_ids.as_slice(), track_ids.as_slice()) {
        ([r], [t]) => (r.clone(), t.clone()),
        _ => return Err(format!("Select one road and one track that cross ({} road{} and {} track{} selected).", road_ids.len(), if road_ids.len() == 1 { "" } else { "s" }, track_ids.len(), if track_ids.len() == 1 { "" } else { "s" })),
    };
    let value: Value = match params["description"].as_str().map(str::trim).filter(|t| !t.is_empty()) {
        Some(raw) => serde_json::from_str(raw).map_err(|e| format!("the level crossing description is not valid JSON: {e}"))?,
        None => json!({}),
    };
    let treatment = level::parse_treatment(&value)?;
    let pick = |want: &dyn Fn(&str) -> bool| -> Vec<&Vec<crate::geom::P>> { items.iter().filter(|i| want(&i.name)).filter_map(|i| i.subs.first()).filter(|s| s.len() >= 2).collect() };
    let (rls, tls) = (pick(&|n| segment_id(n).as_deref() == Some(r.as_str())), pick(&|n| rail_builder::track_id(n).as_deref() == Some(t.as_str())));
    if level::find_crossing(&rls, &tls).is_none() {
        return Err(format!("Road {r} and track {t} do not cross. Draw the track across the road, then select both."));
    }
    let id = spec.level_crossings().into_iter().find(|(_, v)| v["road"] == r.as_str() && v["track"] == t.as_str()).map(|(k, _)| k)
        .unwrap_or_else(|| (1..).map(|n| format!("lc{n}")).find(|id| !spec.level_crossings().iter().any(|(k, _)| k == id)).unwrap_or_default());
    spec.set_level_crossing(&id, level::record(&r, &t, &treatment));
    let mut out = Out::new();
    for it in &items {
        if let Some(iid) = it.id {
            out.keep(iid);
        }
    }
    Ok(out.finish_with(Some(&spec.to_value())))
}

