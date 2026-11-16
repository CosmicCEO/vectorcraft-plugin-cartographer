//! The Cartographer plug-in: manifest and commands. Select everything and run it (Build), or run a command that describes the selected line.

use cartographer_core::abi::Output;
use cartographer_core::{builder, vc_plugin};
use serde_json::Value;

const MANIFEST: &str = r#"{
  "id": "com.cosmicceo.cartographer.build",
  "name": "Cartographer",
  "version": "0.2.0",
  "kind": "filter",
  "author": "CosmicCEO",
  "description": "Draws maps from lines and shapes plus a description: a line named road.<id> becomes the road described by its cross-section; roads cross in intersections; water, embankments, bridges and culverts. Select All first, so everything is included.",
  "params": {},
  "commands": [
    {
      "id": "set_settings",
      "label": "Set Settings",
      "params": {
        "description": {
          "type": "string",
          "default": "{}",
          "maxLength": 1000,
          "label": "Settings (JSON)",
          "help": "Any of: target_print_scale (true or 1:N), style (black-white, hand-drawn, high-contrast, medium-contrast), seed (whole number), north_deg (-360 to 360). Keys left out keep their value."
        }
      }
    },
    {
      "id": "set_cross_section",
      "label": "Describe Road (Cross-section)",
      "params": {
        "cross_section": {
          "type": "string",
          "default": "lane back | lane fwd",
          "maxLength": 1000,
          "label": "Cross-section",
          "help": "Left to right, separated by |. Each item: an element (lane, bike-lane, apron, shoulder, margin, median, turn-pocket, sidewalk ...), optional direction (N S E W fwd back), width (3, 3m, 10ft), repeat (x2). Start with 'facing N:' to say which way the list reads. Example: facing N: bike-lane | apron | lane S x2 | margin 3 | lane N x2 | apron | bike-lane"
        },
        "segment": {
          "type": "string",
          "default": "",
          "maxLength": 16,
          "label": "Road id",
          "help": "Leave empty to use the selected line (a plain line becomes a new road)."
        },
        "anchor": {
          "type": "string",
          "default": "",
          "maxLength": 24,
          "label": "Line follows",
          "help": "Which part of the road the drawn line follows: auto (the middle of the median or margin if there is one, else the middle of the road), center, left, right, or an element name such as margin. Leave empty to keep the current setting."
        },
        "scope": {
          "type": "choice",
          "options": [
            "segment",
            "selected"
          ],
          "default": "segment",
          "help": "segment: every piece of this road. selected: only the selected pieces (they get an id of their own, e.g. after you broke the line and want to change one piece)."
        }
      }
    },
    {
      "id": "set_junction",
      "label": "Describe Junction",
      "params": {
        "description": {
          "type": "string",
          "default": "{\"legs\": [{\"side\": \"left\", \"cross_section\": \"lane back | lane fwd\"}, {\"side\": \"right\", \"cross_section\": \"lane back | lane fwd\"}], \"control\": \"signal\", \"crosswalks\": true}",
          "maxLength": 3000,
          "label": "Junction (JSON)",
          "help": "legs: up to two ({side: left/right or west/east/north/south, cross_section, length}); control: none, signal or stop; crosswalks: true/false; radius: kerb corner radius in metres. No legs = just carry the road across the gap."
        },
        "junction": {
          "type": "string",
          "default": "",
          "maxLength": 16,
          "label": "Junction id",
          "help": "Leave empty to use the selected connecting line (a plain line becomes a new junction)."
        }
      }
    },
    {
      "id": "set_crossing",
      "label": "Make Intersection",
      "params": {
        "description": {
          "type": "string",
          "default": "{\"treatment\": \"signal\", \"crosswalks\": true}",
          "maxLength": 1000,
          "label": "Intersection (JSON)",
          "help": "treatment: signal, two-way-stop, four-way-stop, uncontrolled, overpass, roundabout (cloverleaf later); crosswalks: true/false; radius: kerb corner radius in metres; minor: road that stops (two-way stop); over: road that goes over (overpass); diameter: roundabout size in metres. Select one line of each of the two roads that cross."
        }
      }
    },
    {
      "id": "set_hub",
      "label": "Make Intersection of Three or More Roads",
      "params": {
        "description": {
          "type": "string",
          "default": "{\"treatment\": \"signal\", \"crosswalks\": true}",
          "maxLength": 1000,
          "label": "Hub (JSON)",
          "help": "treatment: signal, two-way-stop, four-way-stop, uncontrolled; crosswalks; radius; minor; stub: keep or trim. Select one line of each road that meets at the point."
        }
      }
    },
    {
      "id": "set_transition",
      "label": "Add or Drop a Lane",
      "params": {
        "description": {
          "type": "string",
          "default": "{\"change\": \"add\", \"side\": \"right\"}",
          "maxLength": 1000,
          "label": "Lane change (JSON)",
          "help": "change: add or drop; side: left (toward the centre) or right (toward the edge); direction: fwd or back; count 1 to 3; at: metres from the start of the line; length: taper in metres (else the MUTCD formula); speed in mph; piece: which line of the road. Select one line of the road."
        }
      }
    },
    {
      "id": "set_water",
      "label": "Describe Water",
      "params": {
        "description": {
          "type": "string",
          "default": "{\"kind\": \"flowing\", \"width\": 8}",
          "maxLength": 1000,
          "label": "Water (JSON)",
          "help": "kind: still (a closed shape: lake, pond), flowing (an open line: creek, river) or shore (an open line with water on one side); width in metres and flow (fwd, back, none) and speed in m/s for flowing; side (left or right) for a shore; bank_left, bank_right (heights in metres) and slope (H:V). Select the line or shape."
        }
      }
    },
    {
      "id": "set_embankment",
      "label": "Put Road on Fill",
      "params": {
        "description": {
          "type": "string",
          "default": "{\"height\": 2, \"slope\": 2}",
          "maxLength": 1000,
          "label": "Embankment (JSON)",
          "help": "height of the road above the ground in metres; slope as horizontal per vertical (2 means 2:1); from and to in metres from the start of the line (left out: the whole line); sides both, left or right; ramp: metres over which the slope grows at each end; piece: which line of the road. Select one line of the road."
        }
      }
    },
    {
      "id": "set_wet",
      "label": "Road Meets Water",
      "params": {
        "description": {
          "type": "string",
          "default": "{\"treatment\": \"bridge\"}",
          "maxLength": 1000,
          "label": "Crossing (JSON)",
          "help": "treatment: bridge (deck_height above the water in m, abutment m beyond the banks), culvert (count, size in m, shape pipe or box, fill_height, slope) or causeway (fill_height, slope). Select one line of the road and one water line."
        }
      }
    }
  ]
}"#;

fn run(input: &[u8], params: &Value) -> Result<Output, String> {
    let input: Value = serde_json::from_slice(input).map_err(|e| format!("could not read the selection: {e}"))?;
    match params["_context"]["command"].as_str() {
        Some("set_settings") => builder::set_settings_text(&input, params),
        Some("set_cross_section") => builder::set_cross_section_text(&input, params),
        Some("set_junction") => builder::set_junction_text(&input, params),
        Some("set_crossing") => builder::set_crossing_text(&input, params),
        Some("set_hub") => builder::set_hub_text(&input, params),
        Some("set_transition") => builder::set_transition_text(&input, params),
        Some("set_water") => builder::set_water_text(&input, params),
        Some("set_embankment") => builder::set_embankment_text(&input, params),
        Some("set_wet") => builder::set_wet_text(&input, params),
        _ => builder::build_text(&input, params),
    }
    .map(Output::Raw)
}

vc_plugin!(MANIFEST, run);

#[cfg(test)]
mod tests {
    use super::MANIFEST;

    #[test]
    fn the_manifest_version_is_the_workspace_version() {
        let m: serde_json::Value = serde_json::from_str(MANIFEST).unwrap();
        assert_eq!(m["version"], cartographer_core::VERSION);
    }
}
