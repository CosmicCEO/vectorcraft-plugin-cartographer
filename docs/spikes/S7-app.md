# Spike S7: the Cartographer app (Tauri 2, macOS)

2026-11-19, on the owner's Mac (macOS 27, Apple silicon), tauri-cli 2.12.6, Node 26.13.1. Code: `app/` (not part of the plug-in workspace).

## What was tried

`app/src/main.rs` picks a free port, starts `node panel/server.mjs --port <port>` with the user's login `PATH`, waits for the port, and points the window at `http://127.0.0.1:<port>/`. Quitting the app kills the panel. Bundled with `cargo tauri build --bundles app`, opened with `open Cartographer.app` (as Finder does).

## Findings

- The window shows the panel: WebKit's network process holds a live connection to the panel server (the page's updates), the page answers 200. No browser address to type.
- An app opened from Finder has almost no `PATH`. Asking the user's login shell (`$SHELL -lc 'printf %s "$PATH"'`) gives the full one: the panel finds `node` and the AI agents (`claude` in `~/.local/bin` here).
- Quit (menu or Cmd-Q) stops the panel. A crash or a force quit would leave it running: W2 makes the panel exit when the app that started it is gone.
- The bare app is 10 MB; with Node inside it will be about 120 MB (W4).
- Unsigned and built on this Mac it opens without a warning; a downloaded copy carries the quarantine flag and will need the one-time right-click, Open (W4 checks this with the released `.dmg`).
- The icon (`app/app-icon.png`, made into `app/icons/` with `cargo tauri icon`) is a road bridge over a five-track yard cut from a Cartographer drawing (owner: a dense drawing sample for now).

## Decisions for W2

- Node and the panel go into the bundle's Resources; the app finds them there (in development, the repository).
- The panel gets the app's process id and exits when it is gone.
- The port stays free-chosen; nothing else needs it.
