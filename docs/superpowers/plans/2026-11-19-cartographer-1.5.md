# Cartographer 1.5 Plan (autonomous)

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:executing-plans (inline). One ledger per plan. Each work package below is its own branch, PR, review and merge; the release is the last package.

**Goal:** 1.5.0: the first release after 1.0 that closes what 1.0 left open: everything deferred as minor in Plans 4 to 6, the first user-visible gaps (a road crossing a track, print minimums for water and symbols, the stale 1.0.0 release), and the two engineering debts that limit document size (rail fuel, the unmeasured point limit). Run start to finish by the agent without the owner, under the autonomy rules below.

**Spec:** `docs/superpowers/specs/2026-11-16-cartographer-design.md` (sections 6.4 to 6.6, 7, 10), `docs/STYLES.md`, `docs/superpowers/plans/NEXT.md`, and the ledgers' deferred minors (copied below so this plan stands alone).

## What 1.0/1.0.1 did

Roads (Road Builder parity, 15 scenarios identical), railway track (Track Layer parity, 8 scenarios identical when scaled), styles and target print scale, migration of old documents (live-checked against both real plug-ins), the panel with schema-built settings/track/yard/joint cards, release workflow on macOS and Linux, package fix (1.0.1).

## What was deferred (the inventory)

| # | Item | From | Effect on a user |
|---|---|---|---|
| D1 | Describe Track can rename `centerline`, `profile.`, `flag.`, `arrow.`, legacy `J7` lines | Plan 4 | a menu command can rename a line it should not |
| D2 | `build_into` leaves `out.layer` on Rails | Plan 4 | latent: anything written after rail lands on the rails stack |
| D3 | 30-note cap keeps road notes first; track notes can vanish | Plan 4 | missing warnings on big mixed documents |
| D4 | Track spine width written 0.042, docs say 0.0423 | Plan 4 | doc accuracy only |
| D5 | `skipCollapsedRings` removes every new ring near a point, counts the old side only | Plan 4 | the rail parity gate could hide an extra ring |
| D6 | Clearing a settings number sends null (default) | Plan 4/5 | seed jumps to 1 instead of 0 |
| D7 | `docs/RAIL.md`: wrong test-file name, broken table row | Plan 4 | doc accuracy |
| D8 | "Drawn."/"Rebuilt." message lost after Draw (forced refresh) | Plan 4 | no confirmation line after Draw |
| D9 | Hand-drawn stroke factor keyed on object name: a marking steps in width where two segments join | Plan 5 | visible step at a join |
| D10 | Print-minimum stroke also widens crosswalk bars, arrows, culvert pipes; at 1:10000 crosswalks print solid | Plan 5 | wrong drawing at small scales |
| D11 | Settings Draw: settings stored but rebuild fails says "Could not draw" and keeps the proposal; restores replaced ids | Plan 5 | confusing message |
| D12 | Junction offsets (radii, setbacks) do not scale with 1:N | Plan 5 | junctions look cramped at small scales |
| D13 | Migration: integers above 2^53 and float formatting rewritten | Plan 6 | a huge `seed` could be corrupted |
| D14 | Migration: key collisions between the two old records are silent | Plan 6 | latent |
| D15 | Migration: user objects named `gen.*` deleted unreported | Plan 6 | silent loss if a user named something `gen.` |
| D16 | Migration: only `kind.path` is scaled (compound paths unconfirmed) | Plan 6 | a compound track line would not be scaled |
| D17 | Migration: a document with both records scales tracks about the origin, roads not, no warning | Plan 6 | relative position changes silently |
| D18 | Migration command: dangling-symlink output, missing folder stack trace, non-atomic write | Plan 6 | rough edges |
| D19 | Live migrate check could pass vacuously (no assert that the migrated doc is active / rebuilt; no Bezier) | Plan 6 | weaker proof |
| F1 | Road meets track: a level crossing between `road.<id>` and `track.<id>` (today "not supported yet") | NEXT | the most visible missing feature |
| F2 | Minimum print size for water widths and symbols | STYLES | water vanishes at small scales next to widened roads |
| F3 | Black-white hatch and stipple tones (spike: can a filter write a pattern fill?) | STYLES | b/w looks flat grey |
| F4 | Schema-built road cards | NEXT | panel consistency; agents get one card model |
| E1 | Rail fuel 15 to 35% above Track Layer's (shared `Out` copies and checks every path) | PARITY-RAIL | big yards hit the host limit sooner |
| E2 | Real point limit of the plug-in not re-measured; `water.rs` closed-shape test by coordinates | NEXT | a note at 1,200 points may be wrong |
| R1 | The v1.0.0 release still offers the broken package | 1.0.1 | a user can download a package that cannot install |
| R2 | Old repositories not retired (banners, archive) | RELEASE-1.0 | users find the old plug-ins |

