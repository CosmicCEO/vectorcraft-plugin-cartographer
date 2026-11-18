# Styles and the target print scale

Both are document settings (the **Document settings** card at the bottom of the panel, or the Set Settings command). **Draw this** on the card stores the change and rebuilds every road and track at once.

## Styles

| Style | What a build does |
|---|---|
| `medium-contrast` (the default) | Flat fills and strong outlines: exactly what 0.4.0 drew. Nothing is changed. |
| `high-contrast` | Every colour is pushed away from mid-grey (contrast 1.6): light fills lighter, dark lines darker. |
| `black-white` | Every colour becomes one of four greys (black, two greys, white) by its brightness. |
| `hand-drawn` | Roads wander a little, deterministically from the document `seed`, and each stroke gets its own width (0.85 to 1.15 times). |

Rules that hold in every style:

- The line you drew is never moved (its red highlight keeps its colour: it is a tool, not part of the drawing).
- The wander is a function of position only, so two objects that meet at a point move to the same place: a join or an intersection never opens. Its largest size is 0.4 m of ground at scale `true`, or 0.25 mm on paper at `1:N` (never more than 3 m).
- The same seed and the same document draw the same picture; a different seed draws a different one.
- Railway track is a model at true scale: it is coloured by the style but never moved or widened, so its gauge stays exact.
- A style that cannot be read (a typo in a stored record) draws as `medium-contrast`; a record from a newer Cartographer is refused, as before.

Not done yet: hatch and stipple tones for black-white (a filter can write only paths and solid fills, so patterns need their own spike), hairline strokes, a taper along a stroke, per-element styles, a style for rail beyond colour.

## Target print scale

`true`, or `1:N`. With a scale set, a build gives everything a minimum print size, measured on the paper:

- a band (a lane, a margin, a bike lane ...) is at least **0.3 mm** wide on paper, which is `0.3 * N / 1000` metres of ground (1.5 m at 1:5000); a narrower band is widened equally about the road's own line;
- a road stroke (kerb lines, lane lines) is at least **0.12 mm** on paper (0.6 m at 1:5000);
- the line you drew and every join stay where they are; junctions and intersections follow the widened bands because they read widths from the same place.

At `true` nothing is widened. Rail is not widened (it is at true model scale). Changing the setting needs a rebuild, which **Draw this** does.

## Cost

Hand-drawn at 1:5000 costs 6 to 8% more build fuel than the plain build of the same road (`node scripts/fuel-parity.mjs`, limit 1.25). Medium-contrast at scale `true` costs about 0.2% more than 0.4.0 (one test per point).

## Checking it

`node scripts/live-style-check.mjs out` drives the settings card against a running VectorCraft and exports a PNG and an SVG per style into `out/`.
