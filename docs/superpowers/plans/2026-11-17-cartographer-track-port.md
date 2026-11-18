# Cartographer Track Port Implementation Plan (Plan 4)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port Track Layer (N-scale model railroad track: parallel tracks, turnouts, crossings, yards, branches and joins) into Cartographer as the `track` kind, next to roads, with its Rust and panel tests passing unchanged, and a combined build for documents that hold both.

**Architecture:** Port first, generalise later (as in Plans 2 and 3). The rail modules move over verbatim; the shared `Out` gets per-kind output settings (decimals and curve-thinning tolerance) because rail dimensions are model millimetres at 0.16 points each under Cartographer's 1 point = 1 metre rule; one `build_text` builds roads and tracks together; the panel gets a `track` branch next to the road branch, with cards for tracks, yards and joints built from schemas (the first schema-built kind cards beyond settings). A small kinds registry (data) tells the panel and layers which line prefixes belong to which kind.

**Tech Stack:** Rust (edition 2024, `serde_json` only), Node 22 (zero dependencies), headless VectorCraft (macOS) for parity renders.

**Spec:** `docs/superpowers/specs/2026-11-16-cartographer-design.md`. **Prep and decisions:** `docs/superpowers/plans/PLAN-4-PREP.md` and `docs/spikes/S5-rail-port.md` (owner agreed to the recommendations on 2026-11-17). **Source:** `/Volumes/Mingus/Developer/vectorcraft-plugin-tracklayer` at `5859e9b` (0.9.1).

## Global Constraints

- Plans 1 to 3's constraints hold (Node 22, zero dependencies, never push to `main`, never merge or tag, commit trailers `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>` and `Claude-Session: https://claude.ai/code/session_01HBq3ghaocqFgSq9fTWECkM`, plain prose in documents, panel rules from Plan 3: textContent only, CLI JSON read whatever the exit code, a changed title drops proposals, the settings card kept apart).
- **Parity:** every ported test keeps its name and expected numbers; it may change only import names and the `standards` module name (`rail_standards`) and the crate name. Tolerances are never loosened: precision is solved in `Out`.
- The kind is `track`: lines `track.<id>`, built shapes `gen.track.<id>.*`, layers `track <id>`; document keys `tracks`, `yards`, `joints`, `lines`. Command ids `set_tracks`, `set_yard`, `set_joint` (no `set_feature` yet). Version `0.4.0`.
- Units: rail modules work in model millimetres at 1:160; `PT_PER_MM = 0.16` (1 mm of model is 0.16 m of ground); rail output is written with 5 decimals and curve thinning of 0.00677 points; roads keep 3 decimals and 0.12. Road output and cost must not change (the road parity gate and fuel numbers are re-run).
- Standards: `library/standards.json` (road, unchanged) and `library/standards-rail.json` (NMRA) are two files under one tool. Mark `verified` only if read in the document; nothing new is read in this plan.
- A road and a track crossing (a level crossing between a road line and a track line) is not in this plan.

## Review Focus

1. Each rail command, and a build that has tracks, refuse a document from a newer schema; `schema`, `settings`, road data and unknown keys survive them (Task 4).
2. Roads and tracks in one document build together: both kinds' objects and both kinds' notes appear, and rail objects sit above road objects (Task 4).
3. Output precision does not leak: after a track is written, road numbers still have 3 decimals; track numbers have 5 (Tasks 1 and 4).
4. A `track.x` line with one point, a closed track shape, or a yard line too short gives a plain message and no empty object (Task 4).
5. Selecting a road line together with a track line shows a plain "not supported yet" note, never an intersection card or an error; two track lines make a joint (Task 6).

---

## Tasks

### Task 1: Output settings per kind

**Files:**
- Modify: `core/src/output.rs`
- Test: `core/tests/output_settings.rs`