**Not in 1.5 (deliberately):** `set_feature` folding (no user effect; a 2.0 API change), labels/scale bar/legend (a filter cannot make text: needs its own design), trails/forests/swamps/estuaries, per-element styles, the legacy centreline model and `.pmap` (dropped by decision).

## Autonomy rules (how the agent runs this without the owner)

- **Rulings, not stops.** Every decision the owner would otherwise make is taken, recorded as `Ruling:` in the ledger with what it costs if wrong, and listed in the PR and the release notes.
- **Gates every package must pass before merge:** Rust and Node suites green, no warnings, `check.mjs`, `standards.mjs check`, road parity 15/15, rail parity 8/8, fuel parity rows ok, CI green on macOS and Linux (a runner timeout is re-run, never ignored), and a fresh-context review (Opus) with one fix pass (RED test first per finding).
- **Live checks on the real VectorCraft** whenever a package touches what is drawn or the panel; pictures read by the agent.
- **Parity is not loosened.** A change that alters medium-contrast output at scale `true` must be a deliberate, ruled scenario change (none is expected in 1.5 except F1, which adds new scenarios and changes none).
- **Stops (only these):** a destructive or irreversible action outside the repository other than tagging/publishing this repository (e.g. archiving or force-pushing the old repositories), a security-sensitive action, or a gate that cannot be made green without loosening it. On a stop the agent writes `needs input:` with the one thing it needs.
- **Outward actions allowed:** branches, PRs, merges with merge commits (no squash), tags `v1.5.0` and pre-release-free, release text edits on existing tags, PRs (not merges, not archives) on the two old repositories.

## Work packages (each: branch, TDD, gates, review, merge)

### WP1 Release hygiene (R1, R2) — 1.0.2 is not cut; text only
- Edit the v1.0.0 release notes: "Do not use: the package cannot install; use 1.0.1 or later" with a link. (Allowed: release text on an existing tag.)
- Open (not merge) a PR on each old repository adding the banner from `docs/RELEASE-1.0.md`. Archiving stays the owner's (irreversible outside this repo).

### WP2 Correctness and hygiene sweep (D1 to D8, D11, D13 to D19)
Each item gets a failing test first. Highlights of the intended fixes:
- D1: share one reserved-name rule between `builder::is_reserved_name` and `rail_builder::is_reserved_name` (one function in `vcdoc` or `builder`), tested with every reserved form.
- D2: `build_into` saves and restores the layer like the other settings (test: a road written after a track lands on the roadbed stack).
- D3: cap notes per kind fairly (road and track each keep up to 15, then the rest), tested with 40 road notes + 1 track note.
- D5: `skipCollapsedRings` requires exactly one new ring per collapsed old point; count both sides.
- D6: an emptied number field sends nothing (keeps the stored value) rather than null; settings card test.
- D8: keep a Draw result message through the forced refresh for every kind (roads included: a deliberate panel behaviour change, ruled).
- D11: settings stored then rebuild failure: proposal cleared, message "Settings saved; the rebuild failed: …".
- D13: migration rewrites only the JSON paths it changes (splice the plug-in data and the edited path nodes into the original text) so numbers elsewhere are byte-identical; or, if splicing is unsound, parse numbers losslessly (big-integer-safe reviver). Ruling after a spike.
- D14 to D18: report collisions, report `gen.*` user objects by name before deleting (and keep a layer that holds non-`gen` children), scale compound paths after confirming the format from a real saved compound path, warn when both records are present, write via temp file + rename with `wx` semantics and refuse dangling symlinks.
- D19: the live migrate check asserts the active document is the migrated one, that Cartographer's record is present, that the `gen.*` id set changed, and adds a Bezier track line.
- D4, D7: doc fixes.

