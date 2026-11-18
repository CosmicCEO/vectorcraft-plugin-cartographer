# Migrating Road Builder and Track Layer documents

`node scripts/migrate.mjs <in.vectorcraft> <out.vectorcraft>` turns a saved VectorCraft document made with Road Builder and/or Track Layer into one Cartographer builds, with the same drawing. It is run **once, on a copy**: the input is never changed and the output is never written over an existing file (add `--force` to replace one).

## What it does

- Copies each old plug-in's stored record (`com.cosmicceo.roadbuilder.build`, `com.cosmicceo.tracklayer.build`) to Cartographer's id (`com.cosmicceo.cartographer.build`), `schema` first. A document with both gets one joined record; the notes are put one after the other. Unknown keys are kept and listed in the report.
- Scales Track Layer's `track.*` lines from its 72/25.4 document points per millimetre to Cartographer's 0.16 (factor 0.0564444), handles included. Road lines and record fields that are in millimetres (`spacing`, `at`, `length`) are not changed.
- Removes the `gen.*` objects the old plug-ins drew (they are in the old scale and are drawn again by the first Build).
- Leaves the old records in the file (harmless, and a way back).
- Prints a plain-words report: what moved, how many lines were scaled, how many generated objects were removed, what was kept but is not used.

A document that has nothing to migrate, that already has Cartographer data, or that uses the older centreline model, is reported and no file is written. The output may not be the input under any name (a hard link or another capitalisation of the same file is found by its identity), even with `--force`.

## Steps

1. Copy the document file (or use Save a Copy in VectorCraft).
2. `node scripts/migrate.mjs old.vectorcraft migrated.vectorcraft`
3. Install Cartographer (`node scripts/install.mjs`), open `migrated.vectorcraft`, select all, press **Build** (or **Draw** in the panel).
4. Look at it. Layers named `road r1`, `track t1 ...` carry over because object names do.

## Proof

`node scripts/live-migrate-check.mjs <road_builder_filter.wasm> <track_layer_filter.wasm>` builds a road document with Road Builder and a track document with Track Layer in a live VectorCraft, saves a copy, migrates it, builds it with Cartographer and compares the SVG: identical for the roads, identical when scaled (within 0.001 point) for the tracks. It passed on VectorCraft 0.24.1 on macOS (2026-11-18). The 15 road and 8 rail parity scenarios (`docs/PARITY.md`, `docs/PARITY-RAIL.md`) say the same about the building itself.

## What it does not do

- **The older centreline model** (`centerline`, `profile.*`, `flag.*`, junction markers like `C1.signal.right.Main-St`), **`.pmap` files** and **Add Junction** are not in Cartographer. A document that has a `centerline` line is **refused** (nothing is written): its only drawing is made by the older model, which Cartographer cannot redraw, so migrating would delete it. Keep using Road Builder for such documents; they stay in the Road Builder repository. The Ross Bridge map was moved onto cross-sections in Road Builder already, and the owner decided against exporting simulation data (which `.pmap` v2 was for).
- It does not convert a document back.
- It needs the saved file: it cannot read an open document.
