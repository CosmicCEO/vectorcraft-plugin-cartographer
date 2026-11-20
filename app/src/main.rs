//! Cartographer app (spike W1): starts the panel (`node panel/server.mjs`) on a free port, shows it in the window, stops it on quit.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{Manager, RunEvent};

struct Panel(Mutex<Option<Child>>);

/// The folder that holds `panel/` and `scripts/`: `CARTOGRAPHER_ROOT`, else the repository this app was built in (spike).
fn root() -> PathBuf {
    std::env::var_os("CARTOGRAPHER_ROOT").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."))
}

/// An app opened from Finder gets almost no PATH; the panel calls programs by name (node, the AI agents), so take the user's login PATH.
fn login_path() -> String {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
    Command::new(shell).args(["-lc", "printf %s \"$PATH\""]).output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok()).filter(|p| !p.is_empty())
        .unwrap_or_else(|| "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin".into())
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0").and_then(|l| l.local_addr()).map(|a| a.port()).unwrap_or(8789)
}

fn wait_for(port: u16, within: Duration) -> bool {
    let end = Instant::now() + within;
    while Instant::now() < end {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

fn main() {
    let app = tauri::Builder::default()
        .manage(Panel(Mutex::new(None)))
        .setup(|app| {
            let (port, path, dir) = (free_port(), login_path(), root());
            let log = dirs_log();
            let child = Command::new("node")
                .arg(dir.join("panel/server.mjs")).args(["--port", &port.to_string()])
                .env("PATH", &path).current_dir(&dir)
                .stdout(log.try_clone().map(Stdio::from).unwrap_or(Stdio::null())).stderr(Stdio::from(log))
                .spawn()?;
            *app.state::<Panel>().0.lock().unwrap() = Some(child);
            let win = app.get_webview_window("main").expect("main window");
            std::thread::spawn(move || {
                if wait_for(port, Duration::from_secs(20)) {
                    let _ = win.navigate(format!("http://127.0.0.1:{port}/").parse().unwrap());
                } else {
                    let _ = win.eval("document.getElementById('m').textContent = 'The panel did not start. See ~/Library/Logs/Cartographer/panel.log.'");
                }
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Cartographer could not start");
    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            if let Some(mut c) = handle.state::<Panel>().0.lock().unwrap().take() {
                let _ = c.kill();
            }
        }
    });
}

/// The panel's output goes to ~/Library/Logs/Cartographer/panel.log.
fn dirs_log() -> std::fs::File {
    let dir = PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Library/Logs/Cartographer");
    let _ = std::fs::create_dir_all(&dir);
    std::fs::File::create(dir.join("panel.log")).expect("log file")
}
