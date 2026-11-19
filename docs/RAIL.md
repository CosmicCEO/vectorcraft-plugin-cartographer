# Rail scenarios (the `track` kind)

A line named `track.<id>` becomes N-scale model railroad track: rails, ties and roadbed. This is Track Layer's scenario list, ported unchanged in what it builds (see [PARITY-RAIL.md](PARITY-RAIL.md)). Model work is in millimetres at 1:160 and written at **0.16 document points per millimetre** (one document point is one metre of ground, and 1 mm of model is 0.16 m). Commands: Describe Track (`set_tracks`), Describe Yard (`set_yard`), Join Two Lines (`set_joint`). In the panel choose **Railway track (N scale)** for a plain line, or select a `track.<id>` line; the cards are built from `cartographer-cli schema tracks|yard|joint`. The standards behind the numbers are in [STANDARDS-RAIL.md](STANDARDS-RAIL.md).

| Scenario | Status | Description JSON |
|---|---|---|
| Parallel tracks | built | `{"tracks": 1..12, "spacing", "ballast", "ties"}` |
| Yard with ladders, stub or merge | built | `{"mains": 1..4, "left": 0..16, "right": 0..16, "ends": "merge"|"stub", "frog": 4..20, "spacing", "at", "length"}` |
| Words in a panel, and cards for tracks, yards and joints | built (`panel/`) | |
| Crossover between neighbouring tracks | built | `"crossovers": [{"from", "to", "at"}]` in a yard (mains = the tracks) |
| Single turnout, siding | built as a yard with one spur | |
| Crossing of two drawn lines | built | `{"kind": "crossing", "main"}` by the **Join Two Lines** command |
| Branch (siding off another line) | built (single-track branch) | `{"kind": "branch", "frog"}` |
| Join end to end (equal track counts) | built | `{"kind": "join"}` |
| Branch or join with different track counts | designed, not built | Track Layer's `docs/superpowers/specs/2026-11-13-joining-lines-design.md` |
| Slip switches | planned | |
| Wye, balloon loop | planned | |
| Bézier drawn lines | built (rails, roadbed, ties, yards follow them) | |
| Easements on curves | planned | |
| Brand parts and parts list | later | |

## Yard rules

- Mains lie along the drawn line, centred on it, one `spacing` apart (default 13 ft prototype = 24.8 mm, the RP-12 crossover basis).
- Each side has a ladder off the outermost main. The ladder runs at the frog angle; spur *j* leaves it where the ladder is *j* spacings out, then runs parallel. The outermost spur is the ladder itself.
- Each turnout is the NMRA RP-12.35 N straight switch turnout for the chosen frog number (4 to 20): straight switch rail, closure arc, then straight at the frog angle. A spur turns back to parallel with the same shape mirrored. With the lead's end at (x, y) from the point of switch, the frog angle a, and q = x sin(a) - y cos(a), spur *j* leaves the ladder at x + (j s - q - y) / tan(a) along the line, where *s* is the spacing. No. 6, 8 spurs: about 1.26 m of ladder. With `merge`, the same again at the far end.
- The line needs `min_line_mm` of length from `at`; otherwise only the mains are drawn and a note says so.
- Ties of a spur start where it is a tie length clear of its parent; the parent's ties are lengthened over the turnout.
- A crossover goes from main `from` to the neighbouring main `to` as the line is followed: the lead of a No. N turnout, a straight run at the frog angle of length (spacing - 2 y) / sin(a) (y is the sideways rise at the end of the lead), then the lead back the other way, arriving at the heel angle as a real switch rail does. It adds two turnouts. Without `at` it is centred on the yard. `core/tests/rail_turnouts.rs` checks the table's crossover column and that the closure rails end just short of the point of frog; every frog number 4 to 20 fits between tracks 13 ft or 12 ft apart.
- The lengthened switch ties of both mains reach the whole crossover, so it has no ties of its own.
- The roadbed is one band across all the mains (no bare strip between them).

## Joint rules (two drawn lines)

- A joint is a record (`joints` in the document data) naming two lines and the intent; it holds no coordinates. Every build finds where the lines meet again, within 30 mm.
- Found from the drawing alone: **crossing** (the interiors cross, each more than 30 mm from the ends), **branch** (an end of one lies on the other), **join** (two ends meet). Asking for a kind the drawing does not show says what it does show.
- **Branch:** the line that ends is the branch, the other the main. The main is walked in whichever direction lets the branch leave forward; a No. N turnout (RP-12.35) goes on the main's outer track on the branch's side, with its point of switch placed (3 mm steps, after a coarse sweep) so the connector into the branch line turns least. The connector is two arcs and a straight, tangent at both ends, radius from the larger of 450 mm and the turnout's closure radius down to 191 mm (RP-11) plus half the width of the main's tracks. The branch's ties give way within a tie length of the main's track; the main gets lengthened switch ties and a wedge of roadbed. A branch with several tracks is not built.
- **Join:** each line is trimmed back by 0 to 220 mm, the least that lets a connector (same radius rule) join them tangentially. Needs equal track counts and spacing.
- The line whose track runs through is `main` if given, else the one with the wider footprint (tracks, or a yard's mains), else the first by name. The other's ties stop at the edge of the main's strip; both roadbeds and both sets of rails are drawn.
- Everything new is stacked roadbed, then ties, then rails, across all lines, so one line's roadbed never hides another's ties.
- A joint whose lines are missing, or no longer meet, is left out with a note; nothing is guessed.
