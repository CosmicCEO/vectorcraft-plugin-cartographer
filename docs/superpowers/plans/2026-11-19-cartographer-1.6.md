# Cartographer 1.6 Plan (autonomous)

From the owner's panel check of 1.5.0 (2026-11-19). Same autonomy rules as `2026-11-19-cartographer-1.5.md` (rulings in the ledger, every package: tests first, parity, fuel, CI on both systems, an Opus review with one fix pass, a live check when the drawing or the panel changes).

| Package | Items from the owner's check |
|---|---|
| P1 setup | `build.mjs` in a package says there is nothing to build; `start-vectorcraft.mjs` waits for the control port and says when a running VectorCraft must be quit first; the panel says plainly when its port is in use. |
| P2 panel and crossings | Shape-tool objects (line, ellipse, rectangle, polygon, star: VectorCraft reports them as `Line`, `Ellipse`, ... not `Path`) count as lines in the panel and its layers; a build note when a track crosses a road with no level crossing; gates on the approaching driver's right (right-hand traffic only, by owner decision). |
| P3 railway in metres | No N-scale or model-railroad wording; track, yard and joint values in metres (spacing 3.96 m, not 24.765 mm); stored records in metres with an automatic conversion of documents saved by 1.0 to 1.5 (record version bump); the rail code keeps working in millimetres inside so Track Layer parity holds; NMRA stays the cited source of the geometry. |
| P4 grade separation | A third and fourth level-crossing treatment: `road over` (a road bridge over the track) and `track over` (a rail bridge over the road, the road in an underpass). |
| P5 release 1.6.0 | Version, changelog, release notes (rulings and deferred), NEXT.md (style defaults for high-contrast and black-white go to the 2.0 list, by owner decision), tag, verify the published package. |
