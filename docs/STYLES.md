# Styles and the target print scale

Both are document settings (the **Document settings** card at the bottom of the panel, or the Set Settings command). **Draw this** on the card stores the change and rebuilds every road and track at once.

## Styles

| Style | What a build does |
|---|---|
| `medium-contrast` (the default) | Flat fills and strong outlines: exactly what 0.4.0 drew. Nothing is changed. |
| `high-contrast` | Every colour is pushed away from mid-grey (contrast 1.6): light fills lighter, dark lines darker. |
| `black-white` | Every colour becomes one of four greys (black, two greys, white) by its brightness. |
| `hand-drawn` | Road edges, kerbs, fills and marks wander slowly, deterministically from the document `seed` (long edges are cut into pieces so even a straight road wanders), and each stroke gets its own width (0.85 to 1.15 times). |

Rules that hold in every style:

- The line you drew is never moved (its red highlight keeps its colour: it is a tool, not part of the drawing).
- The wander is a function of position only, so two objects that meet at a point move to the same place: a join or an intersection stays closed (a point that lies along another object's edge stays within about 5% of the amplitude of it). The wander is slow (one wave is 48 times its size) and its largest size is 0.4 m of ground at scale `true`, or 0.25 mm on paper at `1:N` (never more than 3 m).
- The same seed and the same document draw the same picture; a different seed draws a different one.
- Railway track is a model at true scale: it is coloured by the style but never moved or widened, so its gauge stays exact.
- A setting that cannot be read (a typo in a stored record, `north_deg` out of range) is ignored and named in the build notes; the other settings still apply. A record from a newer Cartographer is refused, as before.

Not done yet: a minimum print size for water and symbols (only road bands and road strokes get one), hatch and stipple tones for black-white (a filter can write only paths and solid fills, so patterns need their own spike), hairline strokes, a taper along a stroke, per-element styles, a style for rail beyond colour.

## Target print scale

`true`, or `1:N`. With a scale set, a build gives road bands and road strokes a minimum print size, measured on the paper (water, junction details and symbols are not held to one yet):

- a band (a lane, a margin, a bike lane ...) is at least **0.3 mm** wide on paper, which is `0.3 * N / 1000` metres of ground (1.5 m at 1:5000); a narrower band is widened equally about the road's own line;
- a road stroke (kerb lines, lane lines, and also arrows, crosswalk bars and water edges, which share the writer) is at least **0.12 mm** on paper (0.6 m at 1:5000);
- the line you drew and every join stay where they are; junctions and intersections follow the widened bands because they read widths from the same place.

At `true` nothing is widened. Rail is not widened (it is at true model scale). Changing the setting needs a rebuild, which **Draw this** does.

## Cost

Hand-drawn at 1:5000 costs 1.3 to 1.5 times the build fuel of the plain build of the same road, because long edges are cut into pieces (`node scripts/fuel-parity.mjs`, limit 2.0; the 20 km road is 94 M, the host's limit grows with the input size). Medium-contrast at scale `true` costs about 0.2% more than 0.4.0 (one test per point).

## Checking it

`node scripts/live-style-check.mjs out` drives the settings card against a running VectorCraft and exports a PNG and an SVG per style into `out/`.
