# Cartographer 1.5: release notes

1.5 was planned and run by the agent without the owner, under the rules in `docs/superpowers/plans/2026-11-19-cartographer-1.5.md`. Every package went through tests written first, the road and rail parity gates, the fuel gate, CI on macOS and Linux, a fresh review (Opus) with one fix pass, and, where it changes the drawing or the panel, a check on the real VectorCraft. What changed is in `CHANGELOG.md`; this page lists every decision taken on the owner's behalf and everything left for later.

## Work packages

| Package | Result |
|---|---|
| WP1 release hygiene | v1.0.0 marked "do not use". The old repositories were already archived, so their banners could not be added (owner: unarchive, add, re-archive if wanted). |
| WP2 correctness sweep | done (#13) |
| WP3 small-scale drawing | done (#14); junction radii scaling and symbol minimums cut |
| WP4 road meets track | done (#16): level crossings |
| WP5 black-white hatching | cut after spike S6 (`docs/spikes/S6-patterns.md`) |
| WP6 cost and limit | done (#15); rail target 1.20 x missed narrowly (1.198 / 1.231) |
| WP7 schema-built road cards | cut |
| WP8 release | this release |

## Decisions taken (rulings), with what each costs if wrong

- Result: v1.0.0 release notes now say "do not use, get 1.0.1". Ruling: the old repositories were already archived by the owner (read-only), so the banner PRs could not be opened; adding the banners needs the owner to unarchive, add, re-archive — left to the owner, cost if wrong: a visitor to the old repos sees no pointer (the archive itself signals retirement).
- Ruling: D11's "restores replaced ids" is not changed: the selection restored after a settings rebuild is the panel's selection, already mapped from generated shapes to the user's lines, so it never holds replaced ids — cost if wrong: VectorCraft could refuse a select.set after a rebuild.
- Ruling: D8 now also shows "Drawn."/"Rebuilt." after a road Draw (a panel behaviour change for roads, as the plan said).
- Result: D9 hand-drawn stroke factor keyed on the marking (piece number dropped from the name); D10 crosswalk bars, arrows and culvert pipes keep their true width (no print minimum); F2 creeks and rivers get the band minimum about their own line. Ruling: D12 (junction radii and setbacks growing with widened bands) is cut from 1.5: the fixed offsets are spread through junction, crossing and hub code and scaling them risks the parity of every junction scenario — cost if wrong: junctions look cramped at small print scales (as in 1.0). Ruling: symbol minimum sizes (part of F2) cut likewise; listed in STYLES "not done yet".
- Result: E2 real point limit measured on the live VectorCraft by binary search: a plain two-lane road builds up to about 19,450 points, refused above for its instruction budget. NEAR_LIMIT_POINTS 1,200 -> 12,000 and the note text names the real cause. Ruling: the two ported guard tests (build_guard, rail_guard) changed their point counts from 1,300 to 12,500 on purpose — cost if wrong: none, the old threshold was an estimate.
- Ruling: water keeps deciding closed shapes by coordinates (E2's second half dropped): Road Builder's ported test draws a lake as a ring whose ends meet without the closed flag, and migrated Road Builder documents rely on that — cost if wrong: an open line whose ends meet is drawn as a lake.
- Ruling: E1's target (rail at most 1.20 x Track Layer, and no per-path copy in open_object) is missed narrowly: 1.198 (yard1) and 1.231 (tracks2); the per-path copy stays because the displacement, densify and closing-point drop need an owned path, and a 4-decimal number writer made it worse — cost if wrong: a rail build reaches the host limit about 3% sooner than the target would allow.
- Ruling: objects are named gen.track.<track>.level.<id>.* so they sit in the track layer between ballast and rails (a separate layer would cover the rails or hide under the road).
- Ruling: track layers are now kept above road layers by the panel's organise (a rail over a road is the only kind of road-track crossing there is) — cost if wrong: a document that relied on roads above tracks shows tracks on top.
- Ruling: the panel is symmetric (wider side of a yard both ways); overpass, half barriers and signals are not in 1.5; the plan's agent prompt for level crossings is not written (the card has one choice).
- Spike S6 (live VectorCraft 0.24.1): a plug-in fill may name a pattern swatch — `{"type":"pattern","pattern":"<name>"}` — and the host draws it; but the host refuses the whole build ("the document has no pattern ...") when that swatch does not exist, and a filter cannot create swatches. Ruling: black-white hatch and stipple (F3) cut from 1.5: it needs the panel to create the swatches, a record telling the plug-in they exist, and an answer for a user who deletes one — cost if wrong: black-white stays four flat greys in 1.5. Findings written to docs/spikes/S6-patterns.md.
- Ruling: schema-built road cards (F4) cut from 1.5: no visible effect for the user, the hand-written cards work and are tested, and replacing them risks the panel's 62 ported tests late in the release — cost if wrong: the agent and the panel keep two card models until 2.0.

## Deferred (minor findings from the reviews)

- --force onto an existing symlink replaces the link with a file
- fair_notes with cap 1 gives the slot to tracks; MAX_NOTES duplicates the 30 in roads.rs
- skipCollapsedRings pairs greedily in document order across <path> elements
- document identity uses `title`; Save As now drops a pending proposal (uid/path would be steadier)
- saved-but-rebuild-failed leaves the selection unrestored
- marking_key drops index 3 for any gen.* name; wet/crossing/hub keys with dots could share a hand-drawn factor (cosmetic)
- builder.rs had two doc comments on the constant (fixed while there), 12,000 is an estimate (now labelled), the 1199 no-note check tests nothing at the new threshold, PARITY-RAIL.md "gallery figures below" and the leftover "15 to 30%" sentence, the ledger's "open_object" mention, high-contrast/black-white still hash names per stroke.
- a second crossing of the same road and track gets cut ties but no panel; a refused (too oblique) crossing still cuts ties
- level crossing notes come after fair_notes and can be cut on a busy document
- a junction or crossing layer made later can still sit above track layers
- old footprint_half doc comment sits above crossing_half_mm; a track ending on the road counts as crossing; RAIL.md says the panel covers the road (it reaches 0.5 m past); flangeway band and reuse of the railroad scenario symbols not done

## Owner checks still open

- The agent steps of `docs/PANEL-CHECK.md` (the rest passed on macOS and, by the owner, on Linux).
- The side-by-side look at the old README picture sets (`docs/RELEASE-1.0.md`).
- Banners on the archived old repositories (optional).
