//! The Cartographer plug-in: manifest and commands. Select everything and run a command; each command keeps the selected objects.

use cartographer_core::abi::Output;
use cartographer_core::{builder, vc_plugin};
use serde_json::Value;

const MANIFEST: &str = r#"{
  "id": "com.cosmicceo.cartographer.build",
  "name": "Cartographer",
  "version": "0.1.0",
  "kind": "filter",
  "author": "CosmicCEO",
  "description": "Draws maps from lines and shapes plus a description. This foundation version has one command: Set Settings (target print scale, style, seed, north).",
  "params": {},
  "commands": [
    {
      "id": "set_settings",
      "label": "Set Settings",
      "params": {
        "description": {"type": "string", "default": "{}", "maxLength": 1000, "label": "Settings (JSON)", "help": "Any of: target_print_scale (true or 1:N), style (black-white, hand-drawn, high-contrast, medium-contrast), seed (whole number), north_deg (-360 to 360). Keys left out keep their value."}
      }
    }
  ]
}"#;

fn run(input: &[u8], params: &Value) -> Result<Output, String> {
    let input: Value = serde_json::from_slice(input).map_err(|e| format!("could not read the selection: {e}"))?;
    match params["_context"]["command"].as_str() {
        Some("set_settings") => builder::set_settings_text(&input, params).map(Output::Raw),
        _ => Err("This version has nothing to build yet. Use the command Set Settings.".into()),
    }
}

vc_plugin!(MANIFEST, run);