**Interfaces:**
- Produces: `Out::precision(&mut self, decimals: u8)` (clamped to 3 to 6; objects written afterwards use it; default 3); `Out::thinning(&mut self, tolerance: f64)` (curve thinning tolerance in document points, default 0.12); `output::num_with(out: &mut String, x: f64, decimals: u8)`; `output::num(out, x)` stays and equals `num_with(.., 3)`.

- [ ] **Step 1: Write the failing tests:** `the_default_is_three_decimals` (an object written with a point at `12.3456789` contains `12.346`); `precision_five_writes_five_decimals` (after `out.precision(5)` the same point is `12.34568`); `precision_can_change_between_objects` (an object at 3 decimals, then 5, then 3 again: each as written); `numbers_beyond_four_million_are_written_in_full` (a point at `5000000.123` contains `5000000.123`; today it is clamped); `a_bad_decimals_value_is_clamped` (`precision(0)` behaves as 3, `precision(9)` as 6); `thinning_is_a_setting` (a curve through 30 points all within 0.01 of a straight chord has 2 anchors by default, and more than 2 after `out.thinning(0.001)`); `negative_and_zero_still_write` (`-0.5`, `0`, `-0.00001` at 5 decimals).
- [ ] **Step 2: Run** `cargo +stable test --release --test output_settings`. Expected: FAIL (methods missing).
- [ ] **Step 3: Implement** `precision`, `thinning` and `num_with` in `output.rs` (the spike's 5-decimal version of `num`, parametrised, using `u64` so there is no 4.29 million cap); `curve` and `curve_fills` read the thinning field instead of the `THIN` constant.
- [ ] **Step 4: Run the whole workspace.** Expected: PASS, with the road tests unchanged. **Run** `node scripts/parity.mjs <cli> <road_builder_dir>` (expected all 15 identical) and `node scripts/fuel-parity.mjs <road_builder_filter.wasm>` (expected ratios within 1% of 0.973, 0.851, 0.544).
- [ ] **Step 5: Commit** (`Out: precision and thinning as settings`).

### Task 2: Rail standards and the one standards tool

**Files:**
- Create: `library/standards-rail.json`, `core/src/rail_standards.rs`, `core/src/turnout_table.rs` (generated), `scripts/lib/standards-rail.mjs`, `docs/STANDARDS-RAIL.md` (generated)
- Modify: `scripts/standards.mjs`, `core/src/lib.rs`, `.github/workflows/ci.yml`
- Test: `core/tests/rail_standards.rs`, `scripts/test/standards-rail.test.mjs`

**Interfaces:**
- Produces: `rail_standards::{SCALE, GAUGE, RAIL_HEAD, TIE_LENGTH, TIE_WIDTH, TIE_PITCH, FOOT, TRACK_CENTERS, MIN_RADIUS, PT_PER_MM, ...}` as in Track Layer's `standards.rs` with `PT_PER_MM = 0.16`; `standards-rail.mjs` exports `problems(db)`, `markdown(db)`, `rust(db)` (Track Layer's functions); `node scripts/standards.mjs check|md|rs` covers both databases (`md` writes both documents, `rs` writes `core/src/turnout_table.rs`).

