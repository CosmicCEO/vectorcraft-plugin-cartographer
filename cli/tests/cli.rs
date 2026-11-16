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
