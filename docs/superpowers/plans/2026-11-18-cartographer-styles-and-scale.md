# Styles and Target Print Scale Implementation Plan (Plan 5, 0.5.0)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (inline) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The document settings `style`, `seed` and `target_print_scale`, which are stored and shown today but ignored by builds, change what a build draws: four styles (medium-contrast is today's output, unchanged) and a minimum print size for bands and strokes.

**Architecture:** One `StyleKit` (style, seed, wobble amplitude) lives in `Out`. Every colour goes through it (`tone`), every road point through a deterministic displacement field (`wobble`, hand-drawn only, a function of position so two objects that share a point move together and a join never opens), every road stroke width through a per-object factor. The target print scale becomes a minimum band width in the element `Library` (one line in `layout`) and a minimum stroke width in `Out`. Medium-contrast with scale `true` takes none of these paths, so the parity gates stay as they are. Rail is a model at true scale: it is never wobbled or widened, only its colours go through the style.

**Tech Stack:** Rust (core, filter, cli), Node (panel, scripts). No new dependencies.

**Spec:** `docs/superpowers/specs/2026-11-16-cartographer-design.md` sections 6.4, 6.5 and 7, and `docs/superpowers/plans/NEXT.md`.

## Global Constraints

- Medium-contrast with `target_print_scale` `true` must give byte-identical output to 0.4.0: `node scripts/parity.mjs` 15/15 and the rail gate 8/8 stay green; the smallest road build may cost at most 1.05 times Road Builder's (`docs/PARITY.md`).
- "The spine and centreline are never wobbled; hand-drawn jitter has a bounded amplitude so a join never opens; the style cannot change geometry that other features depend on." (spec 6.4)
- "1 mm on paper is N mm of ground, so the minimum width in metres is `min_print_width * N / 1000`"; widening is equal about the band's own centreline and never moves a spine or a join (spec 6.5). `TargetScale::min_width_m(print_mm)` already computes it.
- Rail output keeps its own decimals and thinning (`rail_builder::build_into`) and is not displaced.
- Same seed and same document give the same output; a different seed gives a different hand-drawn look.
- PHP-free, `serde_json` is the only dependency; Edition 2024; no warnings; ported tests keep their names and numbers.

## Review Focus

1. A document with only a track in it, set to hand-drawn: the rails are not displaced (gauge stays exact), colours follow the style (Task 2).
2. A style or scale that is stored but invalid, or a settings record from a newer schema: the build falls back to defaults or refuses as the commands do, never panics (Task 2).
3. Two roads that meet at a point (an intersection, a join) in hand-drawn: both objects get the same point, so nothing opens (Task 2).
4. `1:N` with a large N (1:1,000,000) or 1:1: widths and strokes stay finite, no zero or NaN (Task 3).
5. Fuel: hand-drawn on the 20 km road stays inside the host's limit and is measured (Task 4).

---

### Task 1: The style kit

**Files:**
- Create: `core/src/style.rs`, `core/tests/style.rs`
- Modify: `core/src/lib.rs` (register `style`)

**Interfaces:**
- Consumes: `storage::Style` (`BlackWhite`, `HandDrawn`, `HighContrast`, `MediumContrast`).
- Produces: `pub struct StyleKit`; `StyleKit::new(style: Style, seed: u32, amplitude: f64) -> StyleKit` (`amplitude` in metres: the largest displacement along each axis; 0 for no wobble); `StyleKit::plain() -> StyleKit` (medium-contrast); `tone(&self, rgb: [f64; 3]) -> [f64; 3]`; `wobble(&self, p: [f64; 2]) -> [f64; 2]`; `stroke_factor(&self, name: &str) -> f64`; `is_plain(&self) -> bool`.

- [ ] **Step 1: Write the failing tests** in `core/tests/style.rs`: `plain_is_the_identity` (medium-contrast: `tone` returns its input, `wobble` returns its input, `stroke_factor` is 1.0, `is_plain` true); `high_contrast_pushes_light_lighter_and_dark_darker` (`tone([0.8,0.8,0.8])` has every channel above 0.8 and `tone([0.2,0.2,0.2])` every channel below 0.2, all within 0 to 1); `black_white_has_four_grey_tones` (over a sweep of 200 colours `tone` returns only grey values from a set of exactly four, equal channels, 0.0 and 1.0 among them); `wobble_is_bounded_and_repeatable` (hand-drawn, amplitude 0.5: for 2,000 points on a grid `|dx|` and `|dy|` are at most 0.5, and the same call twice gives the same value); `wobble_is_continuous` (two points 0.01 apart move by less than 0.02 relative to each other); `wobble_depends_on_the_seed` (seed 1 and seed 2 give a different displacement at (10, 10)); `wobble_is_only_for_hand_drawn` (the other three styles return the point unchanged even with an amplitude); `stroke_factor_is_stable_per_name_and_bounded` (hand-drawn: same name same factor, between 0.85 and 1.15, two names differ somewhere over 20 names; other styles 1.0).
- [ ] **Step 2: Run** `cargo +stable test --release --test style`. Expected: FAIL (module missing).
- [ ] **Step 3: Implement `core/src/style.rs`.** `tone`: high-contrast `c' = clamp((c - 0.5) * 1.6 + 0.5)` per channel; black-white: luminance `0.2126 r + 0.7152 g + 0.0722 b` rounded to the nearest of `[0.0, 0.4, 0.75, 1.0]`, written to all three channels; the others identity. `wobble`: for hand-drawn with amplitude above 0, value noise: a lattice of cells of side `12 * amplitude` metres, a 32-bit integer hash of (cell x, cell y, seed, channel) mapped to -1..1, smoothstep interpolation; `dx` and `dy` use different channels; displacement is `amplitude * noise`. `stroke_factor`: FNV-1a hash of the name and the seed mapped to 0.85..1.15. Register the module.
- [ ] **Step 4: Run** the test file. Expected: PASS. **Commit** (`Style kit`).

