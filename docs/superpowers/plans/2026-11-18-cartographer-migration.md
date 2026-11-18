# Migration of Road Builder and Track Layer documents Implementation Plan (Plan 6, 0.6.0)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (inline) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A tool that converts a saved VectorCraft document made with Road Builder and/or Track Layer into one Cartographer builds, with the same drawing, and a written report of what it did and could not do; plus the release-1.0 checklist and the wording that retires the two old repositories. No tag, no release, nothing pushed to the old repositories.

**Architecture:** A filter cannot read another plug-in's stored data, and a command parameter is too small for a whole record, so migration is an offline step on a **copy** of the saved `.vectorcraft` file (the spec: "run once, on a copy, and reports what it could not convert"): `scripts/migrate.mjs <in> <out>`, built on a pure function over the parsed file (`scripts/lib/migrate.mjs`). It copies each old plug-in's record to Cartographer's id with `schema` first, scales Track Layer's `track.*` lines from 72/25.4 points per millimetre to Cartographer's 0.16, drops the stale `gen.*` objects (the first Build redraws them), and keeps the old records as they were. The proof is a live comparison: the same document built by the old plug-in and by Cartographer after migration gives identical SVG (roads) and identical SVG when scaled (tracks).

**Tech Stack:** Node (no dependencies), the existing headless/live VectorCraft scripts.

**Spec:** `docs/superpowers/specs/2026-11-16-cartographer-design.md` sections 10 (phase 5, the parity gate) and 12 (migration); `docs/superpowers/plans/NEXT.md`.

## Global Constraints

- "It is run once, on a copy, and reports what it could not convert." The input file is never modified; the output path must not exist (unless `--force`) and must differ from the input.
- Object names carry over so layers and spines keep working (spec 12); `schema` comes first in the written record; unknown keys are kept, never dropped.
- Track Layer's documents are in model millimetres at 72/25.4 points per mm; Cartographer writes rail at 0.16 points per mm (`docs/PARITY-RAIL.md`): factor `0.16 / (72 / 25.4)`. Record fields in millimetres (`at`, `length`, `spacing`) are unchanged: Cartographer keeps model mm in records.
- Road Builder's road data is identical in name and meaning to Cartographer's (Plan 2 parity); migration must not change it.
- Nothing here tags, releases, merges, or pushes to the two old repositories.

## Review Focus

1. A document that has both Road Builder and Track Layer records, or only one, or neither: the right records move, notes are joined, nothing is lost, and "nothing to migrate" is said plainly (Task 1).
2. A document that already has a Cartographer record: it is refused, not overwritten (Task 1).
3. A saved file whose JSON is followed by other bytes, or that is gzip-compressed: the bytes after the JSON survive and the file opens again (Task 1).
4. Track lines with Bezier handles: handles scale with the points, closed shapes stay closed, `road.*` lines are not scaled (Task 1).
5. The input file is left byte-identical, and the output is never written over an existing file (Task 2).

---

### Task 1: The conversion

**Files:**
- Create: `scripts/lib/migrate.mjs`, `scripts/test/migrate.test.mjs`

**Interfaces:**
- Produces: `export const IDS = { roadbuilder, tracklayer, cartographer }` (the three plug-in ids); `export const TRACK_SCALE` (0.16 / (72 / 25.4)); `migrateDocument(file: object) -> { file: object, report: string[] }` (pure: the input object is not changed; throws `Error` with a plain message when the document already has a Cartographer record); `migrateBytes(bytes: Buffer) -> { bytes: Buffer, report: string[] }` (reads plain or gzip JSON, keeps every byte after the JSON part (a `\0VCBLOBS` tail), writes the same kind back).

