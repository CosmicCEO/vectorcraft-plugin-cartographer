//! `cartographer-cli`: checks what an agent proposes and prints schemas for the panel. One line of JSON on stdout; exit 1 on a problem.

use cartographer_core::builder::settings_summary;
use cartographer_core::schema::settings_schema;
use cartographer_core::storage::Settings;
use serde_json::{Value, json};
use std::process::ExitCode;

fn fail(message: impl std::fmt::Display) -> Value {
    json!({"ok": false, "error": message.to_string()})
}

fn run(args: &[String]) -> Value {
    match args.first().map(String::as_str) {
        Some("version") => json!({"ok": true, "version": cartographer_core::VERSION}),
        Some("parse-settings") => {
            let Some(text) = args.get(1) else { return fail("usage: parse-settings '<settings JSON>'") };
            let value: Value = match serde_json::from_str(text) {
                Ok(v) => v,
                Err(e) => return fail(format!("not valid JSON: {e}")),
            };
            match Settings::from_json(&value) {
                Ok(s) => json!({"ok": true, "settings": s.to_json(), "summary": settings_summary(&s)}),
                Err(e) => fail(e),
            }
        }
        Some("schema") => match args.get(1).map(String::as_str) {
            Some("settings") => {
                let mut v = settings_schema();
                v["ok"] = json!(true);
                v
            }
            _ => fail("usage: schema settings"),
        },
        _ => fail("usage: cartographer-cli version | parse-settings '<json>' | schema settings"),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = run(&args);
    println!("{result}");
    if result["ok"] == json!(true) { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
