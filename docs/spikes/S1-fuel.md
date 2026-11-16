# Spike S1: baseline fuel

Run on 2026-11-16 with `tools/fuel` (wasmi, about one fuel per instruction) against Road Builder 0.7.0 and Track Layer 0.9.0 filters, inputs from `scripts/spikes/fuel-inputs.mjs` (Road Builder) and Track Layer's own `scripts/fuel-input.mjs`; limits tested in a headless VectorCraft 0.24.1 session with `scripts/spikes/s1-limit.mjs`.

## The budget

The plug-in docs give a light filter 50 M instructions per call **plus 4,000 per input byte**. Track Layer measured refusal at about 21 M in wasmi's fuel unit for a small input. So the real ceiling depends on how big the input is: a long road brings more budget with it. A `"budget": "heavy"` filter (1 G) exists but is not usable here: installing it needs the user's approval (or a CLI started with `--allow-plugin-grants`), it runs in the background, and a headless session confined to a folder cannot install it at all, which would rule out our headless tests.

## Measured

| Scenario | Input bytes | Fuel | Output bytes |
|---|---|---|---|
| Road Builder: 600 m two-lane road, straight | 121 | 984,433 | 1,603 |
| Road Builder: same road with a 10 m creek and a bridge | 228 | 1,941,285 | 9,404 |
| Road Builder: winding road, 1,001 points (10 km) | about 32,000 | 64,301,957 | |
| Road Builder: winding road, 2,001 points (20 km) | 63,560 | 129,424,996 | 325,669 |
| Track Layer: 3-track yard (2 mains, 4 left, 2 right, merge), 300 mm | 164 | 539,044 | 16,523 |
| Track Layer: 2 parallel tracks, 300 mm | 164 | 460,288 | 16,319 |

Cost per input point on a winding road is about 64,000 fuel.

## Real limit in a headless session (Road Builder, winding road)

| Points | Length | Result |
|---|---|---|
| 501 | 5 km | built |
| 1,001 | 10 km | built |
| 1,501 | 15 km | built |
| 2,001 | 20 km | **refused: plug-in exceeded its instruction budget** |
| 2,501 | 25 km | refused |

So one road in one build tops out between 1,500 and 2,000 points (15 to 20 km at 10 m spacing). A map with many roads shares one build.

## Decisions

- Stay a light filter. Do not use `"budget": "heavy"`.
- Fuel ceilings (my call, from 1.5 times the measured value, rounded up): a straight 600 m road 1.5 M; a road with a creek and bridge 2.9 M; a 3-track yard 0.81 M; 2 parallel tracks 0.69 M; and at most 100,000 fuel per input point on a winding line.
- The engine must report fuel use in CI (the `tools/fuel` check) and must give a plain note when a build is close to the limit, since the host's own message ("exceeded its instruction budget") says nothing about which line to simplify.
- Because the budget grows with input size but the cost per point is higher than the budget it brings (about 64,000 against about 53,000 fuel per point), the practical limit is a total point count across everything selected, not a length.
