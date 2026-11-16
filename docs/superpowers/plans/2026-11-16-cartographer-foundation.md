# Cartographer Foundation Implementation Plan (Phase 0 and Phase 1)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Answer the four open technical questions (spikes S1 to S4), then build the shared foundation of Cartographer: one workspace, one merged copy of the framework, units, versioned storage, library schema, a first plug-in command with a schema-driven panel card, portable scripts, and CI on macOS and Linux.

**Architecture:** One Cargo workspace (`core`, `filter`, `cli`) with the framework merged from Road Builder and Track Layer. A first end-to-end slice (document settings: target print scale, style, seed, north) proves the whole path: Rust validation, plug-in command, document data, native CLI, panel card generated from a schema. Scripts are Node, not shell. Later plans port the two old plug-ins onto this foundation.

**Tech Stack:** Rust (edition 2024, WebAssembly plug-in and native CLI), `serde_json` only, Node 22 or newer (zero dependencies) for the panel and scripts, GitHub Actions, VectorCraft 0.24.1 (`docs/plugins.md` in `storytold/vectorcraft`).

**Spec:** `docs/superpowers/specs/2026-11-16-cartographer-design.md` (approved 2026-11-16). Sources for the code being merged: `/Volumes/Mingus/Developer/vectorcraft-road-builder` (v0.7.0 on `main`) and `CosmicCEO/vectorcraft-plugin-tracklayer` (private; read-only copy at `$CLAUDE_JOB_DIR/tmp/tl` or clone it).

## Global Constraints

- Workspace: edition 2024, `resolver = "3"`, release profile `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true`; only dependency `serde_json = "1"` (the fuel tool alone also uses `wasmi`, outside the workspace).
- Node 22 or newer, zero dependencies; scripts are Node, never shell: no `sed -i`, no `open -a` outside one OS branch, no Homebrew path outside one function.
- 1 document point = 1 metre of ground. Names: user lines `<kind>.<id>`; generated shapes `gen.<kind>.<id>.<piece>.<role>`.
- Plug-in id `com.cosmicceo.cartographer.build`. Document data is one versioned record: `schema` number first.
- Host rules: a filter sees only visible, unlocked selected objects; new objects stack below kept objects; an object with no anchor points is refused (never write one); stroke dashes in filter output are ignored; the host stops a build at about 21 M fuel.
- Fuel: `f64::hypot` and `f64::round` are slow in WebAssembly (use `Len2::len2`, a truncating cast).
- CI on `ubuntu-latest` and `macos-latest`; minimum developer machine is the owner's MacBook Pro M1, 16 GB (all builds and tests must run there in a few minutes).
- Never push to `main`; work on branches; never force-push; never merge, tag or release. Commit trailer: `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>` and `Claude-Session: https://claude.ai/code/session_01HBq3ghaocqFgSq9fTWECkM`.
- Documents (README, docs, comments, commit messages) are plain prose; chat answers to the owner are short and plain. Drawings are "illustrative reconstruction, not a survey".
- Do not touch the owner's open VectorCraft document: use headless sessions (`vectorcraft-cli mcp`, macOS only) for experiments.

## Review Focus

1. A document saved by a newer schema (or with keys this version does not know) must never be overwritten or stripped: the plug-in declines with a plain message, and unknown keys survive a round trip (Task 8).
2. A bad target print scale (`1:0`, `1:-5`, `1:2.5`, `abc`, empty, `0:5`) must give a plain message naming `1:N`, never a crash or a zero width (Task 7).
3. Scripts must work with spaces and non-ASCII characters in paths, and say in one plain line what to fix when Node is older than 22, rustup or the wasm target is missing, or VectorCraft is not running (Task 11).
4. Output never contains an empty object, a zero-width stroke or a NaN: `Out` skips them (Task 6).
5. A closed shape must not be mistaken for an open line and the reverse (Task 6).

---

## Phase 0: spikes

Each spike ends with a short written result in `docs/spikes/` and, where it changes the design, a one-paragraph edit to the spec's section it affects. Work on branch `spikes` created from `design`.

### Task 1: Headless harness and spike S3 (what survives Save, Export and print)

