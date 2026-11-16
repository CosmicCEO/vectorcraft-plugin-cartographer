//! What the panel needs to draw a card: the fields of a kind of description, with their types, limits and help.

use crate::storage::Style;
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
