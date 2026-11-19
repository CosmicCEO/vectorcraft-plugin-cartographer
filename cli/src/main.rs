//! `cartographer-cli`: parse, validate and edit descriptions (cross-sections, intersections, water ...) and print schemas. Always prints one line of JSON; exit code 1 when `ok` is false.
//!
//! ```text
//! cartographer-cli parse "facing N: bike-lane | lane S x2 | margin 3 | lane N x2 | bike-lane"
//! cartographer-cli edit  "<text>" delete 0 | move 0 5 | width 2 1.5 | insert 3 "turn-pocket N"
//! cartographer-cli library
//! ```
//! `parse` and `edit` print `{"ok": true, "canonical", "facing", "items": [...], "total_width", "notes"}` or `{"ok": false, "error"}`.

use cartographer_core::builder::settings_summary;
use cartographer_core::crosssection::{Item, Parsed, format, parse};
use cartographer_core::junction::{Control, JunctionSpec};
use cartographer_core::joint::{self, JointSpec};
use cartographer_core::library::Library;
use cartographer_core::rail_builder::to_mm;
use cartographer_core::schema::{joint_schema, level_crossing_schema, settings_schema, tracks_schema, yard_schema};
use cartographer_core::storage::Settings;
use cartographer_core::tracks::TrackSpec;
use cartographer_core::vcdoc::subpaths_of;
use cartographer_core::yard::YardSpec;
use serde_json::{Value, json};

fn describe(p: &Parsed, lib: &Library) -> Value {
    let mut total = 0.0;
    let items: Vec<Value> = p
        .items
        .iter()
        .enumerate()
        .map(|(i, it)| {
            let def = lib.get(&it.kind);
            let width = it.width.or(def.map(|d| d.width)).unwrap_or(0.0);
            total += width;
            json!({
                "index": i,
                "kind": it.kind,
                "label": def.map_or(it.kind.as_str(), |d| d.label.as_str()),
                "way": it.way.map(|w| w.letter()),
                "width": (width * 1000.0).round() / 1000.0,
                "explicit_width": it.width.is_some(),
                "directional": def.is_some_and(|d| d.directional),
                "fill": def.map(|d| d.fill),
            })
        })
        .collect();
    json!({"ok": true, "canonical": format(p), "facing": p.facing.map(|w| w.letter()), "items": items, "total_width": (total * 1000.0).round() / 1000.0, "notes": p.notes})
}

fn label(it: &Item, lib: &Library) -> String {
    lib.get(&it.kind).map_or(it.kind.clone(), |d| d.label.to_lowercase())
}

fn width(it: &Item, lib: &Library) -> f64 {
    it.width.or(lib.get(&it.kind).map(|d| d.width)).unwrap_or(0.0)
}

fn way_word(w: Option<cartographer_core::crosssection::Way>) -> &'static str {
    use cartographer_core::crosssection::Way::*;
    match w {
        Some(North) => "northbound",
        Some(South) => "southbound",
        Some(East) => "eastbound",
        Some(West) => "westbound",
        Some(Fwd) => "along the line",
        Some(Back) => "against the line",
        None => "no direction",
    }
}

/// The differences between two cross-sections as plain sentences: elements are matched in order by kind (longest common subsequence),
/// so a removal, an addition, a change of direction or width, and a different reading side each show up on their own line.
fn changes(a: &Parsed, b: &Parsed, lib: &Library) -> Vec<String> {
    let (n, m) = (a.items.len(), b.items.len());
    let mut t = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            t[i][j] = if a.items[i].kind == b.items[j].kind { t[i + 1][j + 1] + 1 } else { t[i + 1][j].max(t[i][j + 1]) };
        }
    }
    let mut out = Vec::new();
    if a.facing != b.facing {
        out.push(format!("reading side changed from {} to {}", a.facing.map_or("the line", |w| w.letter()), b.facing.map_or("the line", |w| w.letter())));
    }
    let (mut i, mut j) = (0, 0);
    while i < n || j < m {
        if i < n && j < m && a.items[i].kind == b.items[j].kind && t[i][j] == t[i + 1][j + 1] + 1 {
            let (x, y) = (&a.items[i], &b.items[j]);
            if x.way != y.way {
                out.push(format!("{}: direction {} to {}", label(x, lib), way_word(x.way), way_word(y.way)));
            }
            if (width(x, lib) - width(y, lib)).abs() > 1e-9 {
                out.push(format!("{}: width {} m to {} m", label(x, lib), width(x, lib), width(y, lib)));
            }
            i += 1;
            j += 1;
        } else if j < m && (i == n || t[i][j + 1] >= t[i + 1][j]) {
            out.push(format!("added {} ({} m{})", label(&b.items[j], lib), width(&b.items[j], lib), if b.items[j].way.is_some() { format!(", {}", way_word(b.items[j].way)) } else { String::new() }));
            j += 1;
        } else {
            out.push(format!("removed {} ({} m{})", label(&a.items[i], lib), width(&a.items[i], lib), if a.items[i].way.is_some() { format!(", {}", way_word(a.items[i].way)) } else { String::new() }));
            i += 1;
        }
    }
    out
}

