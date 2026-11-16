# Cartographer for VectorCraft: design

Status: draft for the owner's approval. Nothing has been built. Date: 2026-11-16.

## 1. Intent and why

One VectorCraft plug-in, **Cartographer**, draws maps from lines and shapes plus a description in plain words. It replaces two plug-ins the owner already has:

- **Road Builder** (`CosmicCEO/vectorcraft-road-builder`, v0.7.0): roads, intersections, hubs, lane changes, gores, water, embankments, bridges and culverts, a standards database, a companion panel.
- **Track Layer** (`CosmicCEO/vectorcraft-plugin-tracklayer`, private): N-scale model railroad track, turnouts, crossings, yards, joined lines.

Today the two repos hold two copies of the same plumbing (plug-in glue, geometry, offsetting, output, document reading, layers, the panel server, agent providers, the standards checker). A third element type would mean a third copy. Cartographer keeps one copy, and a new kind of map element (trails, forests, swamps, estuaries, anything later) is added as data plus, only when its geometry needs it, one small module.

The finished drawings are printed as vector art up to A0 (841 x 1189 mm), and printed at N scale (1:160) to lay out scale models. The document may later drive an animation along its vectors, objects and paths, so what the plug-in writes must stay identifiable and machine-readable after Save, Save As, Export or print.

## 2. Goals, non-goals, anti-goals

Goals
- One install, one panel, one document format for roads, rail, streams, water bodies and shores.
- A way to add map elements in future without touching shared code.
- Output that prints cleanly from A0 maps down to N-scale layouts: smooth Bezier curves, honest line widths.
- Four drawing techniques from day one in the design (section 7).
- Works the same on macOS and Linux, and is easy to install on both.
- Drawings stay "illustrative reconstruction, not a survey" unless a value is verified (section 9).

Non-goals (for the first release)
- Text labels, scale bar, legend, north arrow, grid, title block (section 11; the data is reserved now).
- Automatic detection and suggestion of crossings (a later release; manual in the first).
- Mixed scales inside one document, per-element styles, per-layer styles.
- Any traffic simulation or animation feature. Only the data survival that a future animation needs.
- Forests, swamps, trails, estuaries as shipped elements (the engine must be ready for them; they come after parity).

Anti-goals (would make it a failure even if it works)
- A centreline or join that moves because a line was made wider for legibility.
- An agent answer that can break a print or a build (every answer is validated by the native CLI before anything is drawn).
- A third copy of the framework, or a macOS-only script in the install path.
- Telling anyone a drawing is "compliant" with a standard whose value was not read and cited.

## 3. What the owner decided (interview, 10 questions)

1. Files are handled by VectorCraft Save, Save As, Export or print; the plug-in's data must be kept for later animation along vectors, objects and paths. Output is printed up to A0 and at N scale.
2. One map scale per document. The owner chooses the print scaling when printing.
3. A document setting **target print scale** (or "true"). With a scale set, the plug-in enforces minimum print widths and symbol sizes for that scale; the user rebuilds after changing it. Centrelines and joins always stay true.
4. New elements are started by words: the user draws a plain line or shape with VectorCraft's pen, then describes it in the panel. The agent works out the kind, and the card shows what it understood and can be corrected. The agent should "intuit and introduce desired effects" (a shore with a cement embankment draws differently from a shore with a 30 m flat sandy beach and 20 m of dunes).
5. Unknown words: first use the closest existing material and say so; if the user wants a new look, the agent proposes a new material from a small set of paint pieces with a preview, saved to the document library only on Accept. The up-front library is built from public mapping standards.
6. One drawing style per document, switchable for the whole document later with one command. Styles: precise black and white, hand-drawn, high-contrast colour, medium-contrast colour with hard edges.
7. Crossings: on request only for the first release (select both, choose a treatment); detect-and-suggest in a later release.
8. Scope of the first release: full port. Cartographer ships when it matches Road Builder and Track Layer; then the two are deprecated.
9. The panel stays on zero-dependency Node; the CLI stays native Rust.
10. No labels or map furniture in the first release, but the data (a name field, a reserved text layer) is in from day one, mindful of a later release with labels, scale bar, legend, north arrow, grid and title block.

