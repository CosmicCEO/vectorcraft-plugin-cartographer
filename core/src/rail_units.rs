//! Railway lengths in metres of ground (records, descriptions, messages) against the model millimetres the rail code works in.
//! The track is drawn at 1:160 (N gauge): 1 metre of ground is 6.25 mm of model, 1 mm of model is 0.16 m (one document point is 1 m).
//! Schema 2 stores `tracks` and `yards` in metres; schema 1 (Cartographer 1.0 to 1.5) stored model millimetres and is upgraded on load.

use serde_json::{Map, Value, json};

/// Model millimetres per metre of ground.
pub const MM_PER_M: f64 = 6.25;

/// The record keys that hold a length.
const LENGTHS: [&str; 3] = ["spacing", "at", "length"];

fn convert(v: &Value, to_mm: bool) -> Value {
    if let Some(x) = v.as_f64() {
        return if to_mm { json!(x * MM_PER_M) } else { json!((x / MM_PER_M * 1e6).round() / 1e6) };
    }
    // Text in real units ("2m", "4000mm", "100cm", "36in") becomes a number; feet (of ground already), era names and anything else are left
    // to the rail parser.
    if let (true, Some(t)) = (to_mm, v.as_str()) {
        let t = t.trim().to_lowercase();
        let split = t.find(|c: char| !(c.is_ascii_digit() || c == '.')).unwrap_or(t.len());
        let metres_per = match t[split..].trim() {
            "m" => Some(1.0),
            "cm" => Some(0.01),
            "mm" => Some(0.001),
            "in" | "\"" => Some(0.0254),
            _ => None,
        };
        if let (Some(k), Ok(n)) = (metres_per, t[..split].parse::<f64>()) {
            return json!(n * k * MM_PER_M);
        }
    }
    v.clone()
}

fn record(v: &Value, to_mm: bool) -> Value {
    let Some(obj) = v.as_object() else { return v.clone() };
    let mut out = obj.clone();
    for k in LENGTHS {
        if let Some(x) = obj.get(k) {
            out.insert(k.into(), convert(x, to_mm));
        }
    }
    if let Some(list) = obj.get("crossovers").and_then(Value::as_array) {
        out.insert("crossovers".into(), Value::Array(list.iter().map(|c| record(c, to_mm)).collect()));
    }
    Value::Object(out)
}

/// A track or yard description or record in metres, as the rail code reads it (model millimetres).
pub fn record_to_model(v: &Value) -> Value {
    record(v, true)
}

/// A track or yard record from the rail code (model millimetres), as stored (metres).
pub fn model_to_record(v: &Value) -> Value {
    record(v, false)
}

fn each_record(data: &Map<String, Value>, to_mm: bool) -> Map<String, Value> {
    let mut out = data.clone();
    for store in ["tracks", "yards"] {
        if let Some(table) = data.get(store).and_then(Value::as_object) {
            out.insert(store.into(), Value::Object(table.iter().map(|(k, v)| (k.clone(), record(v, to_mm))).collect()));
        }
    }
    out
}

/// A document's plug-in data with its railway in metres and `schema` 2: a schema 0 or 1 record (millimetres) is converted, a newer one is
/// left alone (the schema check refuses it).
pub fn upgrade(data: &Value) -> Value {
    let Some(obj) = data.as_object() else { return data.clone() };
    if obj.get("schema").and_then(Value::as_u64).unwrap_or(0) >= 2 {
        return data.clone();
    }
    let mut out = each_record(obj, false);
    out.insert("schema".into(), json!(2));
    Value::Object(out)
}

/// The plug-in data the rail code reads: upgraded, then its `tracks` and `yards` in model millimetres.
pub fn model_data(data: &Value) -> Map<String, Value> {
    match data.as_object() {
        Some(m) if m.get("schema").and_then(Value::as_u64).unwrap_or(0) >= 2 => each_record(m, true),
        Some(_) => match upgrade(data) {
            Value::Object(m) => each_record(&m, true),
            _ => Map::new(),
        },
        None => Map::new(),
    }
}

/// The plug-in data the rail code wrote (millimetres), back in metres for storing.
pub fn stored_data(model: &Map<String, Value>) -> Map<String, Value> {
    each_record(model, false)
}

/// A message from the rail code with every "N mm" said in metres of ground ("2524 mm" becomes "403.84 m").
pub fn metres_text(s: &str) -> String {
    let s = s.replace(" prototype =", " =");
    let mut out = String::with_capacity(s.len());
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let starts = (b[i].is_ascii_digit() || b[i] == b'-') && (i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'.'));
        if starts {
            let mut j = i + 1;
            while j < b.len() && (b[j].is_ascii_digit() || b[j] == b'.') {
                j += 1;
            }
            let unit = s[j..].strip_prefix(" mm").filter(|rest| !rest.starts_with(|c: char| c.is_ascii_alphanumeric()));
            if let (Some(_), Ok(x)) = (unit, s[i..j].parse::<f64>()) {
                let m = format!("{:.2}", x / MM_PER_M);
                out.push_str(m.trim_end_matches('0').trim_end_matches('.'));
                out.push_str(" m");
                i = j + 3;
                continue;
            }
            out.push_str(&s[i..j]);
            i = j;
            continue;
        }
        let ch = s[i..].chars().next().unwrap_or(' ');
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}
