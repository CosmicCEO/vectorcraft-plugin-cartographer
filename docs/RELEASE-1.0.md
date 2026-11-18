# Release 1.0: what is done and what is yours

Nothing here has been tagged, released or pushed to the old repositories. Those steps are yours.

## The parity gate (design, section 10)

"Release 1.0 requires every old Rust and panel test to pass in Cartographer, and a render set of both old repos' README images to match in headless renders (side by side, reviewed by the owner)."

| Part | State |
|---|---|
| Road Builder's Rust tests ported with their names and numbers | done (Plans 2 and 3) |
| Road Builder's panel tests (62) | done (Plan 3) |
| Track Layer's Rust tests (72 rail module tests and 3 standards tests) | done (Plan 4) |
| Track Layer's panel tests (16) | done (Plan 4); its provider and server tests (22) are not ported again, Plan 3's copies from Road Builder cover the same code |
| Same drawing as Road Builder | 15 scenarios, identical SVG (`node scripts/parity.mjs`) |
| Same drawing as Track Layer | 8 scenarios, identical when scaled (`docs/PARITY-RAIL.md`) |
| Old documents carry over | `docs/MIGRATION.md`, live-checked against both real plug-ins |
| Panel against a real VectorCraft | rail, styles and migration live checks passed on macOS; you report the Linux check and the agent steps of `docs/PANEL-CHECK.md` passed |
| **README image render set, side by side** | **yours, not done**: Road Builder's `docs/images/` (3 pictures) and Track Layer's `docs/img/` (14 pictures) next to Cartographer's: `docs/img/gallery-*.png` and `docs/img/rail-*.png` are Track Layer's set re-rendered or copied; render Road Builder's with `node scripts/parity.mjs <cli> <road_builder_dir> --png` (pictures in `target/parity/`) |

## To release

Done on 2026-11-18: the stacked pull requests are on `main` (#6, #9), the version is 1.0.0, CI is green on macOS and Linux, and the release workflow's packaging was tested with a manual run before tagging.

1. `main` is at 1.0.0 and CI is green.
2. Tag: `git tag v1.0.0 && git push origin v1.0.0`. The workflow (`.github/workflows/release.yml`) checks that the tag matches the version, builds and tests the package on macOS and Linux, and publishes the release. If it fails, `node scripts/package.mjs 1.0.0` on each system builds the same packages by hand.
3. Then the two old repositories (below).

## Words for the two old repositories

Put this at the top of each README, then archive the repository on GitHub (Settings, Archive this repository). Nobody here has done it.

Road Builder (`vectorcraft-road-builder`):

> **Road Builder has been replaced by [Cartographer](https://github.com/CosmicCEO/vectorcraft-plugin-cartographer).** Cartographer draws everything Road Builder did, with the same output, and does railway track, styles and print scales too. To bring a Road Builder document across, see Cartographer's `docs/MIGRATION.md`. The older centreline model, `.pmap` files and Add Junction are not in Cartographer; they stay here, which is why this repository is archived, not deleted.

Track Layer (`vectorcraft-plugin-tracklayer`):

> **Track Layer has been replaced by [Cartographer](https://github.com/CosmicCEO/vectorcraft-plugin-cartographer).** Cartographer draws everything Track Layer did (rails, ties, roadbed, turnouts, yards, crossings, branches and joins, with the same drawing) next to roads. To bring a Track Layer document across, see Cartographer's `docs/MIGRATION.md`; its lines are scaled for you.
