# Parity with Road Builder

Cartographer 0.2.0 carries Road Builder's road network over unchanged. Two gates prove it, and both are scripts you can rerun.

## 1. Identical SVG

```
node scripts/parity.mjs <vectorcraft-cli> <road_builder_dir> [--png]
```

Each of the 15 scenarios in `scripts/parity/scenarios.json` is drawn and built in a headless VectorCraft session once with Road Builder (commit `ca577ca`) and once with Cartographer, and both results are exported as SVG. The SVGs are compared after normalising them: the XML header, comments, the document title, `id` and `data-*` attributes are dropped and numbers are rounded to 3 decimals. An `id` that is an object name (`gen.*`, `road.*`, `water.*`, `junction.*`) is kept and compared, so a renamed object fails; every other `id` is a per-session counter and is ignored. Any change in a point, a name or an order fails.

Scenarios: two-lane, divided-story, lane-add-drop, junction-tee, four-way-stop, signal-pockets-slip, roundabout, overpass, on-ramp-gore, hub-three, creek-bridge, culvert, causeway-pond, embankment, creeks-joined.

Result (2026-11-16): **all 15 identical.** With `--png` the Cartographer results are also written as PNGs to `target/parity/` for a look.

It needs the headless `vectorcraft-cli`, which exists for macOS only, so it is not part of CI.

## 2. Equal build cost

```
node scripts/fuel-parity.mjs <road_builder_filter.wasm>
```

The same three inputs from the S1 spike go through both filters under `tools/fuel`; the new cost must be at most 1.05 times the old.

| Input | Road Builder | Cartographer | Ratio |
|---|---|---|---|
| 600 m two-lane road | 984,433 | 957,990 | 0.973 |
| Road, 10 m creek and bridge | 1,941,285 | 1,652,419 | 0.851 |
| Winding road, 2,001 points | 129,424,996 | 70,389,328 | 0.544 |

Cartographer is cheaper than Road Builder because Plan 1's merged framework uses `Len2` (a plain square root) instead of `f64::hypot`, which is a slow software routine in WebAssembly. The near-limit note still triggers at 1,200 points; the real build limit of the new plug-in has not been re-measured (`scripts/spikes/s1-limit.mjs` measures it), so the threshold is deliberately the old, safe one.

## What was changed to reach parity

- Tests keep their names and numbers. They differ from Road Builder's only by the crate name and, in ten places, `closed: false` in an `Item` literal (a field Cartographer added).
- Plan 1's review fixes are kept, except that dropping a ring's repeated first point applies only to smooth rings (`curve_fills`): a plain polygon is written exactly as given, as Road Builder writes it.
- Findings from the review of Plan 2, kept in the data: unknown keys nested inside `segments` entries survive every command (Road Builder rebuilt the map and dropped them); every command writes `schema`; a stored setting that cannot be read does not block Build, and Set Settings repairs it.

## Differences from Road Builder that the 15 scenarios do not exercise

All were judged harmless by the reviewer; they are listed so none is a surprise.

- A closed two-anchor curved shape (a lens) is read as closed; Road Builder read it as an open arc, so `Describe Water` defaults it to still water instead of flowing.
- `Out` leaves out what VectorCraft would refuse: subpaths with fewer than 2 points (3 for a shape), non-finite points, an open line whose points are all the same, strokes whose width is zero, negative or not finite.
- `num` clamps beyond about 4.29 million and rounds by a cast, not `round()`; `Len2` replaces `hypot`: last-bit differences only.
- The command "Add Junction" and the legacy centreline model are not ported (Plan 6); a command id the filter does not know runs Build, as before.