fn fail(msg: impl Into<String>) -> Value {
    json!({"ok": false, "error": msg.into()})
}

fn index(args: &[String], at: usize, what: &str) -> Result<usize, String> {
    args.get(at).and_then(|a| a.parse::<usize>().ok()).ok_or_else(|| format!("{what}: give a whole number (items are numbered from 0)"))
}

/// `<command> '<json>'`: checks a track, yard or joint description with `check`.
fn spec_reply(args: &[String], name: &str, check: impl Fn(&Value) -> Result<Value, String>) -> Value {
    let Some(text) = args.get(1) else { return fail(format!("usage: {name} '<json>'")) };
    match serde_json::from_str::<Value>(text) {
        Ok(v) => check(&v).unwrap_or_else(fail),
        Err(e) => fail(format!("not valid JSON: {e}")),
    }
}

/// `find-joint '<path a>' '<path b>' [id a] [id b]`: how two lines (paths as VectorCraft stores them, document points) meet.
fn find_joint(args: &[String]) -> Value {
    let (Some(a), Some(b)) = (args.get(1), args.get(2)) else { return fail("give two paths: find-joint '<path a>' '<path b>' [id a] [id b]") };
    let line = |raw: &str| serde_json::from_str::<Value>(raw).map(|v| subpaths_of(&v).into_iter().next().unwrap_or_default().into_iter().map(to_mm).collect::<Vec<_>>()).map_err(|e| format!("not valid JSON: {e}"));
    let (pa, pb) = match (line(a), line(b)) {
        (Ok(pa), Ok(pb)) => (pa, pb),
        (Err(e), _) | (_, Err(e)) => return fail(e),
    };
    let (na, nb) = (args.get(3).map_or("a", String::as_str), args.get(4).map_or("b", String::as_str));
    match joint::find(&pa, &pb) {
        Some(f) => json!({"ok": true, "found": f.to_json(), "kind": f.kind(), "summary": f.describe(na, nb)}),
        None => fail(format!("Lines {na} and {nb} do not meet (nowhere do they come within {:.0} mm of each other).", joint::TOLERANCE)),
    }
}