### Task 2: Styles in Out and in the build

**Files:**
- Modify: `core/src/output.rs`, `core/src/builder.rs`, `core/src/rail_builder.rs`
- Test: `core/tests/style_build.rs`

**Interfaces:**
- Consumes: `StyleKit` (Task 1), `storage::Settings::from_json`.
- Produces: `Out::set_style(&mut self, kit: StyleKit)`; `Out::set_wobble(&mut self, on: bool)` (default on; `rail_builder::build_into` turns it off around rail writes and back on, even on error); `builder::style_kit(params: &Value) -> StyleKit` (reads `_context.documentData.settings`; a missing or unreadable record gives the plain kit; amplitude is `0.15` metres for `true`, `0.25 mm * N / 1000` metres for `1:N`, never above 3 metres).

- [ ] **Step 1: Write the failing tests** in `core/tests/style_build.rs`, building a road (`road.a`, 400 m, cross-section `lane back | lane fwd`) with `builder::build_text`: `medium_contrast_output_is_the_same_as_no_settings` (settings `{"style": "medium-contrast"}` and no settings give identical text); `hand_drawn_moves_the_road_and_keeps_the_line` (the `road.a` object is kept untouched, some `gen.road.*` coordinate differs from the plain build, none by more than `2 * amplitude`); `hand_drawn_is_repeatable_and_seeded` (same text twice; seed 2 differs from seed 1); `two_roads_that_cross_share_displaced_points` (hand-drawn, two roads crossing: every output point that is the same in the plain build for both objects is the same in the hand-drawn build for both); `a_track_is_never_displaced` (hand-drawn, one `track.a`: the rail coordinates equal the plain build's, only colours may differ under black-white); `colours_follow_the_style` (black-white: every colour in the output has equal channels); `a_bad_settings_record_builds_plain` (`{"style": "nope"}` builds as medium-contrast, no error) and `a_newer_schema_is_still_refused` (existing behaviour, `schema: 9`).
- [ ] **Step 2: Run** `cargo +stable test --release --test style_build`. Expected: FAIL.
- [ ] **Step 3: Implement.** In `Out`: a `style: StyleKit` and `wobble_on: bool`; `paint` applies `tone`; `open_object` and `segments` and `keep_*` do not change the user's line; the mapped points of `path`, `fills`, `polygon`, `curve`, `segments` pass through `wobble` when on and the kit is not plain; stroke widths of `path` and `curve` are multiplied by `stroke_factor(name)`. In `build_text` call `out.set_style(style_kit(params))` before writing. In `rail_builder::build_into` switch wobble off for the writes and restore it. Spine highlights (`keep_styled`, `keep_outline`) keep their colours.
- [ ] **Step 4: Run** the new tests and then the whole suite and both parity gates. Expected: PASS, road parity 15/15, rail parity 8/8. **Commit** (`Styles in the build`).

### Task 3: Target print scale

**Files:**
- Modify: `core/src/library.rs`, `core/src/roads.rs`, `core/src/crosssection.rs`, `core/src/output.rs`, `core/src/builder.rs`
- Test: `core/tests/print_scale.rs`

**Interfaces:**
- Consumes: `TargetScale::min_width_m(print_mm: f64) -> f64`.
- Produces: `Library::set_min_width(&mut self, metres: f64)` and `Library::min_width(&self) -> f64`; `layout` uses `it.width.unwrap_or(def.width).max(lib.min_width())`; `Out::set_min_stroke(&mut self, metres: f64)` (a road stroke is never thinner than this; rail strokes are not changed); constants `MIN_BAND_PRINT_MM = 0.3` and `MIN_STROKE_PRINT_MM = 0.12` in `core/src/units.rs`; `Spec::from_document_data` sets the library minimum from the settings.

- [ ] **Step 1: Write the failing tests** in `core/tests/print_scale.rs`: `a_narrow_band_is_widened_at_1_to_5000` (a cross-section with a 0.5 m element, scale `1:5000`: its placed width is `0.3 * 5000 / 1000 = 1.5` m; at scale `true` it is 0.5 m); `wide_bands_are_not_changed` (a 3.6 m lane stays 3.6 at 1:5000); `widening_is_about_the_centre` (the placed elements stay symmetric about the line: total left and right extents differ from the plain layout by the same amount on both sides for a symmetric cross-section); `the_spine_does_not_move` (the kept `road.a` object is byte-identical between scales); `strokes_have_a_minimum` (a build at `1:5000` has no `gen.road.*` stroke width below `0.12 * 5000 / 1000 = 0.6`; at `true` the plain widths stay); `extreme_scales_stay_finite` (scales `1:1` and `1:1000000` build without error and every number in the output is finite).
- [ ] **Step 2: Run** `cargo +stable test --release --test print_scale`. Expected: FAIL.
- [ ] **Step 3: Implement** as in Interfaces. `Out` gets the minimum stroke from `style_kit`'s sibling `print_min_stroke(params)` in `builder.rs`; `rail_builder::build_into` sets it to 0 around rail writes and restores it.
- [ ] **Step 4: Run** the new tests, the whole suite, both parity gates. Expected: PASS and unchanged. **Commit** (`Target print scale: minimum band and stroke widths`).

### Task 4: Fuel, release and a look

**Files:**
- Modify: `scripts/spikes/fuel-inputs.mjs`, `scripts/fuel-parity.mjs`, `Cargo.toml` and the manifest version (0.5.0), the version tests, `README.md`, `CHANGELOG.md`, `CLAUDE.md`, `docs/PANEL-CHECK.md`, `docs/GETTING-STARTED.md`, `docs/superpowers/plans/NEXT.md`
- Create: `docs/STYLES.md`, `scripts/live-style-check.mjs`

- [ ] **Step 1: Measure fuel** with `tools/fuel` for `road600` and `road20km` with `settings.style = "hand-drawn"` and with `target_print_scale = "1:5000"`; add them to `fuel-inputs.mjs` as `road600-hand` and `road20km-hand` and to `fuel-parity.mjs` as extra rows with a ceiling of 2.0 times the plain build and an absolute ceiling of 20 M. Expected: inside both; if not, make the noise cheaper (lattice cells cached per call) and rerun.
- [ ] **Step 2: Version 0.5.0**: update the version tests first (RED), then `Cargo.toml`, filter manifest, CLI. Expected: workspace tests pass.
- [ ] **Step 3: Write `docs/STYLES.md`** (what each style does, what is not done yet: hatch and stipple tones for black-white, variable stroke width beyond the per-object factor, per-element styles, a style for rail beyond colour) and update README, CHANGELOG `0.5.0 Styles`, CLAUDE.md (the style kit rules: displacement is a function of position, rail is never displaced, medium-contrast takes no new path), PANEL-CHECK (set the style on the settings card and rebuild), NEXT.md (Plan 6).
- [ ] **Step 4: Write `scripts/live-style-check.mjs`** (like `live-rail-check.mjs`): in a new document draw a road crossing a road, for each style set it through the settings card, Draw, assert the output changes the way the style says (black-white: equal-channel colours seen through `document.inspect`; hand-drawn: coordinates differ from the plain build), export a PNG per style into the output folder. Run it against the running VectorCraft and read the four pictures.
- [ ] **Step 5: Run the full gate** (`cargo +stable test --release --workspace`, `node --test panel/test/*.test.mjs scripts/test/*.test.mjs`, `node scripts/check.mjs`, `node scripts/standards.mjs check`, road parity, rail parity, fuel parity). Push, open a draft pull request against `track-port` (it needs PR #6). **Commit** (`Release 0.5.0 styles and print scale`).

### Task 5: Whole-branch review and fixes

- [ ] One fresh whole-branch review (Opus or better), one fix pass with RED tests, minors to the ledger, final report.

## Calls made in this plan (check these)

- Rail is not wobbled and not widened: it is a model at true 1:160, where gauge and ties must stay exact. Only its colours follow the style.
- Black-white is four grey tones for now, not hatch and stipple patterns (a filter can only write paths and solid fills; patterns need a spike). Hairlines are not applied.
- Hand-drawn amplitude is 0.4 m for scale `true` (0.15 m was invisible at one point per metre) and 0.25 mm on paper for `1:N`; variable stroke width is a stable 0.85 to 1.15 factor per object, not a taper along the stroke.
- Minimum print sizes: 0.3 mm for a band, 0.12 mm for a stroke (the seed materials use 0.25 to 0.3 mm). The materials library's own `min_print_width` is not wired to road elements, which have no such field.
- `set_style` is not a separate command: the existing Set Settings command and the settings card change the style, and the next Build applies it.
- High-contrast is a colour transform (contrast 1.6 around mid-grey), not a re-designed palette.
