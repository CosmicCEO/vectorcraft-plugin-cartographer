# Spike S6: can the plug-in paint with a pattern (for black-white hatching)?

Run on 2026-11-19 against VectorCraft 0.24.1 (live, control port).

- A pattern swatch is made from drawn art with `object.pattern.make`; an object painted with it is stored as `{"type": "pattern", "pattern": "<swatch name>"}`.
- A plug-in's output may use that same paint on a fill, and VectorCraft draws it (tested with a three-line hatch tile on every road surface).
- If the named swatch does not exist in the document, VectorCraft refuses the whole build: `plug-in output: objects[0].fills[0]: the document has no pattern "<name>"`.
- A filter cannot create swatches.

So hatching needs: the panel (or a command run through the control port) to create the hatch swatches in the document, a record telling the plug-in they exist, and a plan for a user who deletes one (the build would be refused). Not in 1.5; black-white stays four flat greys.
