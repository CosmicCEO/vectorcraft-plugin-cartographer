# Spike S3: what survives Save, Save As and export

Run on 2026-11-16 in a headless VectorCraft 0.24.1 session on macOS (`vectorcraft-cli mcp`), using Road Builder 0.7.0 only to create plug-in data and named shapes. Command: `node scripts/spikes/s3-survival.mjs <vectorcraft-cli> <road_builder_filter.wasm>`.

| Artefact | Plug-in data kept | Our spine names | Generated shapes | Spine direction |
|---|---|---|---|---|
| Save copy (.vectorcraft) | yes | yes | yes | kept |
| Export SVG | no | yes | yes | kept |
| Export PDF | yes | yes | yes | not checked |
| Reopen then Save copy | yes | yes | yes | kept |

Also: the saved copy reopens with the same six paths, and the plug-in data comes back. Exporting `.pmap` with Road Builder's exporter fails on a document without a `centerline` (that exporter belongs to the older centreline model, not to this test).

## What this means

- Save, Save As and reopen keep everything: plug-in data, names and direction. That covers the owner's main workflow.
- **PDF keeps the plug-in data** (found inside the PDF's compressed streams). PDF is therefore a second place the full record survives.
- **SVG keeps names, generated shapes and the spine's direction, but drops the plug-in data.** The design rule stands and is now confirmed as necessary: a name must carry the kind and id (`road.a`, `gen.road.a.1.surface.0.lane`), so an SVG can be traced back to its features, and the SVG is a one-way export: the parameters (cross-sections, settings, materials) are not in it.
- Not checked: spine direction inside the PDF, and the PNG (no vector data by definition).

## Decision

Keep names as `<kind>.<id>` and `gen.<kind>.<id>.<piece>.<role>` (spec section 6.1). Treat the native file and PDF as carriers of the full record, and SVG as names plus geometry only. No spec change beyond the one sentence in section 6.1 that records this.