### WP3 Drawing fixes at small scales (D9, D10, D12, F2)
- D9: stroke factor keyed on the feature id plus the marking role (e.g. `road.a/line.1`), not the object name, so joined segments of one marking share a width; joins of different roads may still differ (ruled).
- D10: the print-minimum stroke applies to edge and lane lines only; crosswalk bars, arrows and pipes keep their geometry and get a minimum **gap** instead (bars thinned to keep the pitch readable); tested at 1:10000.
- D12: junction radii and setbacks grow with the widened bands (they already read `layout`; scale the fixed offsets by the same band-widening ratio, capped).
- F2: water width gets the band minimum (0.3 mm on paper) about its own line; symbols (arrows, signals) get a minimum size. Tests at 1:24000 and parity at `true`.

### WP4 Road meets track (F1)
- Data: a `level_crossings` record `{road, track, treatment: "level"|"gated"|"overpass"}` set by a new command `set_level_crossing` (Make Level Crossing), selected as one road line and one track line.
- Drawing: the road paving runs across; the track's roadbed and ties stop at the road edges; rails continue across the road with a flangeway band; `gated` adds crossbucks/gate arms using the existing railroad scenario symbols; `overpass` lifts the road (reuse the overpass treatment).
- Panel: the mixed selection (today "not supported yet") becomes a level-crossing card (schema-built), with an agent prompt.
- Tests: Rust scenario tests with hand-worked numbers, panel tests, a new parity-free scenario set rendered and looked at (no old plug-in had this), fuel row.

### WP5 Black-white hatch and stipple (F3) — spike first
- Spike S6: can a filter output a pattern fill (or only solid)? Try VectorCraft's pattern fill JSON from a saved document. If yes: classes map to hatch angles/densities; if no: draw hatch as clipped line objects inside each fill with a fuel cap, or rule F3 out of 1.5 with the reason.

### WP6 Engineering debt (E1, E2)
- E1: make `Out::open_object` avoid the per-path copy (filter in place / write straight from the iterator) for rail and road; target rail ≤ 1.20× Track Layer; road parity identical; fuel table updated.
- E2: measure the real point limit with `tools/fuel` and a live VectorCraft (binary search on input points); set the note threshold from it; `water.rs` uses `Item.closed`.

### WP7 Schema-built road cards (F4)
- Junction, intersection, hub, water, embankment and water-crossing cards built from `schema <kind>` like the track cards; the hand-written card code removed only where the schema card is a full replacement (panel tests unchanged in what they assert).

### WP8 Release 1.5.0
- Version 1.5.0, CHANGELOG, README status, `docs/STYLES.md`/`RAIL.md`/`MIGRATION.md` updated, NEXT.md for 2.0 (set_feature, labels, areas).
- Dry-run the release workflow, merge, CI green on main, tag `v1.5.0`, verify the published macOS package installs and runs the panel (the 1.0.1 recipe), report.

## Order and why

WP1 (minutes, protects users now) → WP2 (correctness first, small and safe) → WP3 → WP6 (perf work before new drawing work, so WP4's fuel lands on the cheaper base) → WP4 → WP5 → WP7 → WP8. If a package cannot pass its gates within its fix pass, it is cut from 1.5 by ruling and listed as deferred again; the release still ships with the rest.

## Calls made in this plan (check these)

- `set_feature` folding stays out of 1.5 (no user effect, an API change better made once for 2.0).
- The old repositories get PRs, not merges or archives (irreversible, the owner's).
- D8 changes panel behaviour for roads too (a message after Draw); ruled as a fix, not a regression.
- F1 treatments limited to level, gated and overpass; no signals or half-barriers.