## 4. Constraints and lessons from the two repos

Host (VectorCraft 0.24.1) and plug-in rules, learned the hard way:
- A plug-in is a WebAssembly filter. It sees only visible, unlocked selected objects, so the panel shows and unlocks its own layers for a build and restores them. A filter cannot create layers or lock its output.
- New objects are listed first and stack below kept objects; objects of a build are grouped by an explicit layer order, so one element's fill never hides another's ties or lines.
- The host stops a build at about 21 M fuel (about one per instruction) plus a budget per input byte. Measure with `tools/fuel`; never guess. Known costs: `f64::hypot` and `f64::round` are slow software routines in WebAssembly, every number written costs hundreds, a Track Layer tie costs about 1,150. Branch planning must score candidates from numbers and build points only for the winner.
- An object whose path has no anchor points is refused; never write one that may be empty. Stroke dashes in filter output are ignored. Fills are `{"type":"solid","color":...}`; strokes have `paint` and `width`. A command parameter cannot be called `id`.
- A drawn Bezier arrives flattened to a polyline (16 points per segment); offset with the bisector normal at each vertex and refit with Bezier handles on output. Shapes in one object that overlap must wind the same way.
- Right-hand traffic: facing along the line, the right lane is `fwd`.

Process lessons:
- Standards values are marked `verified` only if read in a federal or primary source, `verified-secondary` if read in a state manual that applies it, `unverified` otherwise. FHWA sites refuse automated downloads; do not spoof a browser to get past that.
- Hand-worked numbers in tests are the best regression net; keep them.
- Headless renders (`vectorcraft-cli mcp`, export at scale 10 and crop) catch what unit tests miss.
- The owner is not a developer: chat stays short and plain; documents are plain prose.

Install facts: VectorCraft 0.24.1 ships Linux builds (AppImage, deb, rpm, flatpak, and an aarch64 set). The WebAssembly plug-in and a Node panel are portable. The existing scripts are not (`sed -i ''`, `open -a VectorCraft`, the Homebrew rustup path, a macOS-only `vectorcraft-cli` location). Whether a Linux `vectorcraft-cli` exists is not yet known (spike S2).

## 5. Approaches considered

A. **One engine driven by data, with code only for specialties (chosen).** Linear elements are ordered bands measured outward from the line (`water | seawall 3 | pavement 4`, `ballast | track x2 | ballast`). Areas are a fill material plus an edge band plus optional scatter. Materials, styles and element definitions are JSON in `library/`. Rust modules exist only where geometry needs real code (intersections, hubs, turnouts, yards, wet crossings, joins). One plug-in command (`set_feature`) and one panel card builder read each element's schema. Adding a kind is data, and code only if it has special geometry.

B. One hand-written module and panel card per element, as in both repos. Simple at first, but each new element means Rust, panel card, agent prompt and tests again, and the two copies of the framework drift.

C. A family of plug-ins sharing a crate (one per element). Smaller builds, but plug-ins cannot see each other's output, so a road crossing a stream, or rail crossing a road, cannot be built; the user installs many things.

Chosen: A. It is the only one where roads, rail and water can cross, joins can conform across kinds, and a new element costs data, not plumbing.

## 6. Architecture

One Cargo workspace (edition 2024, `resolver = "3"`, the same release profile as Track Layer):

```
core/        shared: abi, geom, offset, output (text + layers), vcdoc, units, storage (versioned document data), joins
library/     JSON data: materials.json, styles/*.json, elements/*.json, standards.json
elements/    code only for specialties: road, rail, water, crossing, intersection, hub, turnout, yard
filter/      the plug-in: manifest, commands (set_feature, set_style, set_scale, build, ...)
importer/    legacy .pmap importer and the migration reader (section 12)
exporter/    .pmap exporter (kept until the owner says it is not needed)
cli/         cartographer-cli: parse and validate, schema, library, version, migrate
panel/       zero-dependency Node server and page; cards generated from element schemas
scripts/     portable Node scripts: build, install, package, render, standards, vc
tools/fuel/  measures a build's instruction cost (excluded from the workspace)
docs/
```

