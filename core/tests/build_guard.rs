use cartographer_core::builder::{build_text, set_settings_text, set_crossing_text, set_cross_section_text, set_embankment_text, set_hub_text, set_junction_text, set_transition_text, set_water_text, set_wet_text};
use serde_json::{Value, json};

const TWO: &str = "lane back | lane fwd";

fn line(id: i64, name: &str, pts: &[[f64; 2]], closed: bool) -> Value {
    json!({"id": id, "name": name, "path": {"subpaths": [{"anchors": pts.iter().map(|p| json!({"p": p})).collect::<Vec<_>>(), "closed": closed}]}})
}

fn road() -> Value {
    json!({"objects": [line(1, "road.a", &[[200.0, 700.0], [200.0, 100.0]], false)]})
}

fn data(extra: Value) -> Value {
    let mut d = json!({"segments": {"a": {"cross_section": TWO}}});
    for (k, v) in extra.as_object().unwrap() {
        d[k] = v.clone();
    }
    d
}

fn params(document_data: Value) -> Value {
    json!({"description": "{}", "cross_section": TWO, "_context": {"documentData": document_data, "artboard": [0, 0, 600, 800]}})
}

fn parsed(text: Result<String, String>) -> Value {
    serde_json::from_str(&text.unwrap()).unwrap()
}

#[test]
fn every_command_refuses_a_newer_schema() {
    let p = params(json!({"schema": 9}));
    let commands: [(&str, fn(&Value, &Value) -> Result<String, String>); 9] = [
        ("build", build_text),
        ("set_cross_section", set_cross_section_text),
        ("set_junction", set_junction_text),
        ("set_crossing", set_crossing_text),
        ("set_hub", set_hub_text),
        ("set_transition", set_transition_text),
        ("set_water", set_water_text),
        ("set_embankment", set_embankment_text),
        ("set_wet", set_wet_text),
    ];
    for (name, f) in commands {
        let e = f(&road(), &p).unwrap_err();
        assert!(e.contains("newer Cartographer"), "{name}: {e}");
    }
}

#[test]
fn settings_and_unknown_data_survive_every_command() {
    let d = data(json!({"schema": 1, "settings": {"seed": 7, "sheet": {"size": "A0"}}, "extra": {"k": 1}}));
    for out in [parsed(build_text(&road(), &params(d.clone()))), parsed(set_cross_section_text(&road(), &params(d.clone())))] {
        let kept = &out["documentData"];
        assert_eq!(kept["settings"]["seed"], 7);
        assert_eq!(kept["settings"]["sheet"], json!({"size": "A0"}));
        assert_eq!(kept["extra"]["k"], 1);
        assert_eq!(kept["schema"], 1);
    }
}

#[test]
fn nothing_to_build_is_a_plain_message() {
    let input = json!({"objects": [line(1, "something", &[[0.0, 0.0], [5.0, 5.0]], false)]});
    let e = build_text(&input, &params(data(json!({})))).unwrap_err();
    assert!(e.contains("Nothing to build") && e.contains("road.<id>"), "{e}");
}

#[test]
fn a_one_point_line_and_a_closed_road_shape_make_no_empty_object() {
    let input = json!({"objects": [
        line(1, "road.a", &[[10.0, 10.0]], false),
        line(2, "road.b", &[[0.0, 0.0], [100.0, 0.0], [100.0, 100.0]], true),
    ]});
    let d = data(json!({"segments": {"a": {"cross_section": TWO}, "b": {"cross_section": TWO}}}));
    let out = parsed(build_text(&input, &params(d)));
    for o in out["objects"].as_array().unwrap() {
        if o["name"].as_str().is_some_and(|n| n.starts_with("gen.")) {
            let subs = o["path"]["subpaths"].as_array().unwrap();
            assert!(subs.iter().any(|s| s["anchors"].as_array().unwrap().len() >= 2), "empty object {}", o["name"]);
        }
    }
}

#[test]
fn a_crossing_hub_or_wet_naming_a_missing_road_is_survived() {
    let d = data(json!({
        "crossings": {"k1": {"roads": ["a", "zz"], "treatment": "signal"}},
        "hubs": {"h1": {"roads": ["a", "zz", "yy"], "treatment": "signal"}},
        "waters": {"w": {"kind": "flowing", "width": 8}},
        "wets": {"x1": {"road": "zz", "water": "w", "treatment": "bridge"}},
    }));
    assert!(build_text(&road(), &params(d)).is_ok());
}

fn long_road(points: usize) -> Value {
    let pts: Vec<[f64; 2]> = (0..points).map(|i| [100.0 + (i as f64 * 0.5).sin() * 20.0, 10.0 + i as f64 * 10.0]).collect();
    json!({"objects": [line(1, "road.a", &pts, false)]})
}

#[test]
fn a_big_build_gets_a_note_naming_the_point_count() {
    let notes = |n: usize| -> Vec<String> {
        let out = parsed(build_text(&long_road(n), &params(data(json!({})))));
        out["documentData"]["notes"].as_array().map(|a| a.iter().map(|v| v.as_str().unwrap().to_string()).collect()).unwrap_or_default()
    };
    assert!(!notes(1199).iter().any(|n| n.contains("points")), "no note below the limit");
    let big = notes(1300);
    assert!(big.iter().any(|n| n.contains("1300") && n.contains("split") && n.contains("2,000")), "{big:?}");
}

#[test]
fn a_build_and_the_commands_write_the_schema_even_when_the_document_had_none() {
    let d = data(json!({}));
    assert!(d.get("schema").is_none());
    for out in [parsed(build_text(&road(), &params(d.clone()))), parsed(set_cross_section_text(&road(), &params(d.clone())))] {
        assert_eq!(out["documentData"]["schema"], 1);
    }
}

#[test]
fn keys_nested_in_segments_survive_the_commands() {
    let d = json!({"schema": 1, "segments": {"a": {"cross_section": TWO, "future": 5}, "odd": {"material": "x"}}});
    let p = {
        let mut p = params(d);
        p["description"] = json!(r#"{"height": 2, "slope": 2}"#);
        p
    };
    let results = [parsed(build_text(&road(), &p)), parsed(set_cross_section_text(&road(), &p)), parsed(set_embankment_text(&road(), &p))];
    for out in results {
        assert_eq!(out["documentData"]["segments"]["a"]["future"], 5);
        assert_eq!(out["documentData"]["segments"]["odd"]["material"], "x");
        assert_eq!(out["documentData"]["segments"]["a"]["cross_section"], TWO);
    }
}

#[test]
fn a_bad_stored_setting_does_not_block_build_and_a_settings_command_repairs_it() {
    let d = data(json!({"schema": 1, "settings": {"style": "neon"}}));
    assert!(build_text(&road(), &params(d.clone())).is_ok(), "Build does not use settings");
    let mut p = params(d);
    p["description"] = json!(r#"{"style": "hand-drawn"}"#);
    let out = parsed(set_settings_text(&road(), &p));
    assert_eq!(out["documentData"]["settings"]["style"], "hand-drawn");
}
