//! The material library: what a band of a map element is made of (see the design, section 6.2). Widths are metres of ground, minimum
//! print widths are millimetres on paper.

use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Surface,
    Edge,
    Water,
    Bank,
    Vegetation,
    Structure,
    Mark,
}

impl Role {
    fn parse(s: &str) -> Option<Role> {
        Some(match s {
            "surface" => Role::Surface,
            "edge" => Role::Edge,
            "water" => Role::Water,
            "bank" => Role::Bank,
            "vegetation" => Role::Vegetation,
            "structure" => Role::Structure,
            "mark" => Role::Mark,
            _ => return None,
        })
    }
}

/// How well a material's numbers are sourced (as in the standards database).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Verified,
    VerifiedSecondary,
    Unverified,
}

impl Status {
    fn parse(s: &str) -> Option<Status> {
        Some(match s {
            "verified" => Status::Verified,
            "verified-secondary" => Status::VerifiedSecondary,
            "unverified" => Status::Unverified,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub id: String,
    pub role: Role,
    pub default_width: f64,
    pub min_print_width: f64,
    /// A dotted key the style sheets paint, such as `earth.sand`.
    pub class: String,
    pub height: Option<f64>,
    pub source: Option<String>,
    pub status: Status,
}

/// Reads `{"materials": [...]}`. An unknown role or status is an error that names it.
pub fn load(text: &str) -> Result<Vec<Material>, String> {
    let v: Value = serde_json::from_str(text).map_err(|e| format!("the library is not valid JSON: {e}"))?;
    let list = v["materials"].as_array().ok_or("the library needs a \"materials\" list")?;
    list.iter()
        .map(|m| {
            let id = m["id"].as_str().ok_or("a material needs an \"id\"")?.to_string();
            let role = Role::parse(m["role"].as_str().unwrap_or("")).ok_or_else(|| format!("material {id}: unknown role \"{}\"", m["role"].as_str().unwrap_or("")))?;
            let status = Status::parse(m["status"].as_str().unwrap_or("")).ok_or_else(|| format!("material {id}: unknown status \"{}\"", m["status"].as_str().unwrap_or("")))?;
            Ok(Material {
                role,
                status,
                default_width: m["default_width"].as_f64().unwrap_or(0.0),
                min_print_width: m["min_print_width"].as_f64().unwrap_or(0.0),
                class: m["class"].as_str().unwrap_or("").to_string(),
                height: m["height"].as_f64(),
                source: m["source"].as_str().map(str::to_string),
                id,
            })
        })
        .collect()
}

/// Problems with a library, in plain words; empty means valid.
pub fn validate(lib: &[Material]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for m in lib {
        if seen.contains(&m.id.as_str()) {
            problems.push(format!("material {}: duplicate id", m.id));
        }
        seen.push(&m.id);
        if m.default_width.is_nan() || m.default_width <= 0.0 {
            problems.push(format!("material {}: default_width must be more than 0", m.id));
        }
        if m.min_print_width.is_nan() || m.min_print_width < 0.0 {
            problems.push(format!("material {}: min_print_width must not be negative", m.id));
        }
        if !m.class.contains('.') || m.class.starts_with('.') || m.class.ends_with('.') {
            problems.push(format!("material {}: class \"{}\" must be dotted, like earth.sand", m.id, m.class));
        }
        if m.status != Status::Unverified && m.source.as_deref().is_none_or(str::is_empty) {
            problems.push(format!("material {}: a verified entry needs a source", m.id));
        }
    }
    problems
}

/// The library that ships with the plug-in.
pub fn builtin() -> Vec<Material> {
    load(include_str!("../../library/materials.json")).expect("library/materials.json is valid")
}
