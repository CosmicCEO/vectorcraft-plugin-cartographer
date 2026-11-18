# Next: release 1.0 (the owner's steps) and the later releases

Written at the end of Plan 6 (migration, 0.6.0). The plan sequence in the design is finished. What stands between here and 1.0 is `docs/RELEASE-1.0.md`: merge the stacked pull requests, the owner's side-by-side look at the README picture sets, the owner's checks (Linux, the agent steps of `docs/PANEL-CHECK.md`), the version commit, the tag, and the wording that retires the two old repositories. None of it has been done: the tag, the merges and the old repositories are the owner's.

**Open items after 1.0 (none blocks it)**
- Road meets track (a level crossing between a `road.<id>` and a `track.<id>` line): a "not supported yet" note today.
- `set_feature`: the design folds the per-kind commands into one; they keep their ids.
- Schema-built road cards (only settings, track, yard and joint cards come from schemas).
- Hatch and stipple tones for black-white, hairlines, per-element styles, a minimum print size for water and symbols (`docs/STYLES.md`).
- The legacy centreline model, `.pmap` and Add Junction (dropped by decision; they stay in Road Builder).
- `water.rs` decides closed shapes by coordinates, not `Item.closed`; the real point limit is not re-measured (a note above 1,200 points; the host refuses at about 2,000).
- Rail fuel is 15 to 35% above Track Layer's; hand-drawn costs 1.3 to 1.5 times plain.
- Deferred minors of Plans 4 and 5 (see the pull requests and `docs/STYLES.md`).
- The release workflow has never run.

**Later releases (design, section 10):** detect-and-suggest crossings; labels, scale bar, legend, north arrow, then grid and title block (a filter cannot make text: spike S4); trails, forests, swamps, estuaries; per-element styles; mixed-scale insets.
