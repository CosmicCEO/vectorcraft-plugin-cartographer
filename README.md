# Cartographer for VectorCraft

One VectorCraft plug-in that draws roads, railway track and water from lines and shapes plus a description in plain words. It replaces the owner's two earlier plug-ins, Road Builder and Track Layer. Trails, forests, swamps and estuaries are planned for later.

**Status: 1.5.0.** Since 1.0: a road can cross a track at a **level crossing** (plain or gated), print minimums for creeks and rivers, crosswalks and arrows that stay readable at small scales, cheaper rail builds, the real point limit measured (about 19,000 points for a plain road), and a sweep of small fixes ([CHANGELOG](CHANGELOG.md), [release notes](docs/RELEASE-1.5.md)).

- **Roads** (a line named `road.<id>`): a cross-section written in words or shorthand; lane changes, junctions, intersections (lighted, stops, roundabout, overpass, ramps with gores, railroad, cloverleaf), hubs of three or more roads, water (lakes, creeks and rivers that blend where lines meet, shores), embankments, bridges, culverts and causeways. The output is identical to Road Builder's ([docs/PARITY.md](docs/PARITY.md): 15 scenarios, identical SVG, build cost within 5%).
- **Railway track** (a line named `track.<id>`): rails, ties and roadbed to NMRA proportions, described in metres; parallel tracks, yards with NMRA turnouts, crossovers, crossings, branches and joins, ported from Track Layer ([docs/RAIL.md](docs/RAIL.md), [gallery](docs/GALLERY.md), [parity](docs/PARITY-RAIL.md): 8 scenarios identical when scaled).
- **Styles and print scale**: four styles (medium-contrast is the default; high-contrast, black-white, a seeded hand-drawn look) and a minimum print size for road bands and strokes at a `1:N` scale ([docs/STYLES.md](docs/STYLES.md)).
- **Migration**: `node scripts/migrate.mjs old.vectorcraft new.vectorcraft` converts a document made with Road Builder or Track Layer ([docs/MIGRATION.md](docs/MIGRATION.md), live-checked against both real plug-ins).
- **The companion panel**: select a line in VectorCraft and describe it in words (an agent turns the words into a checked description), or use the cards; a **Document settings** card holds the target print scale, style, seed and north. It was tested against a stand-in for VectorCraft and, for rail, styles and migration, against a real VectorCraft on macOS; [docs/PANEL-CHECK.md](docs/PANEL-CHECK.md) is the check for the real application.

Not in 1.5: an overpass of a road over a track, the older centreline model, `.pmap` files and Add Junction (they stay in Road Builder; see [docs/MIGRATION.md](docs/MIGRATION.md)), hatch and stipple tones (see [spike S6](docs/spikes/S6-patterns.md)), labels. [docs/RELEASE-1.0.md](docs/RELEASE-1.0.md) is the release record.

Under it is the shared base: one workspace, units and the target print scale, versioned document storage, the material library schema, a native helper, portable scripts, and tests on macOS and Linux.

Drawings are an illustrative reconstruction, not a survey.

## Documents

- [Design](docs/superpowers/specs/2026-11-16-cartographer-design.md): what Cartographer is and the decisions behind it.
- Plans: [1 foundation](docs/superpowers/plans/2026-11-16-cartographer-foundation.md), [2 road core port](docs/superpowers/plans/2026-11-16-cartographer-road-core-port.md), [3 panel port](docs/superpowers/plans/2026-11-16-cartographer-panel-port.md), [4 track port](docs/superpowers/plans/2026-11-17-cartographer-track-port.md), [5 styles and print scale](docs/superpowers/plans/2026-11-18-cartographer-styles-and-scale.md), [6 migration](docs/superpowers/plans/2026-11-18-cartographer-migration.md) (this release) and [what comes next](docs/superpowers/plans/NEXT.md).
- [Road scenarios](docs/SCENARIOS.md), [rail scenarios](docs/RAIL.md), [gallery](docs/GALLERY.md), [styles](docs/STYLES.md), [migration](docs/MIGRATION.md), [release 1.0](docs/RELEASE-1.0.md), [standards](docs/STANDARDS.md) and [rail standards](docs/STANDARDS-RAIL.md), parity with [Road Builder](docs/PARITY.md) and [Track Layer](docs/PARITY-RAIL.md).
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
node scripts/package.mjs 1.5.0          # a release package in dist/
```

CI runs the same on Ubuntu and macOS. Headless renders need `vectorcraft-cli`, which exists for macOS only. Read [CLAUDE.md](CLAUDE.md) before changing anything.

Minimum developer machine: a MacBook Pro with an M1 chip and 16 GB of memory; everything builds and tests there in a few minutes.