- [ ] **Step 1: Copy `rail_standards.rs` test** (Track Layer's `tests/standards.rs`, 3 tests, reading `library/standards-rail.json`, module `rail_standards`) and write `standards-rail.test.mjs`: `a_duplicate_id_is_reported`, `a_status_other_than_verified_or_unverified_is_reported`, `an_unverified_entry_without_a_note_is_reported`, `a_table_with_frogs_out_of_order_is_reported`, `the_shipped_rail_database_is_valid` (`problems(db)` empty).
- [ ] **Step 2: Run** `cargo +stable test --release --test rail_standards` and `node --test scripts/test/standards-rail.test.mjs`. Expected: FAIL.
- [ ] **Step 3: Copy** Track Layer's `library/standards.json` to `library/standards-rail.json`, its `core/src/standards.rs` to `rail_standards.rs` (change `PT_PER_MM`), and port its checker, document writer and Rust writer into `scripts/lib/standards-rail.mjs`; extend `standards.mjs` so `check` fails if either database has a problem, `md` writes `docs/STANDARDS.md` and `docs/STANDARDS-RAIL.md`, and `rs` writes `turnout_table.rs`; add `node scripts/standards.mjs rs` and `git diff --exit-code docs/STANDARDS-RAIL.md core/src/turnout_table.rs` to CI.
- [ ] **Step 4: Run** both tests and `node scripts/standards.mjs check` (expected: road 59 entries ok, rail 10 entries and 1 table ok) and `node scripts/standards.mjs rs` (expected: no diff in `turnout_table.rs`).
- [ ] **Step 5: Commit** (`Rail standards and one standards tool`).

### Task 3: The rail modules

**Files:**
- Create: `core/src/{tracks,turnout,yard,ties,joint,connector,branch,rail_builder}.rs` (from Track Layer `core/src/{tracks,turnout,yard,ties,joint,connector,branch,builder}.rs`)
- Modify: `core/src/lib.rs`
- Test: `core/tests/rail_{tracks,turnouts,yards,joints,branches,connector,crossovers,curves}.rs`

**Interfaces:**
- Consumes: Tasks 1 and 2.
- Produces (names as in Track Layer, `builder` renamed `rail_builder`): `rail_builder::{track_id, build, build_text, set_tracks_text, set_yard_text, set_joint_text}`; plus `rail_builder::build_into(items: &[Item], data: &Value, out: &mut Out) -> Vec<String>` (new: builds every `track.*` line into an existing `Out` and returns its notes; `build_text` is rewritten on top of it) and `rail_builder::spine_style() -> ([f64; 3], f64)` (the track line's highlight colour and width). `build_into` calls `out.precision(5)` and `out.thinning(0.00677)` before writing and restores `precision(3)` and `thinning(0.12)` before returning.

- [ ] **Step 1: Copy the eight test files** (`track_layer_core` to `cartographer_core`, `builder` to `rail_builder as builder`, `standards` to `rail_standards`). Expected counts: tracks 9, turnouts 9, yards 7, joints 14, branches 14, connector 6, crossovers 10, curves 3 (72). Add `rail_settings.rs`: `build_into_writes_five_decimals_and_restores_three` (build one track; the track objects have a coordinate with more than 3 decimals somewhere; then write a road object with the same `Out` and check it has at most 3).
- [ ] **Step 2: Run** `cargo +stable test --release --tests`. Expected: FAIL to compile (modules missing).
- [ ] **Step 3: Copy the modules**, apply the `rail_standards` rename, set `PT_PER_MM`, add `build_into` and `spine_style`, register the modules in `lib.rs`.
- [ ] **Step 4: Run each test file in turn.** Expected: PASS for all 72 plus the new one, with no warnings.
- [ ] **Step 5: Commit** (`Port the rail modules`).

### Task 4: Combined build, commands, manifest and CLI

**Files:**
- Modify: `core/src/builder.rs`, `filter/src/lib.rs`, `cli/src/main.rs`
- Create: `core/tests/rail_guard.rs`
- Test: `cli/tests/cli.rs` (additions)

**Interfaces:**
- Produces: `builder::build_text` builds roads and tracks in one output (roads first, then tracks; the spine of a `track.*` line styled with `rail_builder::spine_style()`; notes of both kinds merged into the stored `notes`); `builder::{set_tracks_text, set_yard_text, set_joint_text}` run `Doc::check_schema` first and then Track Layer's command, keeping `schema`, `settings`, road data and unknown keys; filter commands `set_tracks`, `set_yard`, `set_joint` (Track Layer's manifest entries), manifest version `0.4.0`; CLI commands `parse-tracks`, `parse-yard`, `parse-joint`, `find-joint` with Track Layer's arguments and output.

- [ ] **Step 1: Write the failing tests** in `rail_guard.rs`: `every_rail_command_and_a_track_build_refuse_a_newer_schema` (`{"schema": 9}` into `set_tracks_text`, `set_yard_text`, `set_joint_text` and `build_text` with a track line: each `Err` contains `newer Cartographer`); `settings_road_data_and_unknown_keys_survive_a_rail_command` (data with `schema`, `settings.seed`, `segments`, `extra`; after `set_tracks_text` all are present and `tracks` holds the new description); `roads_and_tracks_build_together` (a document with `road.a` and `track.b`: the output has objects named `gen.road.a.*` and `gen.track.b.*`, every track object after every road object, and the stored `notes` contain a note from a road problem and a note from a track problem); `nothing_to_build_names_both_kinds` (a plain line: the message contains `road.<id>` and `track.<id>`); `a_one_point_track_and_a_closed_track_make_no_empty_object`; `a_yard_line_that_is_too_short_is_refused_in_plain_words` (as Track Layer's test, through the combined build); `a_road_after_a_track_keeps_three_decimals` (a road and a track in one build; a road object's coordinates have at most 3 decimals, a track object's has more). Add to `cli.rs`: `parse_tracks_prints_the_summary`, `parse_yard_rejects_a_bad_field`, `find_joint_runs`.
- [ ] **Step 2: Run** `cargo +stable test --release --test rail_guard` and `-p cartographer-cli`. Expected: FAIL.
- [ ] **Step 3: Implement** the combined `build_text` and the three guarded commands in `builder.rs` (the `has_road` test also counts `track.*` lines; the near-limit note counts every input point), extend the manifest (copy Track Layer's three command entries; version `0.4.0` and `Cargo.toml` version `0.4.0`, with the version tests updated first) and `run` dispatch, port the four CLI commands.
- [ ] **Step 4: Run** `cargo +stable test --release --workspace`. Expected: PASS (road and rail suites).
- [ ] **Step 5: Commit** (`Combined build, rail commands, manifest and CLI`).

### Task 5: Rail parity

**Files:**
- Create: `scripts/parity/rail-scenarios.json`, `scripts/render.mjs` (ported), `docs/PARITY-RAIL.md`
- Modify: `scripts/parity.mjs`, `scripts/lib/svgcompare.mjs`, `scripts/fuel-parity.mjs`, `scripts/spikes/fuel-inputs.mjs`
- Test: `scripts/test/svgcompare.test.mjs` (additions)

**Interfaces:**
- Produces: `compareSvgScaled(oldText: string, newText: string, { factor: number, tolerance: number }): { same: boolean, summary: string }` (every number in the old SVG's path data is multiplied by `factor`, then compared with the new one's within `tolerance`; everything else must be equal as in `compareSvg`); `parity.mjs <cli> <old_dir> --scenarios <file> --old-id <plugin id> --old-wasm <file> --factor <n>` (road defaults unchanged); scenarios are written in model millimetres and each plug-in's points are scaled (old: 72/25.4 per mm, new: 0.16 per mm).

- [ ] **Step 1: Write the failing tests** in `svgcompare.test.mjs`: `a_drawing_at_another_scale_is_the_same_drawing` (an SVG with `points="0,0 10,20"` and one with `points="0,0 2,4"` match with `factor: 0.2`), `a_difference_beyond_the_tolerance_is_reported` (4.01 against 4.0 with tolerance 0.001 differs, and the summary names the element), `names_and_structure_still_have_to_match` (a renamed `id` or an extra element differs even with a matching scale).
- [ ] **Step 2: Run** `node --test scripts/test/svgcompare.test.mjs`. Expected: FAIL.
- [ ] **Step 3: Implement `compareSvgScaled`;** extend `parity.mjs` and `fuel-parity.mjs` as described; write `rail-scenarios.json` from Track Layer's four examples (`oval`, `station`, `terminal`, `turnouts`) plus `two-tracks`, `crossing`, `branch`, `join` (parameters from the matching Track Layer tests); port `render.mjs` (a scenario in millimetres built in a headless session to a cropped PNG, with Cartographer's plug-in id and scale).
- [ ] **Step 4: Build Track Layer's plug-in** (`cargo +stable build --release --target wasm32-unknown-unknown -p track-layer-filter` in its repository) and run `node scripts/parity.mjs <cli> <track_layer_dir> --scenarios scripts/parity/rail-scenarios.json --old-id com.cosmicceo.tracklayer.build --old-wasm <track_layer_filter.wasm> --factor 0.0564444` (that is 0.16 / (72 / 25.4)). Expected: every scenario identical within 0.001. A difference is a port bug; find the cause, never loosen the comparison. Run `node scripts/fuel-parity.mjs` for the rail inputs (`yard1`, `tracks2`); expected ratio at most 1.05; if over, reduce rail decimals to 4 and rerun the whole task's checks.
- [ ] **Step 5: Write `docs/PARITY-RAIL.md`** (scenarios, the scaled comparison, fuel table, how to rerun) and render the four examples to `target/parity/` for the owner. **Commit** (`Rail parity gate`).

### Task 6: The panel's track branch and the schema-built cards

**Files:**
- Create: `panel/lib/kinds.mjs`, `panel/test/track.test.mjs`, `panel/test/kinds.test.mjs`
- Modify: `core/src/schema.rs`, `cli/src/main.rs`, `panel/lib/{doc,cli,layers,panel,providers,prompt,changes}.mjs`, `panel/public/{index.html,app.js,style.css}`, `panel/server.mjs`, `panel/test/{fake-vectorcraft,page}.test.mjs`
- Test: `core/tests/schemas.rs`

**Interfaces:**
- Produces: CLI `schema tracks`, `schema yard`, `schema joint` (fields: tracks `tracks` integer 1 to 12, `spacing` number mm, `ballast` and `ties` choice on/off; yard `mains`, `left`, `right` integers, `ends` choice, `frog` integer, `spacing`, `at`, `length` numbers, `ballast`, `ties`; joint `kind` choice crossing/branch/join, `main` text, `frog` integer; crossovers stay words-only); `kinds.mjs` exports `KINDS` (`road` and `track`, each `{ line, generated, layer }`) and `kindOfLine(name)`; `doc.mjs` adds `trackId`, `SET_TRACKS`, `SET_YARD`, `SET_JOINT`, the `track` and `track-shape` roles, and `trackLines(doc)`; `Panel` gets target kinds `track` and `joint`, state `track` and `joint`, methods `editTrack(patch)`, `editJoint(patch)`, and `propose`/`draw` branches for them; `providers.mjs` adds `proposeTrack` and `proposeJoint` (Track Layer's `propose` and `proposeJoint`); server actions `track` and `joint`.

- [ ] **Step 1: Write the failing tests.** `core/tests/schemas.rs`: each schema has the listed fields, with the limits; `a_card_model_from_the_tracks_schema_fills_defaults`. Port Track Layer's 16 panel tests into `panel/test/track.test.mjs` unchanged except for imports, ids and the plug-in id; add `kinds.test.mjs`: `kindOfLine_says_road_or_track_or_null`; and in `track.test.mjs`: `a_road_line_and_a_track_line_together_get_a_plain_note` (selecting `road.a` and `track.b` gives target `other` and a `message` containing `not supported yet`), `two_track_lines_make_a_joint` (as Track Layer's), `switching_documents_drops_a_track_and_a_joint_proposal`, `a_track_card_edit_is_checked_before_it_reaches_the_plug_in` (`editTrack({tracks: 99})` sets a message and records no command). Extend `page.test.mjs`: the stub page renders a track, a yard and a joint state with the cards present.
- [ ] **Step 2: Run** `cargo +stable test --release --test schemas` and `node --test panel/test/track.test.mjs panel/test/kinds.test.mjs panel/test/page.test.mjs`. Expected: FAIL.
- [ ] **Step 3: Implement.** Add the schemas and CLI commands; port Track Layer's `doc.mjs` track parts, its prompts (`systemPrompt` as `trackSystemPrompt`, `jointSystemPrompt`), `proposeTrack` and `proposeJoint`, its `changes.mjs` labels, its layer rule (`track <id>` layer, spine and drawing); add the `track` and `joint` branches to `Panel` next to the road branches without changing any road behaviour; render the track, yard and joint cards from their schemas with `cardModel` and the settings card's control builder, with the words box staying as in Track Layer.
- [ ] **Step 4: Run** `node --test panel/test/*.test.mjs scripts/test/*.test.mjs`. Expected: PASS, with Road Builder's 62 and Plan 3's tests unchanged and Track Layer's panel tests passing.
- [ ] **Step 5: Commit** (`Panel track branch and schema-built rail cards`).

### Task 7: Documentation, gallery and handoff

**Files:**
- Create: `docs/RAIL.md` (from Track Layer's `SCENARIOS.md`), `docs/GALLERY.md`, `docs/img/` (Track Layer's gallery images), `examples/rail/*.json`, `scripts/gallery.mjs`
- Modify: `README.md`, `CHANGELOG.md`, `CLAUDE.md`, `docs/GETTING-STARTED.md`, `docs/PANEL-CHECK.md`, `docs/superpowers/plans/NEXT.md`

- [ ] **Step 1: Write the documents.** README status 0.4.0 (roads and track in one plug-in); CHANGELOG `0.4.0 Track`; CLAUDE.md gets Track Layer's traps (fuel: `hypot` and `round` slow, each number written costs hundreds, the tie budget; stroke dashes ignored so ties are not dashed; bisector normals at vertices; winding of overlapping shapes; empty objects refused; the three output layers roadbed, ties, rails) and the rule that rail output settings are set around rail writes only; `docs/PANEL-CHECK.md` gets rail steps for the owner (draw a line, describe a yard in words, join two lines); `NEXT.md` becomes the brief for Plan 5 (styles and the target print scale), listing the open items (road meets track, `set_feature`, schema-built road cards, `water.rs` closed shapes, the real point limit of the new plug-in).
- [ ] **Step 2: Run the full gate:** `cargo +stable test --release --workspace`, `node --test panel/test/*.test.mjs scripts/test/*.test.mjs`, `node scripts/check.mjs`, `node scripts/standards.mjs check`, the road parity gate and the rail parity gate; push; confirm CI green on both systems.
- [ ] **Step 3: Open a draft pull request** with the test totals, the parity results and the calls below. Do not merge. Say plainly that the rail panel has been tested against the fake VectorCraft only.

## Calls made in this plan (check these)

- Two standards files under one tool and one CI step, not one merged file: Track Layer's sources lack the `kind` and copyright fields Road Builder's checker requires, and NMRA's copyright statement is not something to write from memory.
- Rail modules keep model millimetres inside and convert at the edge with `PT_PER_MM = 0.16`; `Out`'s decimals and thinning are per-kind settings set only around rail writes.
- `build_into` and `spine_style` are the only changes to Track Layer's builder; `rail_builder` keeps Track Layer's own `build`, `build_text` and `set_*_text` so its tests pass unchanged.
- Rail parity is the ported tests, a scaled comparison within 0.001 points, the fuel check, and the owner's look at the rendered examples.
- Track Layer's `providers.test.mjs` (17) and `server.test.mjs` (5) are not ported again: Plan 3's copies from Road Builder cover the same code; its 16 panel tests are ported.
- Cards for tracks, yards and joints are schema-built; crossovers and anything not a plain field stay in the words box.
- A road line and a track line selected together get a plain "not supported yet" note.

## Later plans

Plan 5: styles and the target print scale. Plan 6: legacy `.pmap`, migration, release 1.0, deprecation of the old repos. Separate small plan: road meets track (a level crossing between a road line and a track line).
