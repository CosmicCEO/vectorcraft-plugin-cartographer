# Changelog

## Next
- **Shapes everywhere:** road, track and water each pair with themselves and each other drawn with any shape tool (pen, line, rectangle, rounded rectangle, ellipse, polygon, star, spiral, arc): 486 combinations checked by `scripts/shape-matrix.mjs`. A track drawn as a closed shape no longer takes its start corner for a line end (a crossing there was refused as a branch).
- **Every tie, in any yard:** a route's ties are one line that the panel dashes into ties after each build, so big yards no longer fall back to solid strips (owner, 2026-11-20). Rail builds cost a quarter to 0.85 of Track Layer's (were 1.27 to 1.64). Built without the panel (directly in VectorCraft), ties show as a solid strip until the next panel build.
- Lines drawn with any drawing tool count as lines in the panel (a rounded rectangle did not); only layers, groups, text, images and compound paths do not (owner).
- The panel shows the plug-in's name and release at the top (owner).

## 1.7.0
- **Yard ladders:** every turnout in a ladder now stands on ties. The ladder track has its own ties from where it leaves the main, each turnout on it has its lengthened switch ties, and each spur's own ties begin where it clears the ladder (no doubled ties). Before, the ladder ran bare after its first turnout.
- **Yards:** `before_end` stops a yard's ladders that many metres before the end of its line (with `at`, metres from the start); card field "Stops before end (m)".
- **Bridges:** a road bridge's deck follows a hand-drawn road; a rail bridge over a yard with spurs on one side keeps each side wall beside its own side.
- **Panel and setup:** the start script says VectorCraft's control port is not the panel's and names the next step; a plain line or shape selected with a track says to make it a road first.
- Lengthened switch ties no longer cross a level-crossing gap and are not drawn over a solid tie band.

## 1.6.0
- **Railway in metres:** track and yard descriptions, cards, stored records and messages are in metres of ground (spacing 3.96 m, not 24.765 mm); "N scale" is gone from the panel and the guides. Documents saved by 1.0 to 1.5 are converted when they are next read and draw the same. Text such as "4000mm", "100cm" or "36in" means real lengths. 1.5 cannot change a document saved by 1.6 (it says so).
- **Road meets track:** two bridges join the level crossings on the same card: **road over** (the road's surface laid over the track, parapets along it) and **track over** (the track runs on over the road, parapets beside it). Both follow a curving track. Gates now stand on the approaching driver's right. A build note says when a track crosses a road with no crossing stored.
- **Panel:** lines made with the shape tools (line, ellipse, rectangle, polygon) work like pen lines; a road drawn partly with the shape tool keeps its pieces in order.
- **Setup:** `node scripts/build.mjs` in a release package says there is nothing to build; `start-vectorcraft.mjs` waits until VectorCraft answers, says at once when it cannot start, and on a Mac when VectorCraft is running without its control port; the panel says plainly when its port is in use.

## 1.5.0
- **Road meets track:** select a road line and a track line that cross (or use Make Level Crossing) to make a level crossing, plain or gated. The track's ties stop under the road, a crossing panel covers the ballast, the rails run across; gated adds crossbucks and gate arms. Roads drawn from one edge and roads drawn in pieces are handled.
- **Small scales:** creeks and rivers get the band print minimum (bridges span the creek as drawn); crosswalk bars, arrows, culvert pipes, level-crossing ties and gore chevrons keep their true width instead of printing solid; in the hand-drawn style one marking keeps one width along every piece of a road.
- **Cost:** rail builds are cheaper (1.20 to 1.23 times Track Layer, from 1.25 to 1.31). The real point limit was measured (a plain two-lane road builds to about 19,450 points); the note now starts at 12,000 points instead of 1,200.
- **Fixes:** Describe Track no longer renames centreline, profile, flag, arrow or legacy junction lines; track notes are not crowded out by road notes; Draw says "Drawn."/"Rebuilt." after it finishes; clearing a settings number keeps the stored value; settings saved but not rebuilt say so; a second copy of a document is a different document in the panel.
- **Migration:** numbers it does not touch come back exactly as written; collisions and both-records cases are reported; removed generated objects are named; compound track lines are scaled; the output is written safely (temporary file and rename; dangling links and missing folders refused plainly).
- **Release:** v1.0.0 is marked "do not use"; use 1.0.1 or later.
- Not in 1.5 (by ruling, see `docs/RELEASE-1.5.md`): black-white hatching, schema-built road cards, junction radii that grow with print scale, symbol minimum sizes, an overpass of a road over a track.

## 1.0.1 Package fix
- The release package works: `scripts/install.mjs` and the panel looked for the plug-in and the native helper under `target/`, which only exists in a source checkout, so a downloaded package could not install or start the panel without hand-setting `CARTOGRAPHER_CLI`. They now also look where the package keeps them (`cartographer_filter.wasm`, `bin/cartographer-cli`).
- `node scripts/check.mjs` in a package no longer asks for Rust (not needed to install and run a package).
- `scripts/package.mjs` now checks that what it packed can be found by those lookups, so this cannot ship broken again.

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
