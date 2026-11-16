# Cartographer for VectorCraft

One VectorCraft plug-in that draws maps from lines and shapes plus a description in plain words: roads, rail, streams and water now planned, trails, forests, swamps and estuaries later. It will replace the owner's two earlier plug-ins, Road Builder and Track Layer.

**Status: foundation only (0.1.0). It is not yet usable for maps.** What exists is the shared base the map elements will be built on: one workspace, the merged framework (geometry, offsetting, output, document reading), units and the target print scale, versioned document storage, the material library schema, one working command (Set Settings), a native helper, a companion panel with a settings card built from a schema, portable scripts, and tests on macOS and Linux.

Drawings are an illustrative reconstruction, not a survey.

## Documents

- [Design](docs/superpowers/specs/2026-11-16-cartographer-design.md): what Cartographer is and the decisions behind it.
- [Plan 1: foundation](docs/superpowers/plans/2026-11-16-cartographer-foundation.md) (this release) and [what comes next](docs/superpowers/plans/NEXT.md).
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
node scripts/package.mjs 0.1.0          # a release package in dist/
```

CI runs the same on Ubuntu and macOS. Headless renders need `vectorcraft-cli`, which exists for macOS only. Read [CLAUDE.md](CLAUDE.md) before changing anything.

Minimum developer machine: a MacBook Pro with an M1 chip and 16 GB of memory; everything builds and tests there in a few minutes.
