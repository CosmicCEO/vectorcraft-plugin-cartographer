//! The document's plug-in data: one versioned record. Keys this version does not know are kept as they were; a record from a newer
//! version is refused, never overwritten.

use crate::units::TargetScale;
use serde_json::{Map, Value, json};

/// The schema version this build reads and writes.
pub const SCHEMA: u32 = 2;

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

const SETTING_KEYS: [&str; 4] = ["target_print_scale", "style", "seed", "north_deg"];

impl Settings {
    /// For a command or a proposal: a key that is not a setting is an error naming the settings (a misspelt key would otherwise change nothing).
    pub fn check_keys(v: &Value) -> Result<(), String> {
        if let Some(map) = v.as_object() {
            if let Some(bad) = map.keys().find(|k| !SETTING_KEYS.contains(&k.as_str())) {
                return Err(format!("unknown setting \"{bad}\"; the settings are {}", SETTING_KEYS.join(", ")));
            }
        }
        Ok(())
    }

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
            s.target_print_scale = TargetScale::parse(t.as_str().ok_or("target_print_scale must be text: true or 1:N")?)?;
        }
        if let Some(t) = v.get("style").filter(|t| !t.is_null()) {
            s.style = Style::parse(t.as_str().ok_or("style must be text")?)?;
        }
        if let Some(t) = v.get("seed").filter(|t| !t.is_null()) {
            s.seed = t.as_u64().and_then(|n| u32::try_from(n).ok()).ok_or("seed must be a whole number from 0 to 4294967295")?;
        }
        if let Some(t) = v.get("north_deg").filter(|t| !t.is_null()) {
            s.north_deg = t.as_f64().filter(|n| (-360.0..=360.0).contains(n)).ok_or("north_deg must be a number from -360 to 360")?;
        }
        Ok(s)
    }

    /// Reads settings one key at a time: a key that is invalid is left at its default and named in the second value, so one bad field never
    /// turns off the others. A value that is not an object gives the defaults and one problem; keys that are not settings are ignored.
    pub fn from_json_lenient(v: &Value) -> (Settings, Vec<String>) {
        let mut s = Settings::default();
        let mut problems = Vec::new();
        if v.is_null() {
            return (s, problems);
        }
        let Some(map) = v.as_object() else {
            return (s, vec!["the stored settings are not an object, so the defaults were used".into()]);
        };
        for key in SETTING_KEYS {
            let Some(value) = map.get(key).filter(|t| !t.is_null()) else { continue };
            match Settings::from_json(&json!({ key: value })) {
                Ok(one) => match key {
                    "target_print_scale" => s.target_print_scale = one.target_print_scale,
                    "style" => s.style = one.style,
                    "seed" => s.seed = one.seed,
                    _ => s.north_deg = one.north_deg,
                },
                Err(e) => problems.push(format!("setting {key} was ignored: {e}")),
            }
        }
        (s, problems)
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

/// Brings an older record up to `SCHEMA` (0 means no `schema` key; 1 kept the railway in model millimetres). A newer record is an error.
fn migrate(v: &Value) -> Result<Map<String, Value>, String> {
    let raw = match v {
        Value::Null => Map::new(),
        Value::Object(m) => m.clone(),
        _ => return Err("the document's plug-in data is not an object".into()),
    };
    let found = match raw.get("schema") {
        None | Some(Value::Null) => 0,
        Some(v) => v.as_u64().ok_or("the document's schema must be a whole number; this data cannot be read safely")?,
    };
    if found > u64::from(SCHEMA) {
        return Err(format!("this document was saved by a newer Cartographer (schema {found}); update the plug-in before changing it"));
    }
    // Schema 2: the railway is stored in metres (1 kept model millimetres).
    let mut raw = if found < 2 {
        match crate::rail_units::upgrade(&Value::Object(raw)) {
            Value::Object(m) => m,
            _ => Map::new(),
        }
    } else {
        raw
    };
    raw.insert("schema".into(), json!(SCHEMA));
    Ok(raw)
}

impl Doc {
    /// Only checks that the record is one this version may read (a newer schema is refused); the settings are not looked at.
    pub fn check_schema(v: &Value) -> Result<(), String> {
        migrate(v).map(|_| ())
    }

    /// Like `from_document_data`, but stored settings that cannot be read (a value from a later version, or a hand edit) become the defaults
    /// instead of an error, so a settings command can repair them.
    pub fn from_document_data_lenient(v: &Value) -> Result<Doc, String> {
        let raw = migrate(v)?;
        let settings = Settings::from_json(raw.get("settings").unwrap_or(&Value::Null)).unwrap_or_default();
        Ok(Doc { settings, raw })
    }

    pub fn from_document_data(v: &Value) -> Result<Doc, String> {
        let raw = migrate(v)?;
        let settings = Settings::from_json(raw.get("settings").unwrap_or(&Value::Null))?;
        Ok(Doc { settings, raw })
    }

    pub fn to_value(&self) -> Value {
        let mut m = self.raw.clone();
        m.insert("schema".into(), json!(SCHEMA));
        // Settings keys this version does not know (a later version's, same schema) stay as they were.
        let mut settings = m.get("settings").and_then(Value::as_object).cloned().unwrap_or_default();
        if let Some(known) = self.settings.to_json().as_object() {
            settings.extend(known.iter().map(|(k, v)| (k.clone(), v.clone())));
        }
        m.insert("settings".into(), Value::Object(settings));
        Value::Object(m)
    }
}
