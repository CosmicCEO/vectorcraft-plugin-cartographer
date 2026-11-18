# Next: Plan 6, migration and release 1.0

Written at the end of Plan 5 (styles and the target print scale, 0.5.0). Plan 6 is the design's phase 5: a command that reads Road Builder and Track Layer documents and converts them, the legacy `.pmap` importer and exporter, `add_junction`, documentation, release 1.0, and a pointer in the two old repositories saying they are replaced.

**Facts to carry over**
- Parity gates must stay green: `node scripts/parity.mjs <cli> <road_builder_dir>` (15 scenarios), the rail gate in `docs/PARITY-RAIL.md` (8 scenarios, scaled), `node scripts/fuel-parity.mjs`. Release 1.0 also needs the render set of both old repos' README images to match in headless renders, reviewed by the owner (design, section 10).
- Road Builder documents store roads as `road.<id>` lines with plug-in data under its own plug-in id; Track Layer stores `track.<id>` lines under its id. A migration reads the old plug-in data from the saved file and writes it under Cartographer's id with `schema` first, keeping unknown keys; Track Layer's data is in model millimetres at 72/25.4 points per mm and the lines too, so a migrated document needs its track line coordinates scaled by `0.16 / (72 / 25.4)` (see `docs/PARITY-RAIL.md`).
- The panel exposes settings (style, seed, scale) through the settings card; a migrated document starts with the defaults.
- The owner's checks still open: `docs/PANEL-CHECK.md` steps that need an agent, and Linux.

**Open items carried over**
- Road meets track (a level crossing between a `road.<id>` and a `track.<id>` line): a "not supported yet" note today. Own small plan.
- `set_feature`: the design folds the per-kind commands into one; they keep their ids so far. Decide before 1.0.
- Schema-built road cards (only settings, track, yard and joint cards come from schemas).
- Hatch and stipple tones for black-white, hairlines, per-element styles (`docs/STYLES.md`); a spike on patterns first.
- `water.rs` decides closed shapes by coordinates, not `Item.closed`; the new plug-in's real point limit is not re-measured (a note above 1,200 points; the host refuses at about 2,000).
- Rail fuel is 15 to 35% above Track Layer's because the shared `Out` copies and checks every path (`docs/PARITY-RAIL.md`).
- Deferred minors of Plan 4 (see the Plan 4 pull request): Describe Track can rename `centerline`/legacy junction lines; rail notes can be cut after 30 road notes; the track spine width is 0.042 not 0.0423 in the docs; `skipCollapsedRings` counts old-side rings only.
