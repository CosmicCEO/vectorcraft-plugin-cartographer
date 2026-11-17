# Plan 4 prep: Track Layer and the rail kind

Written 2026-11-16 while the owner is away. Nothing here is built. It holds what the research found (spike S5, `docs/spikes/S5-rail-port.md`), the decisions Plan 4 needs with a recommended answer for each, and the task outline I would write the plan from. Plan 4 starts when PR #3 (the panel) is merged; if it is not, the plan branch starts from `panel-port`.

## What Track Layer is (measured)

| | Track Layer 0.9.1 |
|---|---|
| Rust source | 3,004 lines: `tracks` 189, `turnout` 160 (+ generated table 43), `yard` 618, `ties` 81, `joint` 271, `connector` 104, `branch` 234, `builder` 398, `standards` 20, plus the framework Cartographer already has |
| Rust tests | 82 in 10 files: branches 14, connector 6, crossovers 10, curves 3, joints 14, offset 7 (already ported), standards 3, tracks 9, turnouts 9, yards 7 |
| Commands | `set_tracks`, `set_yard`, `set_joint` (and Build) |
| Panel | `lib/panel.mjs` 205 lines (a smaller, earlier fork of Road Builder's), 38 tests: panel 16, providers 17, server 5; the page has no hand-written cards, just the description box and a summary |
| Standards | 10 entries (8 verified from NMRA documents, 2 unverified) and one table (RP-12.35, the N turnout table) that `node scripts/standards.mjs rs` turns into `turnout_table.rs` |
| Extras | `tools/fuel` (already ported), `scripts/render.mjs` and `gallery.sh`, four example layouts (`examples/*.json`), `docs/GALLERY.md`, `SCENARIOS.md`, `STANDARDS.md` |
| Data in the document | `tracks`, `yards`, `joints`, `notes`; lines are named `track.<id>`, built shapes `gen.track.<id>.*` |

Spike S5 found that the Rust modules and 75 of their tests compile and pass in Cartographer unchanged, and that the unit change (1 point = 1 metre, so 1 mm of model = 0.16 points) needs two settings in the shared `Out` made per kind (5 decimals and a thinning tolerance of 0.00677 points for rail; roads keep 3 decimals and 0.12).

## Decisions

Each has a recommendation, so the plan can be written without waiting; the first four are the ones only the owner can settle.

1. **What is the kind called?** Track Layer's lines are `track.<id>` and its built shapes `gen.track.<id>.*`; the design (section 6.1) wrote `rail.<id>`. *Recommend keep `track`:* its tests, the gallery and the documents already use it, and the word is accurate (a line becomes track). The design's naming sentence would be corrected. Renaming means changing expected names in ported tests.
2. **Command ids and `set_feature`.** The design says one generic `set_feature` command. With two kinds a generic command buys little and costs a parameter shape change. *Recommend keep the ids* (`set_tracks`, `set_yard`, `set_joint` next to Road Builder's) and add the kinds registry as data (`kinds`: id, line prefix, generated prefix, layer prefix, schema names) that the panel and layers read; add `set_feature` when a third kind arrives. This deviates from the design's wording.
3. **Cards.** Road cards are hand-written (40 parity tests behind them). The rail card is new code with no legacy. *Recommend: the rail card is built from a schema now (the first real use of schema cards beyond settings); the road cards become schema-built in a later plan.*
4. **Road meets track.** A track line crossing a road (a level crossing) is not in either repo: Road Builder has a `railroad` treatment between two *roads* whose cross-section has a railway element, Track Layer has no roads. *Recommend leaving it out of Plan 4* and writing it as its own small plan afterwards (a crossing between a road line and a track line, using the crossing table), so Plan 4 stays a port.
5. One standards database. *Recommend merging* Track Layer's entries and its table into `library/standards.json` (sources and ids do not collide: its sources are `rp11 rp72 rp12 s32 rp71 rp1235 rp151`) and extending `scripts/standards.mjs` with the `rs` command, so there is one checker, one document and one CI step.
6. Units (a call, required by the design): rail dimensions stay in model millimetres inside the rail modules and convert at the edge with `PT_PER_MM = 0.16`; the output settings above are per kind.
7. Parity for rail: the ported tests (75) are the main gate. The SVG comparison used for roads cannot be exact because the scale differs (2.835 versus 0.16 points per mm); use a comparison that scales the old output and allows 0.001 points. A render of Track Layer's four example layouts is for the owner's eye. The fuel check (`scripts/fuel-parity.mjs`) applies as for roads.
8. Version `0.4.0`.

## Proposed tasks

1. `Out` settings per kind (decimals, thinning tolerance), tested; roads unchanged.
2. Standards: merge the two databases, the `rs` command, the `rail_standards` constants and their 3 tests.
3. The rail modules and their 72 tests, with `PT_PER_MM = 0.16`.
4. The combined builder: tracks, yards, joints and roads in one build (merged `notes`, layer order), manifest, CLI (`parse-tracks`, `parse-yard`, `parse-joint`, `find-joint`), the document guard tests for the new commands.
5. Rail parity: scenario file for Track Layer's plug-in, the scaled comparison, fuel parity, the gallery render.
6. The panel's track branch (classify, layers, targets `track` and `joint`, prompts, changes labels), Track Layer's 16 panel tests, the schema-built rail card, and the kinds registry.
7. Documentation: rail scenarios, gallery and examples, `render.mjs` made generic, README, CHANGELOG, CLAUDE.md (the Track Layer traps: fuel and the tie budget, strokes not dashed, bisector normals, winding, empty objects), the next-plan brief.
8. Gate, draft PR, whole-branch review.

## Risks

- Fuel: Track Layer budgets about 21 M fuel and its tie count was tuned at 2.835 points per mm; the new output precision adds digits. Measure with `tools/fuel` after Task 1; if rail cost rises more than 5% over Track Layer's, reduce decimals to 4.
- A document that mixes roads and tracks makes one build: its point count and fuel add up; the 1,200-point note counts both.
- The panel for tracks needs the Plan 3 merge rules: the settings card's state is separate, a changed title drops proposals, and the helper-crash handling applies.
- Track Layer's open standards items (RP-12.36 curved switch, RP-13.x guard rails, RP-7.2 tables) stay open; nothing is marked verified without reading it.

## What does not depend on the owner's panel check

Tasks 1 to 5 and 8 do not touch the panel. Task 6 does; if the check of `docs/PANEL-CHECK.md` finds a panel fault, it is fixed first on its own branch.
