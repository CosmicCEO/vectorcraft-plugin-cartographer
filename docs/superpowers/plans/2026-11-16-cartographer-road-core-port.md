# Cartographer Road Core Port Implementation Plan (Plan 2)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port Road Builder's road network (cross-sections, roads, lane changes, junctions, intersections, scenarios, hubs, gores, water, embankments, bridges, culverts and causeways) into Cartographer with every Rust test passing unchanged, and prove parity with the old plug-in by identical SVG output and equal build cost.

**Architecture:** Port first, generalise later. `crosssection.rs` is already the band engine (ordered bands from a material library, measured outward from a line), so it moves as it is; the kind registry that spec section 6 describes arrives with the second kind (rail, Plan 4), where it earns its keep. Command ids and generated shape names stay the same, so Road Builder documents and tests carry over. The document record keeps Cartographer's `schema` and `settings` around Road Builder's own sections.

**Tech Stack:** Rust (edition 2024, `serde_json` only), Node 22 (zero dependencies), headless VectorCraft 0.24.1 (macOS) for the parity gate.

**Spec:** `docs/superpowers/specs/2026-11-16-cartographer-design.md` (section 10, Phase 2). **Source being ported:** `/Volumes/Mingus/Developer/vectorcraft-road-builder` at commit `ca577ca` (v0.7.0 plus water conforming). **Spike results that bind this plan:** `docs/spikes/S1-fuel.md`, `S3-survival.md`.

## Global Constraints

