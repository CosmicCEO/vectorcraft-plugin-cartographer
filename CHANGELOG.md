# Changelog

## 0.2.0 Road network core
- Road Builder's road network ported: cross-sections and the element library, roads, lane changes, junctions, intersections and scenarios (stops, signal, roundabout, overpass, ramps and gores, railroad, cloverleaf), hubs, water (lakes, creeks and rivers that conform where lines meet, shores), embankments, bridges, culverts and causeways. 96 Road Builder tests ported with their numbers unchanged.
- The standards database (59 entries) and its checker.
- Commands: Describe Road, Describe Junction, Make Intersection, Make Intersection of Three or More Roads, Add or Drop a Lane, Describe Water, Put Road on Fill, Road Meets Water; `cartographer-cli` gains parse, edit, diff and the parse-* checks.
- Every command and Build refuse a document saved by a newer Cartographer; a build of more than 1,200 points gets a note.
- Parity gate: identical SVG for 15 scenarios and build cost equal or lower than Road Builder's (`docs/PARITY.md`).

## 0.1.0 Foundation
- One Cargo workspace (`core`, `filter`, `cli`) with the framework merged from Road Builder and Track Layer: plug-in glue, geometry, offsetting, text output, document reading (closed shapes are told from open lines).
- Units (1 document point is 1 metre) and the target print scale (`true` or `1:N`).
- Versioned document storage that keeps unknown data and refuses a newer schema.
- The material library schema with twelve seed materials, all unverified.
- Command Set Settings (target print scale, style, seed, north) and the native `cartographer-cli` (`version`, `parse-settings`, `schema settings`).
- A companion panel with a settings card built from a schema, on port 8789.
- Portable Node scripts (check, build, install, package, start VectorCraft, control-port client) and CI on Ubuntu and macOS.
- Spike results for S1 to S4 in `docs/spikes/`.
