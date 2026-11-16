# Next: Plan 4, Track Layer and the rail kind

Written at the end of Plan 3 (the panel port, 0.3.0). Plan 4 ports Track Layer (`CosmicCEO/vectorcraft-plugin-tracklayer`, private; there is no local copy yet: `gh repo clone CosmicCEO/vectorcraft-plugin-tracklayer`) and is where the design's "kinds" arrive.

**Scope** (spec section 10, Phase 3, and section 6): the rail kind (`rail.<id>` lines: parallel tracks, turnouts from the NMRA table, crossovers, yards, ties and the tie budget, joints and branches, connectors, the joining of lines) with Track Layer's Rust tests (about 50 in `core/tests`) ported with their numbers unchanged; rail dimensions in real metres (1:160 model millimetres become ground metres: N gauge 1.44 m) with a conversion checked once against Track Layer's tests; the kinds registry and the generic `set_feature` command now that there are two kinds; the road and rail cards built from schemas served by `cartographer-cli` (the settings card already is one); Track Layer's panel pieces; and the standards-based library.

**Decide first, with the owner:** which Track Layer commands keep their ids (`set_tracks`, `set_yard`, `set_joint`) and which fold into `set_feature`; whether the road cards become schema-built in this plan or the next.

**Facts to carry over**
- Parity method: ported tests keep their numbers; a render comparison against Track Layer's own plug-in (`scripts/parity.mjs` works for any pair of plug-ins with a scenario file) and the fuel check (`scripts/fuel-parity.mjs`) are the gates.
- The panel is ported (Plan 3): `Panel`, layers and cards need a `rail` branch, not a rewrite. Rail layers and ordering (roadbed, ties, rails; `Out::layer`) already exist in the shared `Out`.
- Host limits: a build is refused at about 2,000 input points (a note above 1,200); Track Layer's tie budget (`MAX_TIES` 9,000) is a fuel measure, re-measure with `tools/fuel`.
- Decisions from the owner: one width per river line; no simulation facts export; the panel against a real VectorCraft is the owner's check (`docs/PANEL-CHECK.md`).
- Open from earlier plans: the new plug-in's real point limit is not re-measured; `water.rs` decides closed shapes by coordinates, not `Item.closed`.

**After Plan 4:** Plan 5 styles and the target print scale; Plan 6 the legacy `.pmap` importer and exporter, `add_junction`, the migration command, release 1.0 and the deprecation of the old repos.
