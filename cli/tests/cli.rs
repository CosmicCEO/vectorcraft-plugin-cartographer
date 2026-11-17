use std::process::Command;

fn run(args: &[&str]) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_cartographer-cli")).args(args).output().unwrap();
    (String::from_utf8(out.stdout).unwrap(), out.status.code().unwrap())
}

#[test]
fn parse_prints_the_total_width() {
    let (out, code) = run(&["parse", "lane back | lane fwd"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("\"ok\":true") && out.contains("\"total_width\":7.2"), "{out}");
}

#[test]
fn parse_crossing_accepts_signal() {
    let (out, code) = run(&["parse-crossing", r#"{"roads":["a","b"],"treatment":"signal"}"#]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("\"ok\":true"), "{out}");
}

#[test]
fn a_bad_command_is_a_plain_error() {
    let (out, code) = run(&["no-such-command"]);
    assert_eq!(code, 1);
    assert!(out.contains("\"ok\":false"), "{out}");
}

#[test]
fn version_is_the_workspace_version() {
    let (out, code) = run(&["version"]);
    assert_eq!(code, 0);
    assert!(out.contains("\"version\":\"0.4.0\""), "{out}");
}

#[test]
fn parse_tracks_prints_the_summary() {
    let (out, code) = run(&["parse-tracks", r#"{"tracks": 2}"#]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("\"ok\":true") && out.contains("\"summary\""), "{out}");
}

#[test]
fn parse_yard_rejects_a_bad_field() {
    let (out, code) = run(&["parse-yard", r#"{"mains": 99}"#]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("\"ok\":false"), "{out}");
}

#[test]
fn find_joint_runs_and_says_when_lines_do_not_meet() {
    let a = r#"{"subpaths":[{"anchors":[{"p":[0,0]},{"p":[200,0]}]}]}"#;
    let crossing = r#"{"subpaths":[{"anchors":[{"p":[100,-50]},{"p":[100,50]}]}]}"#;
    let (out, code) = run(&["find-joint", a, crossing, "a", "b"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("\"kind\""), "{out}");
    let far = r#"{"subpaths":[{"anchors":[{"p":[0,500]},{"p":[200,500]}]}]}"#;
    let (out, code) = run(&["find-joint", a, far, "a", "b"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("do not meet"), "{out}");
}

#[test]
fn schema_prints_the_rail_cards() {
    for (kind, first) in [("tracks", "tracks"), ("yard", "mains"), ("joint", "kind")] {
        let (out, code) = run(&["schema", kind]);
        assert_eq!(code, 0, "{out}");
        assert!(out.contains("\"ok\":true") && out.contains(&format!("\"key\":\"{first}\"")), "{out}");
    }
    let (out, code) = run(&["schema", "nothing"]);
    assert_eq!(code, 1, "{out}");
}
