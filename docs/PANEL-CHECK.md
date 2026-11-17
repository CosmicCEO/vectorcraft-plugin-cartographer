# Checking the panel against a real VectorCraft

The panel is tested against a fake VectorCraft (a stand-in for its control port) and against a stub of the page. Only a real VectorCraft proves that the panel and the application agree. This check is for you to run, on macOS and on Linux. It changes a document, so use a new empty document, never one you care about.

Write what you saw next to each step (`ok` or what went wrong) and send it back.

## Set up

1. `node scripts/check.mjs` ends with no `fix:` line.
2. `node scripts/build.mjs`, then `node scripts/start-vectorcraft.mjs`. On Linux, set `VECTORCRAFT_APP` to the AppImage first if the script says it cannot find VectorCraft. If VectorCraft was already running, quit it first: the control port is set only at start.
3. In VectorCraft create a new document, then `node scripts/install.mjs`. Expected: `installed Cartographer 0.3.0`.
4. `node panel/server.mjs` and open http://127.0.0.1:8789 next to VectorCraft. Expected: a green dot in the header and the line "Nothing selected".

## Cards

5. Draw a line with the pen and select it. Expected: the panel says a plain line is selected and offers the road description box.
6. Type "a two lane road with a bike lane on each side" and press Propose (needs the `claude` program, or choose another agent). Expected: a proposed road with its elements; **Draw this** builds it in the document, and the line stays on top.
7. Select the road and open "Add or drop a lane on this line". Expected: a form; a proposal lists what changes; Draw makes the lane taper.
8. Draw a second road across the first, select both. Expected: an intersection card; choose Lighted intersection and Draw. Expected: a paved box, crosswalks and signals.
9. Draw a closed shape, choose **Water** for a plain line, make it a lake; draw a road across it, select the road and the lake, choose Bridge. Expected: a bridge with piers, wing walls and guardrails.
10. Hide or lock the road layers in VectorCraft and press Draw again on any card. Expected: the road is rebuilt, and your hiding and locking are put back afterwards.

## Settings card

11. At the bottom of the page open **Document settings**. Expected: the target print scale `true`, the style `medium-contrast`, seed 1, north 0.
12. Change the style to `hand-drawn` and the scale to `1:5000`. Expected: the card shows a proposed change and the Draw button turns on; nothing in VectorCraft changes yet.
13. Press **Draw this**, then Save As, close the document and open the saved file. Expected: the card shows the same values after opening it (`hand-drawn`, `1:5000`).
14. Type `1:0` into the scale. Expected: a plain message mentioning `1:N`, and nothing changes in VectorCraft.

## Switching documents

15. With a proposal showing on any card, create or open a different document. Expected: the proposal disappears and the cards show the new document.

## Agents

16. `node scripts/live-check.mjs out` (needs the `claude` program) replays the first story through the panel and exports a PNG into `out/`. On a Mac with Apple Intelligence, `target/apple-fm-agent --check` prints `{"available": true}` and "Apple Intelligence (on this Mac)" appears in the Agent list; on Linux it must be absent from the list.

## If something fails

Copy the red message from the panel, the lines the panel printed in its terminal, and the step number.