### 6.1 Names and stored data

Lines and shapes the user draws are named `<kind>.<id>`: `road.a`, `rail.t1`, `stream.c`, `water.l`, later `trail.x`, `forest.f`. Generated shapes are named `gen.<kind>.<id>.<piece>.<role>` and are stable across rebuilds, so a click maps back to the line it came from and an animation can find them. The document's plug-in data holds one versioned record (`schema` number first), with features by id, the document settings (target print scale, style, north, seed), the document library (materials the user accepted), joins and crossings. Names carry enough to rebuild the feature list if plug-in data is lost in an export. Spike S3 (2026-11-16, `docs/spikes/S3-survival.md`) found that Save, Save As, reopen and PDF keep the plug-in data, while SVG keeps names, shapes and direction but drops the plug-in data.

The drawn line itself (the spine) is never altered by a build. Its direction is the feature's direction (traffic or flow with the line), which is what lets a later animation travel along it.

### 6.2 Bands and materials (all linear kinds)

The cross-section shorthand of Road Builder generalises: items separated by `|`, each a material with optional direction, width and repeat. A material in `library/materials.json` has:

- `id`, `role` (surface, edge, water, bank, vegetation, structure, mark), `default_width`, `min_print_width`,
- `semantic` fields that are style independent (it is water, it is paved, it is traversable),
- `height` (for banks, dunes, embankments: feeds slopes),
- `class` (a key the style sheets paint: for example `water.shallow`, `earth.sand`, `paved.asphalt`),
- `source` and a verified status, as in the standards database.

Examples the library must cover on day one (from the owner): seawall, pavement, sand, dune, grass, ballast, track, bank, riprap, mud, marsh, forest floor. The first library is built from public standards (section 9).

### 6.3 Areas

A shape is a fill material, an optional edge band (inward or outward), and optional scatter of symbols (trees, reeds) with a spacing rule and a hard cap on count (fuel). Scatter uses the document seed, so rebuilds are identical. Forests, swamps and estuaries are definitions in `library/elements/`, not code.

### 6.4 Styles (technique is separate from meaning)

A style sheet maps each material `class` to paint: fill, stroke, width rule, pattern, edge treatment. First four styles: precise black and white (tone from hatch and stipple patterns instead of colour, hairlines), hand-drawn (edges wobble by a deterministic noise seeded from the document, variable stroke width), high-contrast colour, medium-contrast colour with hard edges (flat fills, strong outlines, no gradients). One style per document; `set_style` rebuilds everything. Rules that hold in every style: the spine and centreline are never wobbled; hand-drawn jitter has a bounded amplitude so a join never opens; the style cannot change geometry that other features depend on. An optional per-element `style` field is reserved in the data and ignored until a later release.

### 6.5 Units, target print scale, legibility

1 document point is 1 metre of ground (the Road Builder convention). Track Layer's millimetre-at-1:160 convention is dropped: rail gauge and N-scale dimensions are written in real metres (N gauge 9 mm at 1:160 is 1.44 m), and the print scaling of 1:160 is done by VectorCraft. The document setting `target_print_scale` is `true` or `1:N`. With a scale set, every band, mark and symbol gets a minimum print size: widening is applied outward from a band's own side (a road widens equally about its centreline; a stream equally about its line), never moves a spine, and a join never moves. At a given `1:N`, 1 mm on paper is N mm of ground, so the minimum width in metres is `min_print_width * N / 1000`. Changing the setting needs a rebuild.

### 6.6 Joins and crossings

One joins module for all linear kinds, generalising the water joins: lines that meet end to end (within 2 m) conform (width and bands average and ease over half the line or 40 m, whichever is less) on a shared cut at the bisector of their directions; a line ending in the middle of another is a tributary or branch and joins it. A table of (kind pair, treatment) lists what crossing a kind pair may use: road over stream (bridge, culvert, ford, causeway), rail over road (level crossing, overpass), trail over stream (ford, footbridge), and so on. The first release draws a crossing only when the user selects both lines and chooses; the detect-and-suggest release reads the same table.

