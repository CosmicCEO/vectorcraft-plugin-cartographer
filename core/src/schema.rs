//! What the panel needs to draw a card: the fields of a kind of description, with their types, limits and help.

use crate::storage::Style;
use crate::tracks::TrackSpec;
use crate::yard::YardSpec;
use serde_json::{Value, json};

/// The fields of the document settings.
pub fn settings_schema() -> Value {
    let styles = [Style::BlackWhite, Style::HandDrawn, Style::HighContrast, Style::MediumContrast].map(|s| s.id());
    json!({"kind": "settings", "fields": [
        {"key": "target_print_scale", "label": "Target print scale", "type": "text", "default": "true",
         "help": "true, or 1:N such as 1:5000. With a scale, small things get a minimum print size when you rebuild."},
        {"key": "style", "label": "Style", "type": "choice", "options": styles, "default": "medium-contrast",
         "help": "How the whole map is painted."},
        {"key": "seed", "label": "Random seed", "type": "integer", "min": 0, "max": 4294967295u64, "default": 1,
         "help": "Fixes the hand-drawn look so a rebuild comes out the same."},
        {"key": "north_deg", "label": "North (degrees)", "type": "number", "min": -360, "max": 360, "default": 0,
         "help": "Which way north points, clockwise from straight up the page."}
    ]})
}

/// A choice between on and off, for a true/false field.
fn on_off(key: &str, label: &str, help: &str) -> Value {
    json!({"key": key, "label": label, "type": "choice", "options": ["on", "off"], "default": "on", "help": help})
}

/// The fields of a description of parallel tracks (`parse-tracks`).
pub fn tracks_schema() -> Value {
    let d = TrackSpec::default().to_json();
    json!({"kind": "tracks", "fields": [
        {"key": "tracks", "label": "Tracks", "type": "integer", "min": 1, "max": crate::tracks::MAX_TRACKS, "default": d["tracks"],
         "help": "How many parallel tracks the line carries, 1 to 12."},
        {"key": "spacing", "label": "Spacing (m)", "type": "number", "default": crate::rail_units::model_to_record(&d)["spacing"],
         "help": "Distance between track centres, in metres. Words such as 14ft or classic work in the words box."},
        on_off("ballast", "Roadbed", "Draws the roadbed under the track."),
        on_off("ties", "Ties", "Draws the ties.")
    ]})
}

/// The fields of a yard description (`parse-yard`). Crossovers stay in the words box.
pub fn yard_schema() -> Value {
    let d = YardSpec::default();
    json!({"kind": "yard", "fields": [
        {"key": "mains", "label": "Main tracks", "type": "integer", "min": 1, "max": crate::yard::MAX_MAINS, "default": d.mains, "help": "Through tracks along the line, 1 to 4."},
        {"key": "left", "label": "Spurs on the left", "type": "integer", "min": 0, "max": crate::yard::MAX_SPURS, "default": d.left, "help": "Spurs on the left as seen from the start of the line to its end."},
        {"key": "right", "label": "Spurs on the right", "type": "integer", "min": 0, "max": crate::yard::MAX_SPURS, "default": d.right, "help": "Spurs on the right as seen from the start of the line to its end."},
        {"key": "ends", "label": "Far end", "type": "choice", "options": ["merge", "stub"], "default": if d.merge { "merge" } else { "stub" },
         "help": "merge: the spurs join the mains again at the far end. stub: they end."},
        {"key": "frog", "label": "Turnout number", "type": "integer", "min": 4, "max": 20, "default": d.frog, "help": "The frog number of every turnout, 4 to 20 (the numbers model turnouts are sold in)."},
        {"key": "spacing", "label": "Spacing (m)", "type": "number", "default": crate::rail_units::model_to_record(&d.to_json())["spacing"], "help": "Distance between track centres, in metres."},
        {"key": "at", "label": "Starts at (m)", "type": "number", "min": 0, "default": crate::rail_units::model_to_record(&d.to_json())["at"], "help": "Where the first turnout starts, metres along the line."},
        {"key": "length", "label": "Length (m)", "type": "number", "min": 0, "default": Value::Null, "help": "How far the yard runs, metres; empty is to the end of the line."},
        on_off("ballast", "Roadbed", "Draws the roadbed under the yard."),
        on_off("ties", "Ties", "Draws the ties.")
    ]})
}

/// The fields of a joint between two track lines (`parse-joint`).
pub fn joint_schema() -> Value {
    json!({"kind": "joint", "fields": [
        {"key": "kind", "label": "How they meet", "type": "choice", "options": ["crossing", "branch", "join"], "default": "crossing",
         "help": "crossing: two tracks cross. branch: one line leaves the other with a turnout. join: two lines continue into each other end to end."},
        {"key": "main", "label": "Line that runs through", "type": "text", "default": "", "help": "The id of the line whose track runs through; empty lets the program choose."},
        {"key": "frog", "label": "Turnout number", "type": "integer", "min": 4, "max": 20, "default": Value::Null, "help": "For a branch: the frog number, 4 to 20; empty is the one that makes the gentlest connection."}
    ]})
}

/// The fields of a level crossing (a road and a track; `set_level_crossing`).
pub fn level_crossing_schema() -> Value {
    json!({"kind": "level_crossing", "fields": [
        {"key": "treatment", "label": "Crossing", "type": "choice", "options": crate::level::TREATMENTS, "default": "level",
         "help": "level: the road crosses the track on a panel, the rails run across. gated: also a crossbuck and a gate arm on each approach. road over: a road bridge over the track. track over: a rail bridge over the road."}
    ]})
}

