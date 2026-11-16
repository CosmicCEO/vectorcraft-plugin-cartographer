//! The document's plug-in data: one versioned record. Keys this version does not know are kept as they were; a record from a newer
//! version is refused, never overwritten.

use crate::units::TargetScale;
use serde_json::{Map, Value, json};

/// The schema version this build reads and writes.
pub const SCHEMA: u32 = 1;

const STYLE_IDS: [&str; 4] = ["black-white", "hand-drawn", "high-contrast", "medium-contrast"];

/// How the whole document is painted (see the design, section 6.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    BlackWhite,
    HandDrawn,
    HighContrast,
    MediumContrast,
}

impl Style {
    pub fn parse(id: &str) -> Result<Style, String> {
        match id.trim().to_ascii_lowercase().as_str() {
            "black-white" => Ok(Style::BlackWhite),
            "hand-drawn" => Ok(Style::HandDrawn),
            "high-contrast" => Ok(Style::HighContrast),
            "medium-contrast" => Ok(Style::MediumContrast),
            other => Err(format!("style must be one of {} (got \"{other}\")", STYLE_IDS.join(", "))),
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            Style::BlackWhite => STYLE_IDS[0],
            Style::HandDrawn => STYLE_IDS[1],
            Style::HighContrast => STYLE_IDS[2],
            Style::MediumContrast => STYLE_IDS[3],
        }
    }
}

/// Document settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub target_print_scale: TargetScale,
    pub style: Style,
    pub seed: u32,
    pub north_deg: f64,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings { target_print_scale: TargetScale::True, style: Style::MediumContrast, seed: 1, north_deg: 0.0 }
    }
}

impl Settings {
    /// Reads settings from JSON; keys that are missing keep their defaults, a key that is present must be valid.
    pub fn from_json(v: &Value) -> Result<Settings, String> {
        let mut s = Settings::default();
        if v.is_null() {
            return Ok(s);
        }
        if !v.is_object() {
            return Err("settings must be an object".into());
        }
        if let Some(t) = v.get("target_print_scale").filter(|t| !t.is_null()) {
            s.target_print_scale = TargetScale::parse(t.as_str().unwrap_or(""))?;
        }
        if let Some(t) = v.get("style").filter(|t| !t.is_null()) {
            s.style = Style::parse(t.as_str().unwrap_or(""))?;
        }
        if let Some(t) = v.get("seed").filter(|t| !t.is_null()) {
            s.seed = t.as_u64().and_then(|n| u32::try_from(n).ok()).ok_or("seed must be a whole number from 0 to 4294967295")?;
        }
        if let Some(t) = v.get("north_deg").filter(|t| !t.is_null()) {
            s.north_deg = t.as_f64().filter(|n| (-360.0..=360.0).contains(n)).ok_or("north_deg must be a number from -360 to 360")?;
        }
        Ok(s)
    }

    pub fn to_json(&self) -> Value {
        json!({"target_print_scale": self.target_print_scale.to_text(), "style": self.style.id(), "seed": self.seed, "north_deg": self.north_deg})
    }
}

/// The whole record: the settings this version understands plus everything else, kept as it was.
pub struct Doc {
    pub settings: Settings,
    raw: Map<String, Value>,
}

/// Brings an older record up to `SCHEMA` (only 0, meaning no `schema` key, exists so far). A newer record is an error.
fn migrate(v: &Value) -> Result<Map<String, Value>, String> {
    let mut raw = match v {
        Value::Null => Map::new(),
        Value::Object(m) => m.clone(),
        _ => return Err("the document's plug-in data is not an object".into()),
    };
    let found = raw.get("schema").and_then(Value::as_u64).unwrap_or(0);
    if found > u64::from(SCHEMA) {
        return Err(format!("this document was saved by a newer Cartographer (schema {found}); update the plug-in before changing it"));
    }
    raw.insert("schema".into(), json!(SCHEMA));
    Ok(raw)
}

impl Doc {
    pub fn from_document_data(v: &Value) -> Result<Doc, String> {
        let raw = migrate(v)?;
        let settings = Settings::from_json(raw.get("settings").unwrap_or(&Value::Null))?;
        Ok(Doc { settings, raw })
    }

    pub fn to_value(&self) -> Value {
        let mut m = self.raw.clone();
        m.insert("schema".into(), json!(SCHEMA));
        m.insert("settings".into(), self.settings.to_json());
        Value::Object(m)
    }
}
