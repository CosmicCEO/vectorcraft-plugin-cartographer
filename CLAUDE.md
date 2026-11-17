# CLAUDE.md: Cartographer for VectorCraft

A VectorCraft plug-in plus a companion panel that turn drawn lines and shapes and a description in words into map elements: roads, rail, streams, water, and later trails, forests, swamps and estuaries. It replaces Road Builder (`CosmicCEO/vectorcraft-road-builder`) and Track Layer (`CosmicCEO/vectorcraft-plugin-tracklayer`). The design is `docs/superpowers/specs/2026-11-16-cartographer-design.md`; the current plan is under `docs/superpowers/plans/`. Read both before changing anything.

The owner is not a developer: keep chat answers short and plain; write documents and commit messages in plain prose. Never push to `main`; work on a branch. Never force-push, merge, tag or release without being asked. Drawings are an illustrative reconstruction, not a survey.

## Layout
- `core/` shared Rust library: plug-in glue (`abi`), geometry, offsetting, output written as text, document reading, units, storage, the material library; the road network from Road Builder (`crosssection` and `library` are the band engine, `roads`, `transition`, `junction`, `crossing`, `scenarios`, `hub`, `water`, `embank`, `wet`, `standards`) and `builder` (the commands).
- `filter/` the plug-in (WebAssembly): manifest and commands. `cli/` the native helper `cartographer-cli`: checks what an agent proposes and prints schemas for the panel.
- `panel/` zero-dependency Node server and page (port 8789): `lib/panel.mjs` (the Panel: follows the selection, proposes with an agent, draws), `lib/providers.mjs` and `prompt.mjs` (agents; every answer is checked by the CLI and sent back once with the exact error), `lib/layers.mjs` (one layer per feature: spine and drawing; water under roads, crossings above), `lib/doc.mjs`, `lib/cli.mjs`, `lib/cards.mjs` (cards built from schemas: the settings card), `public/` (the page), `apple-fm/` (optional Apple Intelligence helper). `scripts/` portable Node scripts. `library/` JSON data. `tools/fuel/` measures a build's instruction cost (outside the workspace). `spikes/` throwaway experiments (outside the workspace); results are in `docs/spikes/`.

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
- Right-hand traffic: facing along the line the right lane is `fwd`; a driver's "left" is toward the centre of the road for both directions.
- Lane changes (`transition.rs`): `derive` returns the exact operation (which lane appears or vanishes) because identical lanes cannot be told apart from the text; `align` uses it; shared elements are pinned at the joints.
- A trim keeps one end of a line; a *gap* trim cuts the middle out (the second piece is named `.part2` so click mapping still works). A road that only touches another is found by extending its ends 30 m: test both a road stopping short and one stopping more than 30 m short.
- Water lies below roads, intersections and crossings above (the panel's `organise` orders layers). Ring offsets depend on winding: test both. Lines that meet end to end share one cut at the bisector of their directions.
- Parity first: ported tests keep their names and numbers; `node scripts/parity.mjs` must say all identical before a change to the road code is accepted.
- A headless session (`vectorcraft-cli mcp`) can only read and write inside its root and cannot install a heavy-budget plug-in. There is no Linux headless CLI (spike S2).
- Panel: the settings card keeps its own state (`settings`, `settingsPending`) and its own actions, apart from `pending`, and is read from a saved copy only while it is open (a plain selection must make no calls to VectorCraft). A changed document title drops every proposal. The CLI exits 1 when `ok` is false: read its JSON from stdout whatever the exit code. Document and agent text reach the page only through `textContent` or form values (no HTML sinks; `panel/test/page.test.mjs` checks). Never run the panel's tests or scripts against the owner's open document.

## Rail (the `track` kind, ported from Track Layer)
- Rail code works in model millimetres at true 1:160 inside and converts at the edge with `PT_PER_MM = 0.16` (`rail_standards.rs`). Rail output settings (4 decimals, thinning 0.00677 points) are set around rail writes only (`rail_builder::build_into`) and put back; a road written next keeps 3 decimals and 0.12.
- Output objects are stacked in three groups, roadbed then ties then rails (`output::Layer`): call `out.layer(..)` before writing. Stroke dashes are ignored, so ties are separate segments, never dashed; the tie budget (`MAX_TIES`) is a fuel measure.
- Fuel: each number written costs hundreds of instructions; `hypot` and `round` are slow. Searching is dear: score candidate branches from numbers, build points only for the winner. A build with no road must not read the road library (`Spec::without_library`). Measure with `tools/fuel`; the rail inputs are in `scripts/spikes/fuel-inputs.mjs`.
- A drawn Bezier arrives as a polyline: yards use the bisector normal at each vertex. Shapes in one object that overlap must wind the same way (`winding` in `yard.rs`). VectorCraft refuses an object with no anchor points.
- Track Layer collapsed a smooth closed ring that repeats its first point to a point (its curve thinner); `Out` here drops the repeat. The rail parity gate skips exactly those rings and says how many (`skipCollapsedRings`).
- Two standards files, one tool: `library/standards.json` (road) and `library/standards-rail.json` (NMRA; values from the NMRA PDFs only, never from memory); `node scripts/standards.mjs check`.
- Panel: a plain line is a road until the mode card says track; `panel/lib/kinds.mjs` says what names a line and its layer; cards for tracks, yards and joints come from `schema tracks|yard|joint` (`cardModel`); a road line and a track line selected together are not supported yet.

## Build, test, run
- Rust: `cargo +stable test --release --workspace`; WebAssembly: `cargo +stable build --release --target wasm32-unknown-unknown --workspace`. On macOS with Homebrew, put rustup first on `PATH` (`/opt/homebrew/opt/rustup/bin`); `scripts/build.mjs` does this.
- Node: `node --test panel/test/*.test.mjs scripts/test/*.test.mjs` (panel tests need `cargo build --release -p cartographer-cli` first).
- Never touch the owner's open VectorCraft document: test in a headless session.
