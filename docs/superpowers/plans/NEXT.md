# Next: Plan 5, styles and the target print scale

Written at the end of Plan 4 (the track port, 0.4.0). Plan 5 is the design's phase 4: the four styles (black-white, hand-drawn, high-contrast, medium-contrast) applied to every kind, and the target print scale (`1:N`) with minimum print sizes for small things.

**Scope** (spec sections 7 and 8): paint every generated object through one style layer instead of the colours the road and rail code write today; the document `style`, `seed` and `target_print_scale` settings already exist (0.1.0) and are stored and shown; a build with a scale gives small things a minimum print size and never moves a centreline or a join. The hand-drawn style needs the seed so a rebuild comes out the same.

**Facts to carry over**
- Output goes through one `Out`; colours are written by each module today. Rail output has its own decimals and thinning (`rail_builder::build_into`); a style must not change geometry for roads or rail.
- Parity gates must stay green: `node scripts/parity.mjs <cli> <road_builder_dir>` (15 scenarios) and the rail gate in `docs/PARITY-RAIL.md` (8 scenarios, scaled). A style change that alters the default (medium-contrast) output is a parity change: either keep medium-contrast identical, or update both gates deliberately.
- Fuel: the smallest road build is 1.021 times Road Builder's, rail builds 1.15 to 1.29 times Track Layer's (the terminal gallery layout is 13.6 M of about 21 M). A style layer must add little: measure with `tools/fuel` before and after.
- The panel's settings card is schema-built and read only while open; keep it that way.

**Open items carried over (decide before or during Plan 5)**
- Road meets track: a level crossing between a `road.<id>` and a `track.<id>` line. Selecting both gives a "not supported yet" note today. Own small plan.
- `set_feature`: the design folds the per-kind commands into one; Plan 4 kept Track Layer's ids (`set_tracks`, `set_yard`, `set_joint`) and Road Builder's. Decide whether to fold them before 1.0.
- Schema-built road cards: only the settings, track, yard and joint cards come from schemas; the road, junction, intersection, hub, water, embankment and crossing cards are still hand-written.
- `water.rs` decides closed shapes by coordinates, not `Item.closed`.
- The new plug-in's real point limit is not re-measured (a note appears above 1,200 points; the host refuses at about 2,000).
- Rail fuel is 15 to 30% above Track Layer's because the shared `Out` copies and checks every path; cheaper output code would help roads as well (see `docs/PARITY-RAIL.md`).
- The panel (roads and rail) has been tested against the fake VectorCraft only: `docs/PANEL-CHECK.md` is the owner's check, including the new rail steps.

**After Plan 5:** Plan 6, the legacy `.pmap` importer and exporter, `add_junction`, the migration command, release 1.0 and the deprecation of the old repos.
