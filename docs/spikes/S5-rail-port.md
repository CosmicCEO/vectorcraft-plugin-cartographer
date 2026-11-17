# Spike S5: how well does Track Layer's code drop into Cartographer?

Run on 2026-11-16 in a scratch clone of Cartographer `main` (0.2.0, after Plan 2); nothing was committed. Track Layer is `CosmicCEO/vectorcraft-plugin-tracklayer` at `5859e9b` (0.9.1): 3,004 lines of Rust source, 82 Rust tests in 10 files, a panel of 1,090 lines with 38 tests.

## What was done

1. Copied Track Layer's `tracks`, `turnout`, `turnout_table`, `yard`, `ties`, `joint`, `connector`, `branch` modules, its `builder` (as `rail_builder`) and its `standards` (as `rail_standards`) into `core/src`, and its tests into `core/tests` (crate name, and the two renames, changed; nothing else).
2. Pointed the copied modules at `rail_standards` (Cartographer already has a `standards` module for Road Builder's database).
3. Built and ran the 75 tests that are not already in Cartographer (`offset.rs`, 7 tests, came over in Plan 1).

## Results

- **It compiles clean and all 75 tests pass with no change to Track Layer's code**, apart from the module rename. Plan 1's merge of the framework (`geom`, `offset`, `output`, `vcdoc`, `abi`) is what makes this possible: Cartographer's `Out` is Track Layer's superset.
- **Units.** Track Layer works in model millimetres and converts at the edge with `PT_PER_MM = 72 / 25.4` (so a drawing prints at true N-scale size). Cartographer's rule is 1 document point = 1 metre of ground. The converting constant becomes `0.16` (1 mm of model is 0.16 m of ground at 1:160). Every test goes through that constant, so nothing else changes.
- **With only that change, 5 of the 75 tests fail** (3 in `tracks`, 1 in `yards`, 1 in `curves`). Both causes are settings inside the shared `Out` that were tuned for the old scale:
  - `num()` rounds written numbers to 3 decimals. 0.001 points is now 0.006 mm of model, and Track Layer's tests check the gauge to 0.001 mm.
  - `THIN = 0.12` points, the tolerance below which curve points are dropped, is now 0.75 mm of model (it was 0.04 mm), too coarse for rails that are 0.5 mm wide.
- **Both fixed together:** writing 5 decimals and using a thinning tolerance of 0.00677 points (the same 0.04 mm of model) makes all 75 tests pass at the new scale. In the spike these were changed for the whole of `Out`; in the real port they must be settings of `Out` that a kind sets for its own objects (road output must keep 3 decimals and 0.12, or the road parity gate and its fuel numbers change).

## What it means for Plan 4

- The port is mostly bookkeeping: the hard part, the framework, is done. The new work is the per-kind `Out` settings, merging the two builders and the two standards databases, the panel, and the parity method.
- The five failures are the only place the units change touches the tests; the fix is in `Out`, not in the tests, so the tests keep their numbers.
- The `notes` key is written by both builders and must be merged, not overwritten.
