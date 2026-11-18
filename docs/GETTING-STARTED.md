# Getting started

## 1. Check this machine

```
node scripts/check.mjs
```

Each line starts with `ok:` or `fix:`. A `fix:` line says what to install. Lines starting with `note:` are optional: `vectorcraft-cli` (headless renders, macOS only) and a running VectorCraft.

## 2. Build

```
node scripts/build.mjs
```

This builds the plug-in (`target/wasm32-unknown-unknown/release/cartographer_filter.wasm`) and the native helper (`target/release/cartographer-cli`). On macOS with Homebrew the script puts rustup's tools first so the WebAssembly target is found.

## 3. Start VectorCraft with its control port

The installer and the panel talk to a running VectorCraft over a local control port (loopback only, no password: open it only while you use it).

```
node scripts/start-vectorcraft.mjs
```

The port is fixed only when VectorCraft starts: if it is already running, quit it first. The script runs `open -a VectorCraft --args --control 7979` on macOS. On Linux it runs the program named in `VECTORCRAFT_APP` (the AppImage path), or `vectorcraft` from your `PATH`, with `--control 7979`; it says what to set if it finds neither. Whether the Linux build accepts `--control` is not yet confirmed: the manual checklist in [the Linux spike](spikes/S2-linux.md) tests it.

To use another port, set `VECTORCRAFT_CONTROL_PORT` for the scripts and pass `--control <port>` to the panel.

## 4. Install the plug-in

```
node scripts/install.mjs
```

This installs the plug-in into the running VectorCraft. It does not touch your open document.

## 5. Open the panel

```
node panel/server.mjs
```

Open http://127.0.0.1:8789 beside VectorCraft. Select a line or a road to see its cards. The **Document settings** card at the bottom shows the document's target print scale, style, random seed and north (open it to load them). Change a value to see a preview, then press **Draw this** to store it in the document (it is kept when you Save, Save As, or export to PDF; an SVG export keeps the shapes and names but not the plug-in data).

## 6. Agents (describing things in words)

The panel's description box sends your words to an agent, which answers with a description that the native helper checks before anything is drawn. The agents are listed in `panel/providers.json`:

- **Claude** (needs the `claude` program installed and signed in) and **Claude (more careful)**.
- **Apple Intelligence (on this Mac)**: built by `node scripts/build.mjs` when the Xcode command line tools (`swiftc`) are installed; it runs on the Mac and nothing leaves it. It is hidden where it is not built, and always on Linux.
- Your own agents (a command, or an OpenAI-style endpoint): put a `providers.json` of the same shape in `~/.config/vectorcraft-cartographer/providers.json` (or name it in `CARTOGRAPHER_PROVIDERS`); its entries are added to the built-in ones.

An agent that is not installed or not signed in gives a plain message in the panel; nothing else breaks.

To check the panel against a real VectorCraft, follow [PANEL-CHECK.md](PANEL-CHECK.md).

## Troubleshooting

- "VectorCraft is not answering on port 7979": start VectorCraft with its control port (step 3).
- "cartographer-cli not found": run `node scripts/build.mjs`.
- "this document was saved by a newer Cartographer": update the plug-in; the document is left untouched.

## Railway track

Draw a line, select it, and in the panel choose **Railway track (N scale)**. Describe it in words ("double track", "a yard with 8 spurs on the left and 4 on the right, merging back at the end") or use the card (tracks, spacing, roadbed, ties; for a yard also main tracks, spurs, far end, turnout number, start and length). Select two track lines to say how they meet: a crossing, a branch (a siding off the other with a turnout) or a join end to end. Everything is millimetres of model at 1:160, drawn at 0.16 points per millimetre; see [RAIL.md](RAIL.md).
