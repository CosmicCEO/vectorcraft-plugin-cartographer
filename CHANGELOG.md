# Changelog

## 1.0.0 Release
- The first release: roads, railway track, styles and the target print scale, and migration of Road Builder and Track Layer documents, all in one plug-in (0.1.0 to 0.6.0 below). Replaces Road Builder and Track Layer.
- No functional change from 0.6.0. The release workflow now tests packaging before a tag (a manual dry run), checks that the tag matches the version, and publishes only on a tag.
- Not in 1.0: a road meeting a track, the older centreline model, `.pmap` files and Add Junction (see `docs/MIGRATION.md`), hatch and stipple tones for black-white, labels.

## 0.6.0 Migration
- `node scripts/migrate.mjs <in> <out>` converts a saved VectorCraft document made with Road Builder and/or Track Layer: the stored records move to Cartographer's id, Track Layer's `track.*` lines are scaled to 0.16 points per millimetre, stale `gen.*` objects are removed (the first Build redraws them), and a plain report says what happened. It runs on a copy and never changes the input or overwrites a file.
- `scripts/live-migrate-check.mjs`: against the real old plug-ins in a live VectorCraft, the migrated documents build to the same SVG (roads identical, tracks identical when scaled).
- `docs/RELEASE-1.0.md` (what is done and what is the owner's, the commands to tag and release, wording to retire the two old repositories) and `.github/workflows/release.yml` (builds the package on macOS and Linux when a tag is pushed; never run yet).
- Not carried over, by decision: the older centreline model, `.pmap` files and Add Junction (see `docs/MIGRATION.md`).

## 0.5.0 Styles
- The document settings now change what a build draws. Four styles: `medium-contrast` (the default, identical to 0.4.0), `high-contrast`, `black-white` (four grey tones) and `hand-drawn` (a deterministic wander seeded from the document, stroke widths varied per object). The line you drew and a join never move; rail is coloured by the style but never moved.
- Target print scale: with `1:N` every road band is at least 0.3 mm wide on paper and every road stroke at least 0.12 mm (water and symbols are not held to a minimum yet); the line you drew and the joins stay put.
- **Draw this** on the Document settings card rebuilds every road and track, so the new style or scale shows at once.
- Fuel: hand-drawn costs 1.3 to 1.5 times the plain build (long edges are cut so they follow the wander and joins stay closed); medium-contrast about 0.2% more than 0.4.0. Parity gates unchanged (road 15/15, rail 8/8).
- `scripts/live-style-check.mjs` drives the settings card against a real VectorCraft and exports a picture per style. `docs/STYLES.md` lists what is not done yet (hatch and stipple for black-white, hairlines, per-element styles).

## 0.4.0 Track
- Track Layer ported as the `track` kind: a line named `track.<id>` becomes N-scale track (rails, ties, roadbed): parallel tracks, yards, NMRA turnouts, crossovers, crossings, branches and joins. Commands Describe Track, Describe Yard and Join Two Lines; Build handles roads and tracks together (tracks are drawn after roads).
- `cartographer-cli` gains `parse-tracks`, `parse-yard`, `parse-joint`, `find-joint` and `schema tracks|yard|joint`.
- The rail standards database (NMRA, `library/standards-rail.json`) is checked by the same tool as the road one.
- Panel: a track, yard and joint branch with cards built from schemas (no per-kind form code), a **Railway track (N scale)** mode for a plain line, one layer per track, and a plain "not supported yet" note when a road line and a track line are selected together.
- Output settings are per kind: rail numbers use 4 decimals and finer curve thinning, roads keep 3 and 0.12.
- Parity with Track Layer: 8 scenarios identical when scaled, with one difference on purpose (Track Layer drew each branch's ballast wedge as a point; Cartographer draws it). A build with no road in it no longer reads the road element library (0.7 M fuel saved). Rail builds cost 15 to 30% more than Track Layer; the smallest road build costs about 2% more than Road Builder (`docs/PARITY-RAIL.md`, `docs/PARITY.md`).
- `scripts/live-rail-check.mjs` drives the rail panel against a real VectorCraft (track card, yard, crossing joint, road plus track, hidden spine layer); it passed on macOS. Agent steps and Linux are still the owner's check (`docs/PANEL-CHECK.md`).

## 0.3.0 Panel
- Road Builder's companion panel ported: the road card with its element bar and edits, lane changes, junctions, intersections, hubs, water, embankments and crossings, the layers (one per feature, water under roads), and the agents (Claude, a command, an OpenAI-style endpoint, Apple Intelligence on a Mac). The 62 Road Builder panel tests pass unchanged.
- The Document settings card (target print scale, style, seed, north) sits at the bottom of the page and is read only while it is open.
- Switching to another document drops every proposal and reads the new one.
- The panel sends a content-security-policy that forbids inline script, answers a malformed address or unknown action with an error instead of stopping, and shows document and agent text only as text.
- `node scripts/build.mjs` also builds the optional Apple Intelligence helper on a Mac with Xcode tools; `docs/PANEL-CHECK.md` is the manual check against a real VectorCraft.

## 0.2.0 Road network core
- Road Builder's road network ported: cross-sections and the element library, roads, lane changes, junctions, intersections and scenarios (stops, signal, roundabout, overpass, ramps and gores, railroad, cloverleaf), hubs, water (lakes, creeks and rivers that conform where lines meet, shores), embankments, bridges, culverts and causeways. 96 Road Builder tests ported with their numbers unchanged.
- The standards database (59 entries) and its checker.
- Commands: Describe Road, Describe Junction, Make Intersection, Make Intersection of Three or More Roads, Add or Drop a Lane, Describe Water, Put Road on Fill, Road Meets Water; `cartographer-cli` gains parse, edit, diff and the parse-* checks.
- Every command and Build refuse a document saved by a newer Cartographer; a build of more than 1,200 points gets a note.
- Parity gate: identical SVG for 15 scenarios and a build cost within 5% of Road Builder's (`docs/PARITY.md`).

## 0.1.0 Foundation
- One Cargo workspace (`core`, `filter`, `cli`) with the framework merged from Road Builder and Track Layer: plug-in glue, geometry, offsetting, text output, document reading (closed shapes are told from open lines).
- Units (1 document point is 1 metre) and the target print scale (`true` or `1:N`).
- Versioned document storage that keeps unknown data and refuses a newer schema.
- The material library schema with twelve seed materials, all unverified.
- Command Set Settings (target print scale, style, seed, north) and the native `cartographer-cli` (`version`, `parse-settings`, `schema settings`).
- A companion panel with a settings card built from a schema, on port 8789.
- Portable Node scripts (check, build, install, package, start VectorCraft, control-port client) and CI on Ubuntu and macOS.
- Spike results for S1 to S4 in `docs/spikes/`.
