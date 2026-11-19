# Rail parity: Cartographer against Track Layer 0.9.1

The track kind was ported from Track Layer (`CosmicCEO/vectorcraft-plugin-tracklayer`, commit 5859e9b). This page says how that is proven and what is
different on purpose.

## What is compared

Eight scenarios in `scripts/parity/rail-scenarios.json`, written in **model millimetres**: Track Layer's four examples (`oval`, `station`, `terminal`,
`turnouts`) plus `two-tracks`, `crossing`, `branch` and `join`. Each is built in a headless VectorCraft twice, once by Track Layer (72/25.4 document
points per mm) and once by Cartographer (0.16 points per mm: one document point is one metre of ground, and 1 mm of model at 1:160 is 0.16 m; see
`core/src/rail_standards.rs`). Both SVG exports are compared by `compareSvgScaled`: every length in Track Layer's SVG is multiplied by
0.0564444 (0.16 / (72 / 25.4)) and must equal Cartographer's within 0.001 point; element names, order, structure and every number that is not a
length (miter limits, opacity) must match exactly.

Result: **8 of 8 identical**.

## What is different on purpose

Track Layer 0.9.1 collapsed one ring per branch to a point: the ballast wedge where a branch leaves its main was written as `M p L p Z`, so the
wedge never showed. Cartographer's output code (from Road Builder) drops a repeated closing point on a smooth closed ring, so the wedge is drawn.
The gate removes each collapsed ring from Track Layer's SVG and the ring that starts at the same place from Cartographer's, counts them, and prints
the count (`station` 1, `turnouts` 5, `branch` 1). Nothing else is skipped. Also:

- The highlight on the line you drew is 0.26 mm of model wide at any scale (0.0423 points, written as 0.042 because the line you drew keeps 3 decimals; 0.75 points in Track Layer).
- Rail numbers are written with 4 decimals (0.0006 mm of model) and curves thinned at 0.00677 points; roads keep 3 decimals and 0.12.

## Cost

`node scripts/fuel-parity.mjs <road_builder_filter.wasm> <track_layer_filter.wasm>`:

| Input | Old fuel | New fuel | Ratio | Limit |
|---|---|---|---|---|
| yard1 (2 mains, 4 + 2 spurs, 1.5 m of model) | 5,771,887 | 7,188,981 | 1.246 | 1.35 |
| tracks2 (double track, 1.8 m of model) | 1,473,873 | 1,911,571 | 1.297 | 1.35 |

The four gallery layouts (`docs/GALLERY.md`), measured the same way: oval 6,649,676 against 7,905,407 (1.19), station 8,809,221 against 10,557,210 (1.20), terminal 10,546,386 against 13,611,174 (1.29), turnouts 7,144,847 against 8,221,678 (1.15). The largest is about two thirds of the host's limit.

Rail builds cost about 15 to 30% more than in Track Layer, so a rail build reaches the host's limit (about 21 M fuel) a little sooner. Where it
comes from: 4 decimals instead of 3 digits per number (about 8%), and the shared output code (copies each path to check it, skips non-finite
points: about 13%). Reading the road element library was another 0.7 M for every build; a build with no road in it no longer reads it. Roads keep
the 1.05 limit (see `docs/PARITY.md`).

## How to rerun

```
cargo +stable build --release --target wasm32-unknown-unknown -p track-layer-filter    # in the Track Layer repository
node scripts/build.mjs                                                                  # here
node scripts/parity.mjs <vectorcraft-cli> <track_layer_dir> --scenarios scripts/parity/rail-scenarios.json \
  --old-id com.cosmicceo.tracklayer.build --old-wasm <track_layer_filter.wasm> --factor 0.0564444
node scripts/fuel-parity.mjs <road_builder_filter.wasm> <track_layer_filter.wasm>
node scripts/render.mjs <vectorcraft-cli> <scenario.json> target/parity/out.png      # a look at one scenario (Track Layer's example format)
```