fn run(args: &[String], lib: &Library) -> Value {
    match args.first().map(String::as_str) {
        Some("parse") => match args.get(1) {
            Some(text) => parse(text, lib).map_or_else(fail, |p| describe(&p, lib)),
            None => fail("usage: parse \"<cross-section text>\""),
        },
        Some("edit") => {
            let Some(text) = args.get(1) else { return fail("usage: edit \"<text>\" delete|move|width|insert ...") };
            let mut p = match parse(text, lib) {
                Ok(p) => p,
                Err(e) => return fail(e),
            };
            let done = match args.get(2).map(String::as_str) {
                Some("delete") => index(args, 3, "delete").and_then(|i| p.delete(i)),
                Some("move") => index(args, 3, "move from").and_then(|a| index(args, 4, "move to").and_then(|b| p.move_item(a, b))),
                Some("width") => index(args, 3, "width").and_then(|i| match args.get(4).and_then(|w| w.parse::<f64>().ok()) {
                    Some(w) => p.set_width(i, w),
                    None => Err("width: give the new width in metres".into()),
                }),
                Some("insert") => index(args, 3, "insert at").and_then(|i| match args.get(4) {
                    Some(t) => p.insert(i, t, lib),
                    None => Err("insert: give the element text, like \"turn-pocket N\"".into()),
                }),
                _ => Err("edit needs one of: delete <i>, move <from> <to>, width <i> <metres>, insert <i> \"<text>\"".into()),
            };
            done.map_or_else(fail, |()| describe(&p, lib))
        }
        Some("diff") => match (args.get(1).map(|t| parse(t, lib)), args.get(2).map(|t| parse(t, lib))) {
            (Some(Ok(a)), Some(Ok(b))) => json!({"ok": true, "changes": changes(&a, &b, lib)}),
            (Some(Err(e)), _) => fail(format!("current: {e}")),
            (_, Some(Err(e))) => fail(format!("proposed: {e}")),
            _ => fail("usage: diff \"<current>\" \"<proposed>\""),
        },
        Some("parse-junction") => {
            let Some(text) = args.get(1) else { return fail("usage: parse-junction '<junction JSON>'") };
            let value: Value = match serde_json::from_str(text) {
                Ok(v) => v,
                Err(e) => return fail(format!("not valid JSON: {e}")),
            };
            JunctionSpec::from_json(&value, lib).map_or_else(fail, |j| {
                let legs: Vec<String> = j.legs.iter().map(|l| format!("{}: {} ({} m)", l.side, l.cross_section, l.length)).collect();
                let control = match j.control { Control::None => "no control", Control::Signal => "traffic signals", Control::Stop => "stop signs on the legs" };
                let summary = format!("{}, {control}{}{}", match j.kind() { "tee" => "T-junction", "cross" => "four-way junction", _ => "road carried across the gap" }, if j.crosswalks { ", crosswalks" } else { "" }, if legs.is_empty() { String::new() } else { format!("; legs {}", legs.join("; ")) });
                json!({"ok": true, "junction": j.to_json(), "summary": summary})
            })
        }
        Some("parse-crossing") => {
            let Some(text) = args.get(1) else { return fail("usage: parse-crossing '<crossing JSON>'") };
            let value: Value = match serde_json::from_str(text) {
                Ok(v) => v,
                Err(e) => return fail(format!("not valid JSON: {e}")),
            };
            cartographer_core::crossing::CrossingSpec::from_json(&value).map_or_else(fail, |c| json!({"ok": true, "crossing": c.to_json(), "summary": c.summary()}))
        }
        Some("parse-hub") => {
            let Some(text) = args.get(1) else { return fail("usage: parse-hub '<hub JSON>'") };
            let value: Value = match serde_json::from_str(text) {
                Ok(v) => v,
                Err(e) => return fail(format!("not valid JSON: {e}")),
            };
            cartographer_core::hub::HubSpec::from_json(&value).map_or_else(fail, |h| json!({"ok": true, "hub": h.to_json(), "summary": h.summary()}))
        }
        Some("parse-transition") => {
            let Some(text) = args.get(1) else { return fail("usage: parse-transition '<transition JSON>' [\"current cross-section\"]") };
            let value: Value = match serde_json::from_str(text) {
                Ok(v) => v,
                Err(e) => return fail(format!("not valid JSON: {e}")),
            };
            match cartographer_core::transition::TransitionSpec::from_json(&value) {
                Err(e) => fail(e),
                Ok(t) => {
                    let after = args.get(2).map(|cs| parse(cs, lib).and_then(|p| cartographer_core::transition::describe_to(&p, lib, [0.0, 1.0], &t)));
                    match after {
                        Some(Err(e)) => fail(e),
                        Some(Ok(text)) => json!({"ok": true, "transition": t.to_json(), "summary": t.summary(), "after": text}),
                        None => json!({"ok": true, "transition": t.to_json(), "summary": t.summary()}),
                    }
                }
            }
        }
        Some("parse-water") => {
            let Some(text) = args.get(1) else { return fail("usage: parse-water '<water JSON>' [open|closed]") };
            let value: Value = match serde_json::from_str(text) {
                Ok(v) => v,
                Err(e) => return fail(format!("not valid JSON: {e}")),
            };
            let closed = args.get(2).map(|a| a == "closed");
            cartographer_core::water::WaterSpec::from_json(&value).and_then(|w| match closed { Some(c) => w.check_shape(c).map(|()| w), None => Ok(w) }).map_or_else(fail, |w| json!({"ok": true, "water": w.to_json(), "summary": w.summary()}))
        }
        Some("parse-embankment") => {
            let Some(text) = args.get(1) else { return fail("usage: parse-embankment '<embankment JSON>'") };
            let value: Value = match serde_json::from_str(text) {
                Ok(v) => v,
                Err(e) => return fail(format!("not valid JSON: {e}")),
            };
            cartographer_core::embank::EmbankmentSpec::from_json(&value).map_or_else(fail, |e| json!({"ok": true, "embankment": e.to_json(), "summary": e.summary()}))
        }
        Some("parse-wet") => {
            let Some(text) = args.get(1) else { return fail("usage: parse-wet '<crossing JSON>'") };
            let value: Value = match serde_json::from_str(text) {
                Ok(v) => v,
                Err(e) => return fail(format!("not valid JSON: {e}")),
            };
            cartographer_core::wet::WetSpec::from_json(&value).map_or_else(fail, |w| json!({"ok": true, "wet": w.to_json(), "summary": w.summary()}))
        }
        Some("version") => json!({"ok": true, "version": cartographer_core::VERSION}),
        Some("parse-settings") => {
            let Some(text) = args.get(1) else { return fail("usage: parse-settings '<settings JSON>'") };
            let value: Value = match serde_json::from_str(text) {
                Ok(v) => v,
                Err(e) => return fail(format!("not valid JSON: {e}")),
            };
            if let Err(e) = Settings::check_keys(&value) {
                return fail(e);
            }
            match Settings::from_json(&value) {
                Ok(s) => json!({"ok": true, "settings": s.to_json(), "summary": settings_summary(&s)}),
                Err(e) => fail(e),
            }
        }
        Some("schema") => match args.get(1).map(String::as_str) {
            Some("settings") => {
                let mut v = settings_schema();
                v["ok"] = json!(true);
                v
            }
            Some(kind @ ("tracks" | "yard" | "joint" | "level_crossing")) => {
                let mut v = match kind {
                    "tracks" => tracks_schema(),
                    "yard" => yard_schema(),
                    "level_crossing" => level_crossing_schema(),
                    _ => joint_schema(),
                };
                v["ok"] = json!(true);
                v
            }
            _ => fail("usage: schema settings | tracks | yard | joint | level_crossing"),
        },
        Some("parse-tracks") => spec_reply(args, "parse-tracks", |v| TrackSpec::from_json(v).map(|s| json!({"ok": true, "tracks": s.to_json(), "summary": s.summary()}))),
        Some("parse-yard") => spec_reply(args, "parse-yard", |v| YardSpec::from_json(v).map(|s| json!({"ok": true, "yard": s.to_json(), "summary": s.summary(), "min_line_mm": (s.min_line() * 10.0).round() / 10.0}))),
        Some("parse-joint") => spec_reply(args, "parse-joint", |v| JointSpec::from_json(v).map(|s| json!({"ok": true, "joint": s.to_json(), "summary": s.summary()}))),
        Some("find-joint") => find_joint(args),
        Some("library") => {
            let els: Vec<Value> = lib
                .names()
                .into_iter()
                .filter_map(|n| lib.get(n))
                .map(|d| json!({"id": d.id, "label": d.label, "width": d.width, "directional": d.directional, "fill": d.fill, "source": d.source}))
                .collect();
            json!({"ok": true, "elements": els})
        }
        _ => fail("usage: cartographer-cli parse|edit|diff|parse-junction|parse-crossing|parse-hub|parse-transition|parse-water|parse-embankment|parse-wet|parse-tracks|parse-yard|parse-joint|find-joint|library|version|parse-settings|schema ..."),
    }
}

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = run(&args, &Library::builtin());
    println!("{out}");
    if out["ok"] == json!(true) { std::process::ExitCode::SUCCESS } else { std::process::ExitCode::FAILURE }
}