- [ ] **Step 1: Write the failing tests** in `scripts/test/migrate.test.mjs`: `a_road_builder_record_moves_to_cartographers_id` (record `{version: 1, segments: {a: {cross_section: "lane back | lane fwd", anchor: "auto"}}, notes: ["x"]}` under the road builder id appears under the cartographer id with `schema: 1` as its first key and the same segments and notes; the old record is still there; the report says `1 road` in words); `a_track_layer_record_moves_and_its_lines_are_scaled` (record with `tracks` and `yards`; a `track.t1` path with anchors `[0,0]` and `[1000, 500]` with `in`/`out` handles becomes anchors scaled by `0.16 / (72 / 25.4)` including handles; a `road.a` path is not scaled; a path inside a nested layer is found; `closed` is kept); `both_records_are_joined_and_notes_are_concatenated`; `unknown_keys_are_kept_and_reported`; `generated_objects_are_dropped_and_counted` (`gen.road.a.1.line.0` and `gen.track.t1.rails` nodes removed wherever they are, the report counts them and says to build again; user lines stay); `a_document_with_nothing_to_migrate_says_so` (the file comes back equal, the report's first line says there is nothing to migrate); `a_document_that_already_has_cartographer_data_is_refused`; `a_newer_old_record_version_is_reported`; `the_input_object_is_not_changed`; `bytes_after_the_json_survive` (plain file: JSON, then `\0VCBLOBS` and 100 random bytes; the output ends with the same 100 bytes and the JSON part parses); `a_gzip_file_comes_back_gzip`.
- [ ] **Step 2: Run** `node --test scripts/test/migrate.test.mjs`. Expected: FAIL (module missing).
- [ ] **Step 3: Implement `scripts/lib/migrate.mjs`.** The records live at `file.document.unknown.pluginData[<id>]`. Known keys: `version schema settings notes segments junctions crossings hubs waters wets embankments transitions library tracks yards joints`; any other key is kept and listed. Joined record: Road Builder's keys, then Track Layer's, `notes` concatenated (road first), `schema: 1` first, `version: 1`. Walk `document.layers` (nodes have `kind.children` and `kind.path.subpaths[].anchors[]` with `p` and optional `in` and `out`) to scale `track.*` paths (the name is `track.<id>`) and to drop nodes whose name starts with `gen.`. A record whose `version` is above 1 is copied and reported as newer than this tool knows. Bytes: find the `\0VCBLOBS` marker in the latin1 text of the (un-gzipped) bytes; the JSON is everything up to the last `}` before it (or the last `}` in the file when there is no marker); write `JSON.stringify(file, null, 2)` followed by the original tail bytes; gzip again if the input was gzip.
- [ ] **Step 4: Run** the test file. Expected: PASS. **Commit** (`Migration: the conversion`).

### Task 2: The command

**Files:**
- Create: `scripts/migrate.mjs`
- Modify: `scripts/test/migrate.test.mjs` (command tests)

**Interfaces:**
- Consumes: `migrateBytes` (Task 1).
- Produces: `node scripts/migrate.mjs <in.vectorcraft> <out.vectorcraft> [--force]`: prints the report, exit 0 on success, 1 with a plain message otherwise (input missing; output equals input; output exists without `--force`; already migrated; nothing to migrate is exit 0 with no file written).

- [ ] **Step 1: Write the failing tests:** `the_command_writes_a_copy_and_leaves_the_input_alone` (temp dir; input bytes identical afterwards; output exists and parses); `it_refuses_to_overwrite_and_says_so`; `it_refuses_the_input_as_the_output`; `a_missing_input_is_a_plain_message`; `nothing_to_migrate_writes_nothing`; `force_replaces_an_existing_output`.
- [ ] **Step 2: Run** them. Expected: FAIL. **Step 3: Implement** the script around `migrateBytes` (with the node version guard `./lib/require-node.mjs`). **Step 4: Run.** Expected: PASS. **Commit** (`Migration: the command`).

### Task 3: Proof against the real plug-ins

**Files:**
- Create: `scripts/live-migrate-check.mjs`

- [ ] **Step 1: Write the check.** Against a running VectorCraft (`--control`), in new documents: install Road Builder's and Track Layer's wasm (`<road_builder_dir>/target/wasm32-unknown-unknown/release/road_builder_filter.wasm`, `<track_layer_dir>/.../track_layer_filter.wasm`, given as arguments) and Cartographer's; (a) road document: two crossing lines described with Road Builder's `set_cross_section` and a `set_crossing`, Build with Road Builder, export SVG, save a copy; migrate it with `migrateBytes`; open the migrated copy; Build with Cartographer; export SVG; assert the `gen.road` elements are identical (use `scripts/lib/svgcompare.mjs` `compareSvg`); (b) track document: a line described as a yard and a crossing track line (millimetre lines at 72/25.4 points per mm), Track Layer build, SVG, copy, migrate, open, Cartographer build, SVG; assert `compareSvgScaled` with factor `TRACK_SCALE` and tolerance 0.001 (use `skipCollapsedRings` as the rail gate does). Print what it checked.
- [ ] **Step 2: Run it** against the running VectorCraft. Expected: both comparisons identical. A difference is a migration bug: find the cause, never loosen the comparison. **Commit** (`Migration: live check against the old plug-ins`).

### Task 4: Documents, 0.6.0 and the release checklist

**Files:**
- Create: `docs/MIGRATION.md`, `docs/RELEASE-1.0.md`
- Modify: `README.md`, `CHANGELOG.md`, `CLAUDE.md`, `docs/GETTING-STARTED.md`, `docs/superpowers/plans/NEXT.md`, `Cargo.toml`, `filter/src/lib.rs`, version tests

- [ ] **Step 1:** version tests first (0.6.0, RED), then `Cargo.toml` and the manifest. **Step 2:** write `docs/MIGRATION.md` (what it does, how to run it on a copy, the report, what it does not do: the legacy centreline model, `.pmap` files and Add Junction are not carried over; why), `docs/RELEASE-1.0.md` (the parity gate as the spec states it with what is done and what is the owner's: the README image render set side by side, the owner's panel check on both systems, the agent steps; the exact commands to tag and release, **for the owner to run**; the wording for the two old repositories' READMEs, ready to paste, applied by nobody here), README, CHANGELOG `0.6.0 Migration`, CLAUDE.md (migration rules), GETTING-STARTED, and `NEXT.md` (what is left to 1.0 and the open items). **Step 3: Full gate**: `cargo +stable test --release --workspace`, `node --test panel/test/*.test.mjs scripts/test/*.test.mjs`, `node scripts/check.mjs`, `node scripts/standards.mjs check`, road, rail and fuel parity, the live migrate check. Push, draft PR against `styles-and-scale`. **Commit** (`Release 0.6.0 migration`).

### Task 5: Whole-branch review and fixes

- [ ] One fresh whole-branch review (Opus or better), one fix pass with RED tests, minors to the ledger, final report.

## Calls made in this plan (check these)

- **The legacy centreline model, `.pmap` import and export, and Add Junction are dropped.** Plan 2 left them for "the owner may drop them"; the Ross Bridge map was migrated onto cross-sections in Road Builder already, and the owner decided against exporting simulation data (pmap v2 is that hand-off). They stay in the Road Builder repository. If you want them, they are a separate port.
- **Migration is an offline command on a copy, not a plug-in command**: a filter cannot read another plug-in's data and a command parameter cannot hold a whole record.
- The old records are left in the file (harmless, and a way back); only `gen.*` objects are removed.
- Track Layer's records keep model millimetres (the Plan 4 ruling); only its line geometry is scaled.
- No tag, release or change to the old repositories: those are yours (`docs/RELEASE-1.0.md`).
