//! The plug-in's commands. Each takes the filter input and parameters and returns the output text.

use crate::output::Out;
use crate::storage::{Doc, Settings};
use crate::vcdoc::items_from_filter_input;
use serde_json::{Value, json};

/// A short sentence for a card: `style hand-drawn, target print scale 1:5000, seed 7, north 10 degrees`.
pub fn settings_summary(s: &Settings) -> String {
    format!("style {}, target print scale {}, seed {}, north {} degrees", s.style.id(), s.target_print_scale.to_text(), s.seed, s.north_deg)
}

/// Command "Set Settings": changes the document settings named in `description` (a JSON object; keys left out keep their value), keeps every
/// selected object, and writes the record back. A record saved by a newer Cartographer is refused.
pub fn set_settings_text(input: &Value, params: &Value) -> Result<String, String> {
    let mut doc = Doc::from_document_data(&params["_context"]["documentData"])?;
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
