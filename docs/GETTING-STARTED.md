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

- **macOS:** `open -a VectorCraft --args --control 7979`
- **Linux:** run the AppImage (or the installed program) with `--control 7979`. Set `VECTORCRAFT_APP` to its path if it is not on your `PATH`. Whether the Linux build accepts `--control` is not yet confirmed: the manual checklist in [the Linux spike](spikes/S2-linux.md) tests it.

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

Open http://127.0.0.1:8789 beside VectorCraft. The settings card shows the document's target print scale, style, random seed and north. Change a value to see a preview, then press **Draw this** to store it in the document (it is kept when you Save, Save As, or export to PDF; an SVG export keeps the shapes and names but not the plug-in data).

## Troubleshooting

- "VectorCraft is not answering on port 7979": start VectorCraft with its control port (step 3).
- "cartographer-cli not found": run `node scripts/build.mjs`.
- "this document was saved by a newer Cartographer": update the plug-in; the document is left untouched.
