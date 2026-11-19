# Checking the panel against a real VectorCraft

The panel is tested against a fake VectorCraft (a stand-in for its control port) and against a stub of the page. Only a real VectorCraft proves that the panel and the application agree. This check is for you to run, on macOS and on Linux. It changes a document, so use a new empty document, never one you care about.

Write what you saw next to each step (`ok` or what went wrong) and send it back.

## Set up

1. `node scripts/check.mjs` ends with no `fix:` line.
2. From a release package: skip the build (the package is already built). From a source checkout: `node scripts/build.mjs`. Then `node scripts/start-vectorcraft.mjs`: it waits until VectorCraft answers on its control port ("ready on control port 7979"). On a Mac it says at once when VectorCraft is already running without the port (quit it with Cmd-Q and run the script again); elsewhere, if nothing answers within 30 seconds, it says to quit VectorCraft and run it again. On Linux, set `VECTORCRAFT_APP` to the AppImage first if the script says it cannot find VectorCraft.
3. In VectorCraft create a new document, then `node scripts/install.mjs`. Expected: `installed Cartographer 1.5.0`.
4. `node panel/server.mjs` and open http://127.0.0.1:8789 next to VectorCraft. Expected: a green dot in the header and the line "Nothing selected".

## Cards

5. Draw a line with the pen and select it. Expected: the panel says a plain line is selected and offers the road description box.
6. Type "a two lane road with a bike lane on each side" and press Propose (needs the `claude` program, or choose another agent). Expected: a proposed road with its elements; **Draw this** builds it in the document, and the line stays on top.
7. Select the road and open "Add or drop a lane on this line". Expected: a form; a proposal lists what changes; Draw makes the lane taper.
8. Draw a second road across the first, select both. Expected: an intersection card; choose Lighted intersection and Draw. Expected: a paved box, crosswalks and signals.
9. Draw a closed shape, choose **Water** for a plain line, make it a lake; draw a road across it, select the road and the lake, choose Bridge. Expected: a bridge with piers, wing walls and guardrails.
10. Hide or lock the road layers in VectorCraft and press Draw again on any card. Expected: the road is rebuilt, and your hiding and locking are put back afterwards.

## Railway track

A. Draw a line at least 400 document points long (400 m of ground, enough for a small yard) and select it. In the mode box choose **Railway track**. Expected: the panel shows a track card (tracks, spacing, roadbed, ties) and the Draw button says "Draw this".
B. Type "double track" and press Propose. Expected: "Proposed: 2 tracks, ..." with a list of what changes; Draw builds rails, ties and roadbed in a layer called "track t1" with a spine and a drawing part.
C. Change Tracks to 3 in the card. Expected: a proposal; nothing in VectorCraft changes until Draw. Enter 99. Expected: a plain message about 1 to 12, and no change.
D. Type "a yard with 4 spurs on the left and 2 on the right, merging back". Expected: a yard proposal that says how much line it needs; if the line is too short Draw shows the plug-in's refusal in the panel.
E. Draw a second line that crosses the first, select both lines (they must both be tracks: select the second and describe it as track too). Expected: the joint card says "cross, ... m along a ... at ... degrees". Choose a join or a branch for lines that meet end to end or in a T. Expected: the card says what it found; Draw builds the crossing, branch or join.
F. Select a road line and a track line together (a road line is named `road.<id>`). Expected: the panel says that is not supported yet and offers nothing to draw.
G. Hide the "track t1 spine" layer and press Draw. Expected: the track is rebuilt and the spine is hidden again afterwards.

`node scripts/live-rail-check.mjs out` runs steps A, C, E, F and G for you against the running VectorCraft and panel (no agent needed; it works in a new document) and exports `out/live-rail.png`. It passed on VectorCraft 0.24.1 on macOS (2026-11-17). Steps B and D need an agent, and looking at the result is still yours to do.

## Settings card

11. At the bottom of the page open **Document settings**. Expected: the target print scale `true`, the style `medium-contrast`, seed 1, north 0.
12. Change the style to `hand-drawn` and the scale to `1:5000`. Expected: the card shows a proposed change and the Draw button turns on; nothing in VectorCraft changes yet.
13. Press **Draw this**, then Save As, close the document and open the saved file. Expected: the card shows the same values after opening it (`hand-drawn`, `1:5000`).
14. Type `1:0` into the scale. Expected: a plain message mentioning `1:N`, and nothing changes in VectorCraft.

## Styles

S. With some roads built, open **Document settings**, choose `hand-drawn`, press **Draw this**. Expected: the roads are rebuilt at once and their edges wander slightly; the line you drew keeps its straight red highlight. Choose `black-white`: every colour becomes grey. Choose `medium-contrast`: the original drawing returns. Set the scale to `1:5000`: thin lines and narrow bands get wider. `node scripts/live-style-check.mjs out` does this for you and saves a picture per style; passed on VectorCraft 0.24.1 on macOS (2026-11-18).

## Switching documents

15. With a proposal showing on any card, create or open a different document. Expected: the proposal disappears and the cards show the new document.

## Agents

16. `node scripts/live-check.mjs out` (needs the `claude` program) replays the first story through the panel and exports a PNG into `out/`. On a Mac with Apple Intelligence, `target/apple-fm-agent --check` prints `{"available": true}` and "Apple Intelligence (on this Mac)" appears in the Agent list; on Linux it must be absent from the list.

## If something fails

Copy the red message from the panel, the lines the panel printed in its terminal, and the step number.
