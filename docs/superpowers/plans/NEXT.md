# Next: 2.0

Written at the end of 1.5. The deferred and cut items are listed in `docs/RELEASE-1.5.md`; the bigger ones:

- **Black-white hatching** (spike S6: the panel must create the pattern swatches and the plug-in must know they exist; a missing swatch makes VectorCraft refuse the build).
- **Schema-built road cards** and folding the per-kind commands into one `set_feature` (an API change, best once).
- **Labels, scale bar, legend, north arrow** (a filter cannot make text, spike S4: the panel would create text through VectorCraft's own commands).
- **Junction radii and setbacks that grow with the print scale; symbol minimum sizes.**
- Half barriers and signals at level crossings (bridges, road over and track over, came in 1.6).
- **Style defaults for high-contrast and black-white** (owner, 1.6 panel check: "medium-contrast and hand drawn work well enough for now").
- **Areas:** trails, forests, swamps, estuaries (design section 6.3).
- Rail cost: the shared output still copies each path (`docs/PARITY-RAIL.md`).

## 1.7 (owner, 2026-11-19)
- **Turnout ties in a yard ladder:** the first turnout of a ladder is drawn with all its ties, but the turnouts after it have none between the switch and the spur (the ladder runs bare). Every turnout in a ladder gets its ties.