### 6.7 The agent (words in, validated data out)

The agent answers with JSON built only from the element's schema and the library. The native CLI validates it (`parse` plus `schema`); an invalid answer is sent back once with the exact error, then falls to the fallback agent. Unknown words follow the owner's rule: the closest existing material with a note saying what was substituted; a request for a new look yields a **proposed material** made only from the fixed paint pieces (flat colour, hatch, stripes, dots, a repeated symbol), validated, previewed, and saved to the document library only on Accept. The agent never emits geometry or paths.

### 6.8 The panel

The same zero-dependency Node server as today (127.0.0.1, one port, Host and Origin checks, SSE). A card is generated from an element schema served by the CLI: fields, units, enums, help. Selection logic (a plain line, a road, two roads, a road and water) lives in one `classify`/`target` module. The panel is the only place that creates layers (one layer per feature, spine and drawing, water under roads, crossings above). Agent providers as today (Claude, an OpenAI-style endpoint, a command, Apple Intelligence on macOS only, filtered by availability).

## 7. Output quality for print

- Every curve is written as Bezier segments (fitted with handles), not a dense polyline; a flattened input is smoothed only where it was a curve.
- Compound paths wind consistently; no empty objects; no zero-width strokes.
- A0 and N-scale both stay within the host's fuel limit: each element kind has a measured fuel cost per metre and per feature in `tools/fuel`, checked in CI against a ceiling.
- Fonts and text are out of scope until the labels release; the scale bar and legend will use that spike's result.

## 8. macOS and Linux

- The plug-in is one WebAssembly file; the same bytes run on both.
- `scripts/` are Node (22 or newer), not shell: no `sed -i`, no `open -a`, no Homebrew paths. `scripts/vc.mjs` finds the VectorCraft control port and the application on each OS (macOS `open -a`, Linux AppImage, deb, rpm, flatpak) and prints one plain message if it cannot.
- An install check (`node scripts/check.mjs`) reports Node version, Rust toolchain (rustup with `wasm32-unknown-unknown`), `vectorcraft-cli`, and the plug-in directory for the OS, with one fix per problem.
- CI runs on `ubuntu-latest` and `macos-latest`: Rust tests, wasm build, panel tests, the standards check, and a headless render comparison where a headless VectorCraft is available (spike S2).
- File names are treated as case sensitive and paths with spaces are tested. `.DS_Store` and other OS files are ignored.
- Release packaging builds one zip per OS family (plug-in wasm, CLI binary for that OS and CPU, panel, library, docs) with `SHA256SUMS`.

## 9. The up-front standards library

Built before the port is finished, from public sources, in the same database format as Road Builder (`library/standards.json`: entry, source, section, quote, status). Candidates (not yet read; each gets a verified status only when read in the text):

- USGS topographic map symbols (public domain): water, swamp, shore, woods, trails, rail, roads.
- FGDC geologic map symbolization standard: pattern conventions.
- NOAA nautical chart symbols (Chart No. 1): shores, tidal flats, structures.
- USGS National Hydrography Dataset feature codes: naming and classes of streams and water bodies.
- MUTCD 11th edition for roads and crossings (already read in Road Builder), PROWAG, state manuals.
- NMRA standards and recommended practices for rail (already read in Track Layer).
- ISO and IHO portrayal standards: paywalled; listed as unverified references only.

Copyright rule: symbols and numbers may be used where public; paywalled text is cited, not copied. Defaults without a source (for example embankment slope, bridge clearance, culvert size) stay `unverified`.

## 10. Phases and the parity gate