- Everything in Plan 1's Global Constraints still holds: edition 2024, `resolver = "3"`, `serde_json` the only dependency, Node 22, scripts in Node, 1 document point = 1 metre, names `<kind>.<id>` and `gen.<kind>.<id>.<piece>.<role>`, plug-in id `com.cosmicceo.cartographer.build`, light filter only, never push to `main`, never merge or tag, commit trailers `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>` and `Claude-Session: https://claude.ai/code/session_01HBq3ghaocqFgSq9fTWECkM`.
- **Parity:** every ported Rust test keeps its expected numbers and its test name; a test may change only its crate name (`road_builder_core` becomes `cartographer_core`). Behaviour matches Road Builder `ca577ca`.
- **Host limits (S1):** a build is refused at about 2,000 input points (1,500 built); a note is added at more than 1,200 points. Output never contains an empty object.
- **Document data:** a record from a newer schema is refused by every command and by Build; `schema`, `settings` and every unknown key survive every command.
- Not in this plan: the panel (Plan 3), Track Layer (Plan 4), styles and print scale (Plan 5), the legacy centreline model, `.pmap` importer and exporter, and the `add_junction` command (Plan 6 or dropped, the owner's call).

## Review Focus

1. A document from a newer schema is refused by Build and by each of the eight `set_*` commands, and nothing is written (Task 5).
2. `settings`, `schema` and unknown keys in the document data survive every command (Task 5).
3. A build of more than 1,200 input points gets a note naming the count; 1,199 gets none (Task 5).
4. A line with one point, a closed shape named as a road, or a document with no road or water lines never produces an empty object or a crash; the last gives a plain message (Task 5).
5. A stored crossing, hub or wet crossing that names a road or water id that is not in the document gives a note, not a crash (Task 5).

---

## Tasks

### Task 1: Parity harness (written first, fails first)

**Files:**
- Create: `scripts/lib/svgcompare.mjs`, `scripts/parity.mjs`, `scripts/parity/scenarios.json`
- Test: `scripts/test/svgcompare.test.mjs`

**Interfaces:**
- Consumes: `headless(cli, root)` from `scripts/lib/headless.mjs`.
- Produces: `normalizeSvg(text: string): string` (drops the XML header, `id`/`data-*` attributes that differ per session, collapses whitespace, rounds numbers to 3 decimals); `compareSvg(a: string, b: string): { same: boolean, summary: string }` (summary names the first differing element); `scenarios.json`: `[{ name, size: [w, h], lines: [{ points: [[x, y]...], name?: string }], steps: [{ select: number[], command: string, params: object }] }]`; `parity.mjs <vectorcraft-cli> <road_builder_dir> [--png]` prints `identical` or the summary per scenario and exits 1 on any difference.

- [ ] **Step 1: Write the failing tests** in `svgcompare.test.mjs`: `normalizeSvg ignores ids and rounds numbers` (`<path id="a1" d="M1.00004 2"/>` and `<path id="b9" d="M1 2.0000"/>` normalise equal); `compareSvg says same for equal geometry`; `compareSvg names the first differing element` (two SVGs differing in one path's `d`: `same` false and the summary contains that path's index).
- [ ] **Step 2: Run** `node --test scripts/test/svgcompare.test.mjs`. Expected: FAIL (module missing).
- [ ] **Step 3: Implement `svgcompare.mjs`** (regex based, no parser).
- [ ] **Step 4: Run the tests.** Expected: PASS (3).
- [ ] **Step 5: Write `scenarios.json`** with these named scenarios, parameters taken from the matching Road Builder test: `two-lane`, `divided-story` (the `STORY` cross-section in `core/tests/crosssection.rs`), `lane-add-drop`, `junction-tee`, `four-way-stop`, `signal-pockets-slip`, `roundabout`, `overpass`, `on-ramp-gore`, `hub-three`, `creek-bridge`, `culvert`, `causeway-pond`, `embankment`, `creeks-joined`.
- [ ] **Step 6: Write `parity.mjs`.** For each scenario: in one headless session install the old plug-in files from `<road_builder_dir>/target/wasm32-unknown-unknown/release` (copied under `target/parity/` because a session reads only inside its root), draw the lines, run the steps as `plugin.com.cosmicceo.roadbuilder.build.<command>` then `plugin.run`, export SVG; in a second session do the same with `cartographer_filter.wasm` and `plugin.com.cosmicceo.cartographer.build.<command>`; compare with `compareSvg`. With `--png` also export the new result to `target/parity/<name>.png` at scale 10.
- [ ] **Step 7: Run** `node scripts/parity.mjs <cli> /Volumes/Mingus/Developer/vectorcraft-road-builder`. Expected: FAIL for every scenario ("unknown command ...cartographer.build.set_cross_section"). This is the gate Task 6 turns green.
- [ ] **Step 8: Commit** (`Parity harness: scenarios and SVG comparison (red until the port is done)`).

### Task 2: Cross-sections and the element library

**Files:**
- Create: `core/src/crosssection.rs`, `core/src/library.rs`, `library/elements.json`
- Modify: `core/src/lib.rs`
- Test: `core/tests/crosssection.rs`

**Interfaces:**
- Produces (same names and signatures as Road Builder): `crosssection::{Anchor, Dir, Parsed, Placed, parse, format, layout}`, `library::{Library, Role}` with `Library::builtin()`.

- [ ] **Step 1: Copy `core/tests/crosssection.rs`** from Road Builder, changing only the crate name.
- [ ] **Step 2: Run** `cargo +stable test --release --test crosssection`. Expected: FAIL to compile (modules missing).
- [ ] **Step 3: Copy `crosssection.rs`, `library.rs` and `library/elements.json`** unchanged (the `include_str!` path is already `../../library/elements.json`), add both modules to `lib.rs`.
- [ ] **Step 4: Run the test.** Expected: PASS (9 tests).
- [ ] **Step 5: Commit** (`Port cross-sections and the element library`).

### Task 3: Standards database

**Files:**
- Create: `core/src/standards.rs`, `library/standards.json`, `scripts/standards.mjs`, `docs/STANDARDS.md`
- Modify: `core/src/lib.rs`, `.github/workflows/ci.yml`
- Test: `core/tests/standards.rs`

- [ ] **Step 1: Copy `core/tests/standards.rs`** (crate name only). **Step 2: Run** it. Expected: FAIL to compile.
- [ ] **Step 3: Copy `standards.rs`, `library/standards.json`, `scripts/standards.mjs`** (add the `require-node.mjs` import first) and `docs/STANDARDS.md`; add the module to `lib.rs`; add to CI `node scripts/standards.mjs check` and `node scripts/standards.mjs md` followed by `git diff --exit-code docs/STANDARDS.md`.
- [ ] **Step 4: Run** the test (expected PASS, 4 tests) and `node scripts/standards.mjs check` (expected `ok: 59 entries`).
- [ ] **Step 5: Commit** (`Port the standards database`).

### Task 4: The road network modules

**Files:**
- Create: `core/src/{roads,transition,embank,junction,crossing,scenarios,hub,water,wet}.rs`
- Modify: `core/src/lib.rs`, `core/src/geom.rs` (receives `simplify`)
- Test: `core/tests/{roads,transitions,embankments,junctions,crossings,hubs,water,wets}.rs`

**Interfaces:**
- Consumes: Task 2's `crosssection` and `library`, Task 3's `standards`, the framework from Plan 1.
- Produces (unchanged from Road Builder): `roads::{Spec, Style, CURB, flip, heading, emit, boundary, boundary_line, segment_id, build_clipped, anchor_from}` where `Spec::from_document_data(&Value) -> Spec`, `Spec::to_value(&self) -> Value`, `Spec::set_notes(&mut self, &[String])`; `crossing::{trims, build, Trim, CrossingSpec, apply_trims}`; `hub::{trims, build, HubSpec}`; `junction::{build, junction_id, JunctionSpec}`; `transition::TransitionSpec`; `embank::EmbankmentSpec`; `water::{build, water_id, WaterSpec}`; `wet::{build, WetSpec}`; `geom::simplify(pts: &[P], tol: f64) -> Vec<P>` (moved from `builder.rs`, still `pub(crate)`-level behaviour, now `pub`).

- [ ] **Step 1: Copy the eight test files** (crate name only). Expected counts: roads 13, transitions 9, embankments 5, junctions 10, crossings 35, hubs 6, water 10, wets 8 (96).
- [ ] **Step 2: Run** `cargo +stable test --release --tests`. Expected: FAIL to compile (modules missing).
- [ ] **Step 3: Copy the nine modules** and add them to `lib.rs`. One change is needed: `roads.rs` imports `crate::builder::simplify`; move `simplify` into `geom.rs` and import it from there, so no module depends on `builder`. Nothing else changes.
- [ ] **Step 4: Run each test file in turn**, in this order, reading each count: `roads` (13), `transitions` (9), `embankments` (5), `junctions` (10), `crossings` (35), `hubs` (6), `water` (10), `wets` (8). Expected: all PASS, and `cargo +stable build --release` prints no warnings.
- [ ] **Step 5: Commit** (`Port the road network modules`).

### Task 5: Commands, plug-in manifest, CLI and the document guard

**Files:**
- Modify: `core/src/builder.rs`, `filter/src/lib.rs`, `cli/src/main.rs`
- Create: `core/tests/build_guard.rs`, `cli/tests/cli.rs`

**Interfaces:**
- Consumes: Task 4's modules; Plan 1's `storage::Doc`.
- Produces: `builder::build_text(input: &Value, params: &Value) -> Result<String, String>` and `builder::{set_cross_section_text, set_junction_text, set_crossing_text, set_hub_text, set_transition_text, set_water_text, set_embankment_text, set_wet_text}` with the same `(input: &Value, params: &Value) -> Result<String, String>` signature as in Road Builder; private `load_spec(params: &Value) -> Result<Spec, String>` (runs `Doc::from_document_data` on `params._context.documentData` first, so a newer schema is refused, then builds the `Spec` from the same data); `const NEAR_LIMIT_POINTS: usize = 1_200`. Plug-in command ids: `set_settings`, `set_cross_section`, `set_junction`, `set_crossing`, `set_hub`, `set_transition`, `set_water`, `set_embankment`, `set_wet`; a run with no command is Build. Manifest version `0.2.0`. CLI commands: the Plan 1 ones plus `parse`, `edit`, `diff`, `parse-junction`, `parse-crossing`, `parse-hub`, `parse-transition`, `parse-water`, `parse-embankment`, `parse-wet`, `library`, with Road Builder's arguments and output.

- [ ] **Step 1: Write the failing tests.** `core/tests/build_guard.rs` (each builds a one-line `road.a` input with `segments.a.cross_section = "lane back | lane fwd"`):
  - `every_command_refuses_a_newer_schema`: with `documentData` `{"schema": 9}`, `build_text` and each of the eight `set_*_text` return an `Err` containing `newer Cartographer`.
  - `settings_and_unknown_data_survive_every_command`: with `{"schema":1,"settings":{"seed":7,"sheet":{"size":"A0"}},"extra":{"k":1}}`, run `build_text` and `set_cross_section_text`; the output `documentData` still holds `settings.seed == 7`, `settings.sheet`, and `extra.k == 1`.
  - `nothing_to_build_is_a_plain_message`: input with only a plain line gives an `Err` containing `Nothing to build` and `road.<id>`.
  - `a_one_point_line_and_a_closed_road_shape_make_no_empty_object`: a `road.a` with one anchor, and a closed three-anchor `road.b`, give `Ok` output in which every object named `gen.*` has at least one subpath with 2 or more anchors.
  - `a_crossing_hub_or_wet_naming_a_missing_road_is_survived`: stored `crossings.k1 {roads:["a","zz"], treatment:"signal"}`, `hubs.h1 {roads:["a","zz","yy"]}` and `wets.x1 {road:"zz", water:"w", treatment:"bridge"}` with only `road.a` in the document: `build_text` returns `Ok` (no panic).
  - `a_big_build_gets_a_note_naming_the_point_count`: a `road.a` of 1,199 points gives no note; of 1,300 points the stored `notes` contain a string with `1300` and `split`.
  - `cli/tests/cli.rs`: `parse_prints_the_total_width` (`cartographer-cli parse "lane back | lane fwd"` prints `"ok":true` and `"total_width":7.2`), `parse_crossing_accepts_signal`, `a_bad_command_is_a_plain_error` (exit code 1, `"ok":false`).
- [ ] **Step 2: Run** `cargo +stable test --release --test build_guard` and `-p cartographer-cli`. Expected: FAIL (functions and commands missing).
- [ ] **Step 3: Port `builder.rs`:** add Road Builder's `build_text`, `build` and the eight `set_*_text` functions; leave out `build_legacy`, `add_junction_text`, `STYLE`, `runs`, `painted` and `side_edge` (the legacy model, Plan 6). Keep Plan 1's `set_settings_text` and `settings_summary`. Start every command with `load_spec`. In `build_text`, count the points of all input lines and add the note `this build has <N> points; VectorCraft refuses a build of about 1,500 or more points, so split long roads` when `N > NEAR_LIMIT_POINTS`. Change the "Nothing to build" message to drop the pmap mention.
- [ ] **Step 4: Extend the filter manifest and `run`** (copy Road Builder's manifest commands except `add_junction`, bump to `0.2.0`, dispatch on `params._context.command`, no command means `build_text`), and port the CLI commands.
- [ ] **Step 5: Run** `cargo +stable test --release --workspace`. Expected: PASS, including the 96 ported tests, Plan 1's tests, and the new ones.
- [ ] **Step 6: Commit** (`Port the commands, manifest and CLI with the document guard`).

### Task 6: Parity gate

**Files:**
- Create: `scripts/fuel-parity.mjs`, `docs/PARITY.md`

**Interfaces:**
- Consumes: Task 1's `parity.mjs`, `tools/fuel`, `scripts/spikes/fuel-inputs.mjs`, the old and new wasm files.

- [ ] **Step 1: Build** (`node scripts/build.mjs`) and run `node scripts/parity.mjs <cli> /Volumes/Mingus/Developer/vectorcraft-road-builder`. Expected: every scenario prints `identical`. A difference is a bug in the port: find it with superpowers:systematic-debugging, fix the code, never loosen the comparison.
- [ ] **Step 2: Write `scripts/fuel-parity.mjs`:** runs `tools/fuel` for `road600`, `bridge` and `road20km` (inputs from `scripts/spikes/fuel-inputs.mjs`) against the old `road_builder_filter.wasm` and the new `cartographer_filter.wasm`, prints both numbers and exits 1 if any new value exceeds 1.05 times the old. (The new filter's `road20km` input carries a note, so compare fuel only, not output size.)
- [ ] **Step 3: Run it.** Expected: three lines with a ratio at or below 1.05.
- [ ] **Step 4: Render** `node scripts/parity.mjs <cli> <road_builder_dir> --png` and keep the PNGs for the owner's review (`two-lane`, `divided-story`, `signal-pockets-slip`, `four-way-stop`, `creek-bridge`, `creeks-joined`).
- [ ] **Step 5: Write `docs/PARITY.md`:** the scenario list, what "identical" means (normalised SVG), the fuel table, and how to rerun. **Commit** (`Parity gate: identical SVG and equal fuel`).

### Task 7: Documentation and handoff

**Files:**
- Modify: `README.md`, `CHANGELOG.md`, `CLAUDE.md`, `docs/superpowers/plans/NEXT.md`
- Create: `docs/SCENARIOS.md` (copied from Road Builder)

- [ ] **Step 1: Copy `docs/SCENARIOS.md`**, replacing the plug-in name and the `road-builder-cli` references. Update `README.md` (status 0.2.0: the road network core without the panel; how to use the commands through VectorCraft's own command menu; the parity gate), `CHANGELOG.md` (`0.2.0 Road network core`), and `CLAUDE.md` (add Road Builder's traps that apply: transitions `derive` and `align`, trims and gap trims, touching roads extended 30 m, right-hand traffic, water layers below roads, ring offsets by winding).
- [ ] **Step 2: Run the full gate:** `cargo +stable test --release --workspace`, `node --test panel/test/*.test.mjs scripts/test/*.test.mjs`, `node scripts/check.mjs`, `node scripts/standards.mjs check`; push the branch; confirm CI green on both systems.
- [ ] **Step 3: Replace `NEXT.md`** with the brief for Plan 3 (the panel port: `classify`, layers, the road, junction, crossing, hub, lane-change, water, embankment and crossing cards built from schemas, the agent prompts and providers, `changes`, and the 62 Road Builder panel tests with their numbers unchanged) and Plan 6 (legacy `.pmap`, `add_junction`, migration).
- [ ] **Step 4: Open a draft pull request** to `main`; in the description list the parity results and the calls below. Do not merge.

## Calls made in this plan (check these)

- Port first, generalise later: the kind registry and `set_feature` arrive with the second kind (rail), not now.
- Command ids and `gen.*` names stay as in Road Builder.
- The legacy centreline model, `.pmap` importer and exporter, `add_junction`, and their tests (`roundtrip`, `real_map`) wait for Plan 6, where the owner may drop them.
- `simplify` moves from `builder.rs` to `geom.rs` so no module depends on `builder`.
- The near-limit note triggers above 1,200 points (S1: 1,500 built, 2,000 refused).
- Version `0.2.0`; parity means normalised SVG equal and fuel at most 1.05 times Road Builder's.
- The panel tests are not ported here; they are part of Plan 3, and the parity gate for release 1.0 still requires them.

## Later plans

Plan 3: the panel port. Plan 4: Track Layer and the rail kind (the registry, `set_feature`, the standards-based library). Plan 5: styles and the target print scale. Plan 6: legacy `.pmap`, migration command, release 1.0 and deprecation of the old repos.
