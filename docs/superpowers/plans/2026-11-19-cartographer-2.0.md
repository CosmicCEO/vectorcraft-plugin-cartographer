# Cartographer 2.0 Plan: the Mac app (agreed with the owner, 2026-11-19)

Goal (owner, 2026-11-19): no terminal. Double-click Cartographer and work. Mac only in 2.0 (Linux is 2.1), not signed (no Apple Developer ID). With it, the API change best made once. Agreed order: `docs/superpowers/plans/NEXT.md`.

## What the app is

- A Tauri 2 app (Rust, the toolkit the plug-in already uses): one window, the panel inside it, a small menu.
- Inside the app bundle: a copy of Node (the official macOS arm64 binary), the panel (`panel/` as it is), the helper `cartographer-cli`, the plug-in `cartographer_filter.wasm` and its library files. The app starts `node panel/server.mjs --port <free port>` and shows that page in its window; nobody types an address.
- Ruling: ship Node as a plain binary beside the panel rather than as a single executable application (SEA). SEA wants one CommonJS script, so the ES-module panel would need a bundler (a new dependency); a plain Node binary runs the panel exactly as today. Cost: about 110 MB, the same size SEA would be.
- The terminal scripts stay for development and for anyone who prefers them; nothing about the plug-in, its records or its drawings changes.

## Work packages

| Package | What | Done when |
|---|---|---|
| W1 spike | A bare Tauri window that starts Node with the panel from inside the bundle, waits for it, shows it, and stops it when the window closes. Checks: the panel's live updates work in the app's window (WebKit, the same engine as Safari); an app opened from Finder gets almost no `PATH`, so the AI agents the panel calls (claude, codex, ...) must be found through the user's login shell. | The spike runs from Finder on this Mac; findings in `docs/spikes/S7-app.md` |
| W2 app shell | `app/` (new Tauri crate, outside the plug-in workspace so plug-in builds stay as they are). Free-port choice; panel restarted if it stops; quitting the app stops it; one window, remembers its size. | Tests for the port choice and the restart; the app opens the panel on this Mac |
| W3 VectorCraft and setup | On launch: is VectorCraft running with its control port? If not, start it with the port (today's `start-vectorcraft.mjs` logic, moved into a shared module the app's panel calls); if it runs without the port, say so plainly with a Quit-and-retry button. First run, and after an update: install the plug-in into VectorCraft and run the setup check; results shown on the page in plain words. Menu: Check setup, Reinstall plug-in, Open the guide, About (name and release). | Tests for each state (not running, running without port, ready, install failed); live check on this Mac |
| W4 package and release | The release workflow builds the app on macOS and publishes an unsigned `Cartographer-2.0.0.dmg` beside today's packages. The guide shows the one-time right-click, Open (and what macOS says when it blocks an unsigned app). | Dry run of the workflow; the downloaded `.dmg` opens, installs, starts VectorCraft and draws a road on this Mac |
| W5 one command | One `set_feature` command `{kind, ...}` replaces the per-kind describe commands for the panel and agents. Owner: a clean build, no backward compatibility, so the old commands are removed (scripts, parity scenarios and agent prompts move to `set_feature`); records do not change, so 1.x documents still open and build. | Tests: every kind through `set_feature` gives the same record the old command gave; parity and fuel unchanged |
| W6 cards from schemas | The panel builds every card from the plug-in's own field lists (as tracks, yards and joints already are), so a field added to the plug-in shows on its card without panel code. | The panel's card tests pass on schema-built cards; a live panel check |
| W7 release 2.0.0 | Version, changelog, release notes (rulings and deferred), guide, `NEXT.md`, tag, verify the published `.dmg`. | As every release |

Order: W1 first (it decides the rest), then W2 to W4 (the app), then W5 and W6 (the API), then W7. Every package as before: tests first, parity and fuel, CI, a review by a separate model with one fix pass, live checks.

## Risks

- macOS may block or warn about an unsigned app more strongly in later versions; the guide covers the one-time right-click, Open. If it becomes a wall, signing (99 USD a year) is the way out.
- The panel calls AI agent programs by name; from Finder they are not on `PATH` (W1 checks and fixes this).
- The bundled Node must be updated with security releases (the release workflow downloads a pinned version and checks its checksum).
- The app is about 120 MB.

## Owner decisions

1. The app is called Cartographer. Its icon for now is cut from a dense drawing sample (a yard beside a road crossing, rendered by the plug-in itself); made in W2.
2. Clean build: the 1.x describe commands are removed in 2.0, no aliases.