Phase 0, spikes (each answers one question before it can hurt a design):
- S1 fuel of the generic engine against a Road Builder two-lane road and a Track Layer yard.
- S2 does `vectorcraft-cli` exist for Linux, and can renders run headless in CI.
- S3 what survives Save, Save As, SVG, PDF and print: plug-in data, object names, spine direction.
- S4 can a filter output real text objects, or only paths (decides the labels release). Result (2026-11-16, `docs/spikes/S4-text.md`): only paths. A filter cannot create text, so the labels release uses either outlined glyph paths from a bundled font or real text objects created by the panel through VectorCraft's own commands.

Phase 1: workspace, `core` (moved from the two repos, de-duplicated), versioned storage, library schema, portable scripts, CI on both OSs.
Phase 2: port Road Builder (roads, lane changes, junction, intersections, scenarios, hub, gore, water, embankment, wet crossings) onto the band engine; port its Rust and panel tests with their numbers unchanged.
Phase 3: port Track Layer (tracks, turnouts, yards, ties, joints, branches, connectors) the same way. N gauge becomes real metres; the tests' expected geometry is converted and the conversion is checked once.
Phase 4: styles (four) and target print scale; the up-front library; the agent's material proposals.
Phase 5: migration command (reads Road Builder and Track Layer documents and converts them), documentation, release 1.0, then deprecate both old repos with a pointer.

**Parity gate:** release 1.0 requires every old Rust and panel test to pass in Cartographer, and a render set of both old repos' README images to match in headless renders (side by side, reviewed by the owner).

Later releases: detect-and-suggest crossings; labels, scale bar, legend, north arrow, then grid and title block; trails, forests, swamps, estuaries; per-element styles; mixed-scale insets.

## 11. Reserved now, built later

- `name` on every feature; a `text` layer id in the layer plan; a `north` setting; a sheet record (size, margins, title) in the document data.
- The crossing table (section 6.6) so suggestions are a read, not a redesign.
- The per-element `style` field (section 6.4).
- A `schema` version and a migration function from day one.

## 12. Migration of existing documents

The old plug-ins store their data under their own plug-in ids with their own names (`road.*`, `water.*`, `crossings`, `hubs`, `waters`, `wets`, `embankments`; Track Layer's tracks, yards, joints). The `migrate` command reads those records and writes Cartographer's, keeping object names so layers and spines carry over. It is run once, on a copy, and reports what it could not convert. Track Layer's millimetre-at-1:160 documents are converted to real metres (multiply by 160 / 1000 in metres).

## 13. Risks

- Port size: two repos of geometry onto a new engine. Mitigation: port tests first, the parity gate, and keep the specialties as code, not forced into bands.
- Fuel: a generic engine may cost more than hand-tuned code. Mitigation: spike S1 first; a per-kind fuel ceiling in CI.
- Hand-drawn style cost and joins opening under jitter. Mitigation: bounded noise, never on spines, a join test per style.
- Print-scale widening colliding with neighbouring bands. Mitigation: widen about the band's own centre, check overlaps in tests, and report a note when a minimum width cannot fit.
- Linux unknowns (S2). Mitigation: plug-in and panel are portable; headless CI is added when the CLI is confirmed.
- Agent inventing a look that does not print. Mitigation: only the fixed paint pieces, validated, preview, Accept.

## 14. Left to the builder

- The exact grammar extensions for areas and scatter.
- The exact JSON of materials, styles and element definitions (start from Road Builder's `library/elements.json`).
- Which Road Builder and Track Layer commands keep their names and which fold into `set_feature`.
- The structure of the migration reader.

## 15. Calls made without the owner (check these)

- One document point is one metre (Road Builder's convention) and Track Layer's model-millimetre convention is retired.
- The kind prefixes in names (`road`, `rail`, `stream`, `water`, `trail`, `forest`) and keeping the `gen.<kind>.<id>...` pattern.
- Edition 2024 workspace, Node 22 or newer, release profile as Track Layer.
- Four styles named: black-and-white, hand-drawn, high-contrast, medium-contrast.
- Hand-drawn jitter seeded from a document seed.
- Spikes S1 to S4 before Phase 1.
- Keep the `.pmap` exporter until the owner says it is not needed.
