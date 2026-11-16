use cartographer_core::materials::{Material, Role, Status, builtin, load, validate};

fn m(id: &str) -> Material {
    Material { id: id.into(), role: Role::Surface, default_width: 3.0, min_print_width: 0.25, class: "earth.sand".into(), height: None, source: None, status: Status::Unverified }
}

#[test]
fn the_seed_library_loads_and_validates() {
    let lib = builtin();
    for id in ["seawall", "pavement", "sand", "dune", "grass", "ballast", "track", "bank", "riprap", "mud", "marsh", "forest-floor"] {
        assert!(lib.iter().any(|x| x.id == id), "missing {id}");
    }
    assert_eq!(validate(&lib), Vec::<String>::new());
}

#[test]
fn duplicate_ids_are_reported() {
    let problems = validate(&[m("sand"), m("sand")]);
    assert!(problems.iter().any(|p| p.contains("duplicate") && p.contains("sand")), "{problems:?}");
}

#[test]
fn a_verified_entry_needs_a_source() {
    for status in [Status::Verified, Status::VerifiedSecondary] {
        let problems = validate(&[Material { status, ..m("sand") }]);
        assert!(problems.iter().any(|p| p.contains("source")), "{problems:?}");
    }
    let ok = Material { status: Status::Verified, source: Some("USGS topographic map symbols".into()), ..m("sand") };
    assert!(validate(&[ok]).is_empty());
}

#[test]
fn widths_must_be_positive() {
    let problems = validate(&[Material { default_width: 0.0, ..m("a") }, Material { min_print_width: -0.1, ..m("b") }]);
    assert!(problems.iter().any(|p| p.contains("default_width")), "{problems:?}");
    assert!(problems.iter().any(|p| p.contains("min_print_width")), "{problems:?}");
}

#[test]
fn class_is_dotted() {
    assert!(validate(&[Material { class: "sand".into(), ..m("sand") }]).iter().any(|p| p.contains("class")));
    assert!(validate(&[m("sand")]).is_empty());
}

#[test]
fn load_reads_json_and_names_a_bad_role() {
    let ok = r#"{"materials":[{"id":"sand","role":"surface","default_width":20,"min_print_width":0.25,"class":"earth.sand","status":"unverified"}]}"#;
    assert_eq!(load(ok).unwrap()[0].id, "sand");
    let bad = ok.replace("surface", "sparkle");
    assert!(load(&bad).unwrap_err().contains("role"));
    assert!(load("{nope").unwrap_err().contains("not valid JSON"));
}
