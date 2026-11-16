# CLAUDE.md: Cartographer for VectorCraft

A VectorCraft plug-in plus a companion panel that turn drawn lines and shapes and a description in words into map elements: roads, rail, streams, water, and later trails, forests, swamps and estuaries. It replaces Road Builder (`CosmicCEO/vectorcraft-road-builder`) and Track Layer (`CosmicCEO/vectorcraft-plugin-tracklayer`). The design is `docs/superpowers/specs/2026-11-16-cartographer-design.md`; the current plan is under `docs/superpowers/plans/`. Read both before changing anything.

The owner is not a developer: keep chat answers short and plain; write documents and commit messages in plain prose. Never push to `main`; work on a branch. Never force-push, merge, tag or release without being asked. Drawings are an illustrative reconstruction, not a survey.

## Layout
- `core/` shared Rust library: plug-in glue (`abi`), geometry, offsetting, output written as text, document reading, units, storage, library, and (later) the band engine and element modules.
- `filter/` the plug-in (WebAssembly): manifest and commands. `cli/` the native helper `cartographer-cli`: checks what an agent proposes and prints schemas for the panel.
- `panel/` zero-dependency Node server and page (port 8789). `scripts/` portable Node scripts. `library/` JSON data. `tools/fuel/` measures a build's instruction cost (outside the workspace). `spikes/` throwaway experiments (outside the workspace); results are in `docs/spikes/`.

## Rules
- Edition 2024, `resolver = "3"`, the only dependency is `serde_json`. Node 22 or newer, zero dependencies. Scripts are Node, never shell: no `sed -i`, no `open -a` outside one OS branch.
- 1 document point is 1 metre of ground. User lines are named `<kind>.<id>`; generated shapes `gen.<kind>.<id>.<piece>.<role>`. The plug-in id is `com.cosmicceo.cartographer.build`. Document data is one versioned record with `schema` first; a newer schema is refused, never overwritten.
- A centreline or a join never moves because something was widened for legibility.
- Standards values: `verified` only if read in a primary source; anything else is `unverified`. Never say a drawing is compliant with a standard unless the value is verified and cited. Do not spoof a browser to get past a 403.

## Traps (inherited, do not repeat)
- A filter sees only visible, unlocked selected objects. The panel shows and unlocks its layers for a build and restores them. A filter cannot create layers or lock its output. New objects stack below kept objects.
- A filter cannot create text objects (spike S4); only paths and compound paths.
- An object with no anchor points is refused: never write an empty one. Stroke dashes in filter output are ignored. Fills are `{"type":"solid","color":...}`; strokes have `paint` and `width`. A command parameter cannot be called `id`.
- Fuel: the host refuses a build at about 21 M plus a share that grows with the input (50 M instructions plus 4,000 per input byte). `f64::hypot` and `f64::round` are slow in WebAssembly: use `Len2::len2` and a truncating cast. Every number written costs hundreds. Measure with `tools/fuel`; never guess (`docs/spikes/S1-fuel.md`).
- A drawn Bezier arrives flattened (16 points per segment): offset with the bisector normal at each vertex and refit Beziers on output. Shapes in one object that overlap must wind the same way.
- Right-hand traffic: facing along the line the right lane is `fwd`.
- A headless session (`vectorcraft-cli mcp`) can only read and write inside its root and cannot install a heavy-budget plug-in. There is no Linux headless CLI (spike S2).

## Build, test, run
- Rust: `cargo +stable test --release --workspace`; WebAssembly: `cargo +stable build --release --target wasm32-unknown-unknown --workspace`. On macOS with Homebrew, put rustup first on `PATH` (`/opt/homebrew/opt/rustup/bin`); `scripts/build.mjs` does this.
- Node: `node --test panel/test/*.test.mjs scripts/test/*.test.mjs` (panel tests need `cargo build --release -p cartographer-cli` first).
- Never touch the owner's open VectorCraft document: test in a headless session.
