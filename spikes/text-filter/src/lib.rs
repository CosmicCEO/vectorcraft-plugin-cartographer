//! Spike S4: a filter that returns the kept objects plus one new text object of the given kind (`params.kind`: "path", "point" or "area").
mod abi;

use abi::Output;
use serde_json::{Value, json};

const MANIFEST: &str = r#"{
  "id": "com.cosmicceo.spike.text",
  "name": "Spike text",
  "version": "0.0.0",
  "kind": "filter",
  "author": "CosmicCEO",
  "description": "Throwaway spike: adds one text object.",
  "params": {"kind": {"type": "string", "default": "point", "maxLength": 8}}
}"#;

fn run(input: &[u8], params: &Value) -> Result<Output, String> {
    let kind = params["kind"].as_str().unwrap_or("point");
    let input: Value = serde_json::from_slice(input).map_err(|e| e.to_string())?;
    let mut objects: Vec<Value> = input["objects"].as_array().map(|a| a.iter().map(|o| json!({"id": o["id"]})).collect()).unwrap_or_default();
    objects.push(json!({
        "name": format!("gen.label.spike.{kind}"),
        "type": "type",
        "transform": [1, 0, 0, 1, 100, 100],
        "text": {"kind": kind, "vertical": false,
                 "runs": [{"text": "Cedar Creek", "style": {"font_family": "Source Sans 3", "size": 12}}],
                 "paragraphs": [{"justify": "Left"}]}
    }));
    Ok(Output::Json(json!({"objects": objects})))
}

vc_plugin!(MANIFEST, run);