**Files:**
- Create: `scripts/lib/headless.mjs`, `scripts/spikes/s3-survival.mjs`, `docs/spikes/S3-survival.md`
- Test: `scripts/test/headless.test.mjs`

**Interfaces:**
- Produces: `headless(cli: string, root: string): Promise<{ tool(name: string, args: object): Promise<any>, run(command: string, params?: object): Promise<any>, close(): void }>` (a `vectorcraft-cli mcp` stdio JSON-RPC client confined to `root`).

- [ ] **Step 1: Write the failing test** `headless frames requests and matches replies by id`: spawn `node -e` as a fake server that echoes `{jsonrpc, id, result:{content:[{text:"{\"ok\":1}"}]}}` for `tools/call`; assert `tool("x",{})` resolves to `{ok:1}` and an `isError` result rejects with the text.
- [ ] **Step 2: Run** `node --test scripts/test/headless.test.mjs`. Expected: FAIL (module missing).
- [ ] **Step 3: Implement `headless`** in `scripts/lib/headless.mjs`, from the working client in `$CLAUDE_JOB_DIR/tmp/lay/exp10.mjs` (initialize, `notifications/initialized`, line-delimited JSON, a map of waiting ids, `--automation-read-root` and `--automation-write-root` both set to `root`).
- [ ] **Step 4: Run the test.** Expected: PASS.
- [ ] **Step 5: Write `scripts/spikes/s3-survival.mjs`.** In a headless session: install `road_builder_filter.wasm` (path argument), create a document, draw a line, run `set_cross_section` and the plug-in so `documentData` and `gen.road.*` objects exist; then `file.saveCopy` (native `.vectorcraft`), export `svg`, `pdf`, `png` and `pmap`; reopen the saved copy with the `open_file` tool and inspect. For each artefact print: object names present (`gen.road`, `road.`), plug-in data present (the string `cross_section`), and whether the spine's anchor order (direction) is kept. Output a Markdown table.
- [ ] **Step 6: Run it** (`node scripts/spikes/s3-survival.mjs <cli> <road_builder_filter.wasm>`), paste the table into `docs/spikes/S3-survival.md` with a decision paragraph: which formats keep plug-in data, which keep names, which keep direction; whether the spec's "names carry enough to rebuild" holds.
- [ ] **Step 7: Commit** (`git add scripts docs/spikes && git commit -m "Spike S3: what survives save and export"`).

### Task 2: Spike S4 (can a filter create text objects)

**Files:**
- Create: `spikes/text-filter/Cargo.toml`, `spikes/text-filter/src/lib.rs` (outside the workspace, own `[workspace]`), `scripts/spikes/s4-text.mjs`, `docs/spikes/S4-text.md`

**Interfaces:**
- Consumes: `headless` from Task 1.

