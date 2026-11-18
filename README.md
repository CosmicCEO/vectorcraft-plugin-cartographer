# Cartographer for VectorCraft

One VectorCraft plug-in that draws maps from lines and shapes plus a description in plain words: roads, rail, streams and water now planned, trails, forests, swamps and estuaries later. It will replace the owner's two earlier plug-ins, Road Builder and Track Layer.

**Status: 0.5.0, styles and the target print scale.** The document settings now change what a build draws: four styles (medium-contrast is the default and draws exactly what 0.4.0 did; high-contrast, black-white and a seeded hand-drawn look) and a minimum print size for road bands and strokes at a `1:N` scale ([docs/STYLES.md](docs/STYLES.md)); **Draw this** on the settings card rebuilds the document. 0.4.0 was roads and railway track in one plug-in. A line named `track.<id>` becomes N-scale model railroad track (rails, ties and roadbed): parallel tracks, yards with turnouts from the NMRA table, crossovers, crossings, branches and joins, all ported from Track Layer ([docs/RAIL.md](docs/RAIL.md), [gallery](docs/GALLERY.md), [parity with Track Layer](docs/PARITY-RAIL.md)); in the panel a plain line can be made track with **Railway track (N scale)**, and the track, yard and joint cards are built from the CLI's schemas. 0.3.0 was the road network core with its panel. Cartographer now does what Road Builder's plug-in did for roads, except the older centreline model, `.pmap` files and the Add Junction command (they come in Plan 6): a line named `road.<id>` becomes a road from a cross-section written in words or shorthand; lane changes, junctions, intersections (lighted, stops, roundabout, overpass, ramps with gores, railroad, cloverleaf), hubs of three or more roads, water (lakes, creeks and rivers that blend where lines meet, shores), embankments, bridges, culverts and causeways. The output is identical to Road Builder's: [docs/PARITY.md](docs/PARITY.md) shows how that is checked (15 scenarios, identical SVG, and a build cost within 5% of Road Builder's). The companion panel is back: select a line or a road in VectorCraft and describe it in words (an agent turns the words into a checked description), or use the cards for lane changes, junctions, intersections, hubs, water, embankments and crossings; a **Document settings** card at the bottom holds the target print scale, style, seed and north. The panel has been tested against a stand-in for VectorCraft, and the rail parts also against a real VectorCraft on macOS (`scripts/live-rail-check.mjs`): [docs/PANEL-CHECK.md](docs/PANEL-CHECK.md) is the check to run against the real application. A road meeting a track is not supported yet. Under it is the shared base: one workspace, units and the target print scale, versioned document storage, the material library schema, a native helper, portable scripts, and tests on macOS and Linux.

Drawings are an illustrative reconstruction, not a survey.

## Documents

- [Design](docs/superpowers/specs/2026-11-16-cartographer-design.md): what Cartographer is and the decisions behind it.
- Plans: [1 foundation](docs/superpowers/plans/2026-11-16-cartographer-foundation.md), [2 road core port](docs/superpowers/plans/2026-11-16-cartographer-road-core-port.md), [3 panel port](docs/superpowers/plans/2026-11-16-cartographer-panel-port.md), [4 track port](docs/superpowers/plans/2026-11-17-cartographer-track-port.md), [5 styles and print scale](docs/superpowers/plans/2026-11-18-cartographer-styles-and-scale.md) (this release) and [what comes next](docs/superpowers/plans/NEXT.md).
- [Road scenarios](docs/SCENARIOS.md), [rail scenarios](docs/RAIL.md), [gallery](docs/GALLERY.md), [styles](docs/STYLES.md), [standards](docs/STANDARDS.md) and [rail standards](docs/STANDARDS-RAIL.md), parity with [Road Builder](docs/PARITY.md) and [Track Layer](docs/PARITY-RAIL.md).
- [Getting started](docs/GETTING-STARTED.md).
- [Spike results](docs/spikes/): what survives export (S3), text objects (S4), build limits (S1), Linux (S2).

## Quick start

You need Node 22 or newer, Rust (rustup, stable, with the `wasm32-unknown-unknown` target), and VectorCraft 0.24.1 or newer.

```
node scripts/check.mjs                  # tells you what to fix
node scripts/build.mjs                  # builds the plug-in and the native helper
node scripts/install.mjs                # installs the plug-in into a running VectorCraft
node panel/server.mjs                   # the panel: http://127.0.0.1:8789
```

See [Getting started](docs/GETTING-STARTED.md) for starting VectorCraft with its control port on macOS and Linux.

## Develop

```
cargo +stable test --release --workspace
node --test panel/test/*.test.mjs scripts/test/*.test.mjs
node scripts/standards.mjs check        # the standards database
node scripts/parity.mjs <vectorcraft-cli> <road_builder_dir>   # macOS only; rail: see docs/PARITY-RAIL.md
node scripts/package.mjs 0.5.0          # a release package in dist/
```

CI runs the same on Ubuntu and macOS. Headless renders need `vectorcraft-cli`, which exists for macOS only. Read [CLAUDE.md](CLAUDE.md) before changing anything.

Minimum developer machine: a MacBook Pro with an M1 chip and 16 GB of memory; everything builds and tests there in a few minutes.
