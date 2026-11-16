# Next: Plan 3, the panel port (and what follows)

Written at the end of Plan 2 (the road core port, 0.2.0). Plan 3 ports Road Builder's companion panel so the road commands have their cards again.

**Scope of Plan 3** (spec section 10, Phase 2, second half): `classify` and the shape-to-line mapping for roads, junctions, crossings, hubs, water and wet crossings; layers (one per feature, water under roads, crossings above, the panel shows and unlocks its layers for a build); the road card with the element bar and edits, the lane-change card, and the junction, intersection, hub, water, embankment and crossing cards, built from schemas served by `cartographer-cli` as the settings card already is; the agent providers (Claude, a command, an OpenAI-style endpoint, Apple Intelligence on macOS only), prompts, `changes`; and the Road Builder panel tests (`providers.test.mjs` 17, `panel.test.mjs` 40, `server.test.mjs` 5) with their numbers unchanged.

**Facts to carry over**
- The panel talks to the plug-in only through command ids that already exist (`set_cross_section`, `set_junction`, `set_crossing`, `set_hub`, `set_transition`, `set_water`, `set_embankment`, `set_wet`, `set_settings`) and the CLI's `parse`, `edit`, `diff` and `parse-*` commands; the fake VectorCraft in `panel/test` needs handlers for each.
- Decisions from the owner: one width per river line (joined lines conform); no simulation facts export.
- Spike results: filters cannot create text; one build is refused at about 2,000 points (a note appears above 1,200); SVG export drops plug-in data (PDF and the native file keep it); no Linux headless CLI.
- Plan 2 left these open: the new plug-in is cheaper than Road Builder (`docs/PARITY.md`), so its real point limit should be re-measured with `scripts/spikes/s1-limit.mjs` (that script still names Road Builder's plug-in), and `water.rs` still decides closed shapes by coordinates, not by `Item.closed`; adopt the flag when a test for a loop road is added.

**After Plan 3:** Plan 4 Track Layer and the rail kind (the registry and `set_feature` arrive here); Plan 5 styles and the target print scale; Plan 6 the legacy `.pmap` importer and exporter, `add_junction`, the migration command, release 1.0 and the deprecation of the old repos.