- [ ] **Step 1: Write the throwaway filter**: a `cdylib` with the five plug-in exports (copy `vc_plugin!` glue from Road Builder `core/src/abi.rs`) whose run returns one text object (`kind: "path"` and a second with `kind: "point"`, run text "Cedar Creek", style `font_family` "Source Sans 3", `size` 12), shaped as the `Type` section of `docs/plugins.md` describes.
- [ ] **Step 2: Write `s4-text.mjs`**: build with `cargo +stable build --release --target wasm32-unknown-unknown --manifest-path spikes/text-filter/Cargo.toml`, install in a headless session, run it, and print either the resulting object (use `doc_inspect`) or the host's exact refusal message.
- [ ] **Step 3: Run it** and record in `docs/spikes/S4-text.md`: created or refused (exact message); if created, whether text on a path follows the line, and whether it survives SVG and PDF export (reuse Task 1's checks); if refused, the fallback is outlined glyph paths from a bundled open font (record that decision).
- [ ] **Step 4: Commit** (`Spike S4: text objects from a filter`).

### Task 3: Spike S1 (baseline fuel)

**Files:**
- Create: `tools/fuel/` (copy `tools/fuel/Cargo.toml` and `src/main.rs` from Track Layer unchanged), `scripts/spikes/fuel-inputs.mjs`, `docs/spikes/S1-fuel.md`

**Interfaces:**
- Produces: a table in `S1-fuel.md` of measured fuel per scenario, used as the ceilings in later plans.

- [ ] **Step 1: Build the tool** `cargo +stable build --release --manifest-path tools/fuel/Cargo.toml`.
- [ ] **Step 2: Write `fuel-inputs.mjs`** producing `<tag>.in.json` and `<tag>.params.json` for: a 600 m two-lane road (`lane back | lane fwd`) with Road Builder's `build` command; the same road with a 6 m creek and a bridge (`wets`); a Track Layer 3-track ladder yard (reuse `scripts/fuel-input.mjs` from Track Layer); a 20 km road polyline (instruction-per-input-byte check).
- [ ] **Step 3: Run** the tool for each against the old repos' built wasm files, and record the numbers.
- [ ] **Step 4: Write `docs/spikes/S1-fuel.md`**: the table, the 21 M ceiling, and a starting ceiling per element kind of `1.5 x` the measured value (rounded up, my call, recorded as such).
- [ ] **Step 5: Commit** (`Spike S1: baseline fuel numbers`).

### Task 4: Spike S2 (Linux)

**Files:**
- Create: `docs/spikes/S2-linux.md`

- [ ] **Step 1: Record the verified facts** from `gh release view -R storytold/vectorcraft --json assets`: Linux app packages exist (AppImage, deb, rpm, flatpak, x86_64 and aarch64); the only `vectorcraft-cli` asset is `vectorcraft-cli-0.24.1-macos-universal.zip`, so there is no Linux headless CLI; paste the command and its output.
- [ ] **Step 2: Write the consequences**: headless renders and render comparisons are macOS-only; Linux CI runs Rust, wasm build, panel and script tests only; the plug-in itself is the same `.wasm`.
- [ ] **Step 3: Write a manual Linux checklist** for the owner or a helper (install the AppImage, start with the control port, `node scripts/check.mjs`, install the plug-in with `node scripts/install.mjs`, draw a line, run Set Settings) with the expected result for each step, and mark "owner action: run once the foundation is built".
- [ ] **Step 4: Commit** (`Spike S2: Linux facts and manual checklist`) and merge the `spikes` branch into `design` locally (fast-forward only), then open the next branch from it.

---

## Phase 1: foundation

Branch `foundation`, created from `spikes`.

### Task 5: Workspace skeleton and CI

**Files:**
- Create: `Cargo.toml`, `core/Cargo.toml`, `core/src/lib.rs`, `filter/Cargo.toml`, `filter/src/lib.rs`, `cli/Cargo.toml`, `cli/src/main.rs`, `.gitignore`, `.github/workflows/ci.yml`, `CLAUDE.md`
- Test: `core/tests/smoke.rs`

**Interfaces:**
- Produces: crates `cartographer-core` (lib), `cartographer-filter` (cdylib, name `cartographer_filter`), `cartographer-cli` (bin `cartographer-cli`); `core::VERSION: &str = env!("CARGO_PKG_VERSION")`.

- [ ] **Step 1: Write the failing test** `core/tests/smoke.rs`: `version_is_0_1_0` asserts `cartographer_core::VERSION == "0.1.0"`.
- [ ] **Step 2: Run** `cargo +stable test --release --workspace`. Expected: FAIL (no workspace).
- [ ] **Step 3: Create the workspace** as in Global Constraints (`members = ["core", "filter", "cli"]`, `exclude = ["tools/fuel", "spikes/text-filter"]`), the three crates, `.gitignore` (`target/`, `.DS_Store`, `*.in.json`, `*.params.json`, `dist/`).
- [ ] **Step 4: Write `ci.yml`**: matrix `os: [ubuntu-latest, macos-latest]`, `actions/setup-node@v4` with Node 22, rustup minimal with target `wasm32-unknown-unknown`, then `cargo test --release --workspace`, `cargo build --release --target wasm32-unknown-unknown --workspace`, `cargo build --release -p cartographer-cli`, `node --test panel/test/*.test.mjs scripts/test/*.test.mjs`, `node scripts/check.mjs --ci`.
- [ ] **Step 5: Write `CLAUDE.md`**: layout, the Global Constraints as rules, the traps list from spec section 4, how to run tests; plain prose.
- [ ] **Step 6: Run the tests.** Expected: PASS. **Commit** (`Workspace skeleton and CI`).

### Task 6: Merge the framework into `core`

**Files:**
- Create: `core/src/abi.rs`, `core/src/geom.rs`, `core/src/offset.rs`, `core/src/vcdoc.rs`, `core/src/output.rs`
- Modify: `core/src/lib.rs`
- Test: `core/tests/offset.rs`, `core/tests/framework.rs`

**Interfaces:**
- Produces (all `pub`, same names and signatures as today): `abi::{alloc, pack, run, Output}` and `vc_plugin!`; `geom::{P, Len2, sub, add, scale, dot, dash_polyline, bounds_center, Poly (new, len, at, right, offset_point, project, project_hinted), Curve}`; `offset::{MITRE_LIMIT, offset_polyline, offset_polyline_raw}`; `vcdoc::{Item, subpaths_of, items_from_filter_input, items_from_native, path_value, stroke, object}`; `output::{Out, Layer, num, CORNER_DEG}` with `Out::{new, layer, limit_ties, take_ties, keep, rename, keep_styled, keep_outline, path, segments, fills, polygon, curve, curve_fills, finish, finish_with}`.

- [ ] **Step 1: Write the failing tests.** Copy Track Layer's `core/tests/offset.rs` unchanged. In `core/tests/framework.rs` add: `closed_subpath_ends_where_it_began` (a closed three-anchor path flattens to a ring whose last point equals its first, via `items_from_filter_input`), `open_line_stays_open` (same anchors, `closed: false`: first and last differ), `keep_outline_writes_empty_fills` (output of `Out::keep_outline(7, [0.0, 0.5, 0.6], 1.0)` for a kept object contains `"fills":[]`), `out_skips_empty_and_degenerate_objects` (`Out::path` with an empty list, with one point, and `Out::polygon` with two points write no object; no `NaN` appears in `finish()`), `len2_matches_hypot` (`3.0.len2(4.0) == 5.0`).
- [ ] **Step 2: Run** `cargo +stable test --release -p cartographer-core`. Expected: FAIL (modules missing).
- [ ] **Step 3: Merge the files.** `abi.rs` is identical in both repos: copy. `geom.rs`, `offset.rs`, `output.rs`: take Track Layer's (the superset: `Len2`, `offset_polyline_raw`, layers, ties, `curve`) and add Road Builder's `Out::keep_outline`. `vcdoc.rs`: take Track Layer's and add Road Builder's closed-subpath handling in the flattener (a closed subpath with at least 3 anchors appends its closing curve and the first point). Add to `Out` the empty/degenerate guards the tests require, where missing.
- [ ] **Step 4: Run** the tests. Expected: PASS (all). **Commit** (`Merge the framework into core`).

### Task 7: Units and target print scale

**Files:**
- Create: `core/src/units.rs`
- Modify: `core/src/lib.rs`
- Test: `core/tests/units.rs`

**Interfaces:**
- Produces: `units::POINTS_PER_METRE: f64 = 1.0`; `units::N_GAUGE_M: f64 = 1.44`; `enum TargetScale { True, Ratio(u32) }` with `TargetScale::parse(&str) -> Result<TargetScale, String>`, `TargetScale::to_text(&self) -> String` (`"true"` or `"1:5000"`), `TargetScale::min_width_m(&self, print_mm: f64) -> f64` (0 for `True`); `units::ground_m_for_print_mm(print_mm: f64, ratio: u32) -> f64`.

- [ ] **Step 1: Write the failing tests:** `ground_m_for_print_mm(0.25, 5000) == 1.25`; `ground_m_for_print_mm(9.0, 160)` equals `N_GAUGE_M` within 1e-9 (9 mm at 1:160 is 1.44 m); `parse("true")`, `parse(" TRUE ")` give `True`; `parse("1:5000")` and `parse(" 1 : 160 ")` give `Ratio(5000)` and `Ratio(160)`; `Ratio(5000).min_width_m(0.25) == 1.25`; `True.min_width_m(0.25) == 0.0`; `to_text` round-trips both; each of `""`, `"abc"`, `"0:5"`, `"1:0"`, `"1:-5"`, `"1:2.5"`, `"1:1000001"` is an `Err` whose text contains `1:N`.
- [ ] **Step 2: Run** `cargo +stable test --release --test units`. Expected: FAIL.
- [ ] **Step 3: Implement `units.rs`**; accepted ratio is an integer from 1 to 1,000,000; error text "target print scale must be true or 1:N (N a whole number from 1 to 1000000)".
- [ ] **Step 4: Run.** Expected: PASS. **Commit** (`Units and target print scale`).

### Task 8: Versioned document storage

**Files:**
- Create: `core/src/storage.rs`
- Modify: `core/src/lib.rs`
- Test: `core/tests/storage.rs`

**Interfaces:**
- Consumes: `units::TargetScale`.
- Produces: `storage::SCHEMA: u32 = 1`; `enum Style { BlackWhite, HandDrawn, HighContrast, MediumContrast }` with ids `"black-white"`, `"hand-drawn"`, `"high-contrast"`, `"medium-contrast"` (`Style::parse(&str) -> Result<Style, String>`, `Style::id(&self) -> &'static str`); `struct Settings { target_print_scale: TargetScale, style: Style, seed: u32, north_deg: f64 }` with `Settings::default()` (`True`, `MediumContrast`, `1`, `0.0`; the default style is my call, see Calls) and `Settings::from_json(&Value) -> Result<Settings, String>` (missing keys keep defaults; `north_deg` outside -360..=360 is an error); `struct Doc { pub settings: Settings, .. }` with `Doc::from_document_data(&Value) -> Result<Doc, String>`, `Doc::to_value(&self) -> Value`.

- [ ] **Step 1: Write the failing tests:** `empty_data_gets_defaults_and_schema_1` (`from_document_data(&json!({}))` then `to_value()` has `"schema": 1` and `settings.style == "medium-contrast"`, `settings.target_print_scale == "true"`); `unknown_keys_survive_a_round_trip` (input `{"schema":1,"roads":{"a":1},"settings":{"seed":9}}` keeps `roads` and `seed == 9`); `a_newer_schema_is_refused_not_overwritten` (`{"schema":2}` gives an `Err` containing `newer Cartographer` and `schema 2`); `schema_zero_or_missing_is_migrated` (data with no `schema` key becomes `schema: 1`, other keys kept); `bad_settings_are_plain_errors` (`style: "neon"` error lists the four ids; `north_deg: 400` error mentions -360 and 360; `seed: -1` an error).
- [ ] **Step 2: Run** `cargo +stable test --release --test storage`. Expected: FAIL.
- [ ] **Step 3: Implement `storage.rs`**: keep the whole input `Value` as `raw`, replace `schema` and `settings` on `to_value`; migration is a function `migrate(Value) -> Result<Value, String>` called first (only 0 to 1 exists now: adds `schema`).
- [ ] **Step 4: Run.** Expected: PASS. **Commit** (`Versioned document storage`).

### Task 9: Library schema

**Files:**
- Create: `core/src/materials.rs`, `library/materials.json`
- Modify: `core/src/lib.rs`
- Test: `core/tests/materials.rs`

**Interfaces:**
- Produces: `struct Material { id: String, role: Role, default_width: f64, min_print_width: f64, class: String, height: Option<f64>, source: Option<String>, status: Status }`; `enum Role { Surface, Edge, Water, Bank, Vegetation, Structure, Mark }`; `enum Status { Verified, VerifiedSecondary, Unverified }`; `materials::load(&str) -> Result<Vec<Material>, String>`; `materials::validate(&[Material]) -> Vec<String>` (empty means valid); `materials::builtin() -> Vec<Material>` (from `include_str!("../../library/materials.json")`).

- [ ] **Step 1: Write the failing tests:** `the_seed_library_loads_and_validates` (`builtin()` has at least the twelve ids `seawall, pavement, sand, dune, grass, ballast, track, bank, riprap, mud, marsh, forest-floor`, and `validate` is empty); `duplicate_ids_are_reported`; `a_verified_entry_needs_a_source` (status `verified` with `source: None` gives a message containing `source`); `widths_must_be_positive` (`default_width <= 0` or `min_print_width < 0` reported); `class_is_dotted` (class without a dot, such as `sand`, is reported; `earth.sand` is fine).
- [ ] **Step 2: Run** `cargo +stable test --release --test materials`. Expected: FAIL.
- [ ] **Step 3: Implement** `materials.rs` and write `library/materials.json` with the twelve seed materials, all `status: "unverified"` and no source (the standards-based library is Plan 3; this seed only proves the schema), classes such as `earth.sand`, `paved.asphalt`, `water.shallow`, `rail.ballast`.
- [ ] **Step 4: Run.** Expected: PASS. **Commit** (`Library schema and seed materials`).

### Task 10: The first command, the CLI and the schema

**Files:**
- Create: `core/src/builder.rs`, `core/src/schema.rs`, `core/tests/settings.rs`
- Modify: `core/src/lib.rs`, `filter/src/lib.rs`, `cli/src/main.rs`

**Interfaces:**
- Consumes: `storage::{Doc, Settings}`, `vcdoc::items_from_filter_input`, `Out`.
- Produces: `builder::set_settings_text(input: &Value, params: &Value) -> Result<String, String>` (params `description` is a JSON string of settings keys; keeps every input object by id; writes `documentData` from `Doc::to_value`); `schema::settings_schema() -> Value` (`{"kind":"settings","fields":[{key,label,type,default,help,...}]}` with fields `target_print_scale` (text), `style` (choice with the four ids), `seed` (integer 0 to 4294967295), `north_deg` (number -360 to 360)); `builder::settings_summary(&Settings) -> String` (for example `style hand-drawn, target print scale 1:5000, seed 7, north 10 degrees`); filter command id `set_settings`, plug-in id `com.cosmicceo.cartographer.build`, manifest version `0.1.0`; CLI commands `version`, `parse-settings '<json>'` (`{"ok":true,"settings":{...},"summary":"..."}` or `{"ok":false,"error":"..."}`), `schema settings`.

- [ ] **Step 1: Write the failing tests** in `core/tests/settings.rs`: `set_settings_keeps_objects_and_stores_data` (input with objects 1 and 2, description `{"style":"hand-drawn","target_print_scale":"1:5000","seed":7,"north_deg":10}`: output keeps ids 1 and 2, `documentData.schema == 1`, `documentData.settings.style == "hand-drawn"`); `unknown_data_is_kept` (existing `documentData` with `roads` stays); `a_newer_schema_declines` (`Err` contains `newer Cartographer`); `bad_description_is_plain` (`{nope` contains `not valid JSON`; `"style":"neon"` lists ids); `summary_text` (exact string above); `schema_lists_four_fields_with_four_styles` (`settings_schema()["fields"]` has 4 entries and the `style` field's options are the four ids).
- [ ] **Step 2: Run** `cargo +stable test --release --test settings`. Expected: FAIL.
- [ ] **Step 3: Implement** `set_settings_text`, `settings_summary`, `settings_schema`; the filter manifest (one command `set_settings` with a `description` string parameter, `maxLength` 1000, default `{}`); the filter's run dispatches on `params._context.command`; the CLI subcommands.
- [ ] **Step 4: Run** `cargo +stable test --release --workspace` and `cargo +stable build --release --target wasm32-unknown-unknown --workspace`. Expected: PASS and a built `cartographer_filter.wasm`.
- [ ] **Step 5: Headless check** (macOS): install the wasm in a `headless()` session, draw a line, run `set_settings`, read the saved copy and confirm `documentData` carries the settings; paste the result in the PR description. **Commit** (`Set Settings command, CLI and schema`).

### Task 11: Portable scripts

**Files:**
- Create: `scripts/lib/env.mjs`, `scripts/lib/vc.mjs`, `scripts/lib/checks.mjs`, `scripts/build.mjs`, `scripts/install.mjs`, `scripts/check.mjs`, `scripts/package.mjs`, `scripts/vc.mjs`
- Test: `scripts/test/env.test.mjs`, `scripts/test/checks.test.mjs`, `scripts/test/package.test.mjs`

**Interfaces:**
- Produces: `rustEnv(platform: string, env: Record<string, string | undefined>, exists: (p: string) => boolean): Record<string, string>` (puts rustup's Homebrew bin first on `PATH` only when `platform === "darwin"` and `exists("/opt/homebrew/opt/rustup/bin")` or `exists("/usr/local/opt/rustup/bin")`; otherwise returns `env` unchanged); `appLaunch(platform: string, env, exists, port: number): { cmd: string, args: string[] } | { error: string }` (macOS: `open -a VectorCraft --args --control <port>`; Linux: `VECTORCRAFT_APP` if set and exists, else `vectorcraft` if on `PATH`, else an error naming `VECTORCRAFT_APP`); `checks(probe): { name: string, ok: boolean, fix?: string }[]` where `probe` supplies `{ nodeVersion, hasRustup, hasWasmTarget, hasCli, controlReachable }`; `sumsLine(name: string, bytes: Uint8Array): string` (`<sha256 hex>  <name>`); `archiveName(version: string, platform: string, arch: string): string` (`cartographer-<version>-<macos|linux>-<arm64|x86_64>.tar.gz`).
- Consumes: `scripts/lib/headless.mjs` (unchanged) for `vc.mjs`'s control-port client, which keeps Road Builder's `scripts/vc.mjs` behaviour (`node scripts/vc.mjs <method> [json]`).

- [ ] **Step 1: Write the failing tests:** `rustEnv` puts the Homebrew path first on darwin only when it exists, and leaves linux untouched; `appLaunch("darwin", ..)` returns `open` with args `["-a","VectorCraft","--args","--control","7979"]`; `appLaunch("linux", {}, () => false, 7979)` returns an error containing `VECTORCRAFT_APP`; `checks` with `nodeVersion: "v20.1.0"` yields a failing entry whose `fix` contains `Node 22`, and with all probes true every entry is `ok`; `sumsLine("Café map.tar.gz", bytes)` ends with that exact name, and `appLaunch("linux", { VECTORCRAFT_APP: "/tmp/Café maps/VectorCraft.AppImage" }, () => true, 7979)` returns that path as `cmd` unchanged; `archiveName("0.1.0","darwin","arm64") === "cartographer-0.1.0-macos-arm64.tar.gz"` and linux x64 gives `linux-x86_64`.
- [ ] **Step 2: Run** `node --test scripts/test/*.test.mjs`. Expected: FAIL.
- [ ] **Step 3: Implement the libraries and scripts.** `build.mjs` runs the cargo builds with `rustEnv`; `install.mjs` installs the wasm through the control port's `plugin.install` (no OS plug-in directory is used, so it is portable); `check.mjs` prints `ok:` or `fix:` lines and exits 1 on any failure (`--ci` skips the VectorCraft checks); `package.mjs <version>` builds, then makes the `.tar.gz` with the system `tar` (wasm, CLI binary, `panel/`, `library/`, `docs/`) and writes `SHA256SUMS`; every script uses `node:path` and `spawn` with argument arrays (never a shell string).
- [ ] **Step 4: Run** the tests and `node scripts/check.mjs --ci`. Expected: PASS.
- [ ] **Step 5: Commit** (`Portable Node scripts`).

### Task 12: Panel slice: a card built from a schema

**Files:**
- Create: `panel/server.mjs`, `panel/lib/control.mjs`, `panel/lib/cli.mjs`, `panel/lib/doc.mjs`, `panel/lib/cards.mjs`, `panel/lib/panel.mjs`, `panel/public/index.html`, `panel/public/app.js`, `panel/public/style.css`, `panel/test/fake-vectorcraft.mjs`
- Test: `panel/test/panel.test.mjs`, `panel/test/cards.test.mjs`

**Interfaces:**
- Consumes: CLI `schema settings` and `parse-settings`; plug-in command `plugin.com.cosmicceo.cartographer.build.set_settings`.
- Produces: `cardModel(schema: object, values: object): { fields: { key, label, type, value, options?, min?, max?, help }[] }` (pure); `Panel` class (`tick()`, `editSettings(patch)`, `draw()`, `cancel()`, `snapshot()`) over a `Control` client; server on `127.0.0.1:8789` with `GET /events` (SSE), `GET /state`, `POST /api/settings`, `POST /api/draw`, `POST /api/cancel`, Host and Origin checks, JSON-only bodies. `control.mjs` is copied from Road Builder `panel/lib/control.mjs` unchanged, with its tests.

- [ ] **Step 1: Write the failing tests.** `cardModel` returns one field per schema field, with the current value filled in and a `choice` field keeping its option list. Panel tests against the fake VectorCraft (copy and trim Road Builder's `panel/test/fake-vectorcraft.mjs`, handling `set_settings`): `settings card shows defaults when nothing is stored`; `a bad scale is refused before it reaches VectorCraft` (`editSettings({target_print_scale:"1:0"})` sets a message containing `1:N` and records no command); `a choice is previewed then Draw stores it` (`editSettings({style:"hand-drawn"})` creates a pending change and records no `set_settings` command; `draw()` then records exactly `select.all`, `set_settings` (one call, params `description` holding the settings JSON), and the fake document's stored data shows `hand-drawn`); `no open document is reported plainly`; `the server refuses a wrong Host header with 403`.
- [ ] **Step 2: Run** `node --test panel/test/*.test.mjs` (after `cargo +stable build --release -p cartographer-cli`). Expected: FAIL.
- [ ] **Step 3: Implement.** The page renders the card model with `textContent` only (never HTML), one input per field type (text, choice, integer, number), shows the CLI's summary text and the pending change, and has Draw and Cancel buttons; the CLI path comes from the environment variable `CARTOGRAPHER_CLI`, else `target/release/cartographer-cli`.
- [ ] **Step 4: Run** the panel tests. Expected: PASS. **Commit** (`Panel slice: settings card from a schema`).

### Task 13: Documentation, gate and handoff

**Files:**
- Create: `README.md` (replace the one-liner), `CHANGELOG.md`, `docs/GETTING-STARTED.md`

- [ ] **Step 1: Write `README.md`** (what Cartographer is, status "foundation only, not yet usable for maps", the install and test commands for macOS and Linux, the minimum spec, links to the spec and plan) and `CHANGELOG.md` (`0.1.0 Foundation`).
- [ ] **Step 2: Write `docs/GETTING-STARTED.md`**: install check, build, install into VectorCraft, start the panel, change a setting; the Linux manual checklist from Task 4 linked.
- [ ] **Step 3: Run the full gate** on the owner's machine: `cargo +stable test --release --workspace`, `node --test panel/test/*.test.mjs scripts/test/*.test.mjs`, `node scripts/check.mjs`, then push the branch and confirm the GitHub Actions run is green on both operating systems.
- [ ] **Step 4: Open a draft pull request** from `foundation` to `main` in `CosmicCEO/vectorcraft-plugin-cartographer` (do not merge); in the description list the spike results and the calls below.
- [ ] **Step 5: Write the next plan's brief** at `docs/superpowers/plans/NEXT.md`: Plan 2 is the band engine and the Road Builder port with its tests (spec section 10, Phase 2), incorporating the spike results.

## Calls made in this plan (check these)

- Default style is `medium-contrast`, default seed 1, default north 0.
- The panel port is 8789 (Road Builder 8787, Track Layer 8788).
- Archives are `.tar.gz`, not zip, because `tar` is on both systems.
- Installing uses the control port's `plugin.install`, so no OS plug-in directory is needed.
- The spikes use the old repos' built plug-ins; no Cartographer code is needed for S1 to S4.
- Fuel ceilings start at 1.5 times the measured baseline.
- The band engine, styles, standards library and every element port are in later plans, not this one.

## Later plans (written after the spike results)

Plan 2: band engine and Road Builder port. Plan 3: Track Layer port and the up-front standards library. Plan 4: styles and target print scale. Plan 5: migration command, release 1.0 and deprecation of the old repos.
