# Cartographer 1.6: release notes

1.6 comes from the owner's panel check of 1.5.0 (2026-11-19) and was run by the agent under the rules in `docs/superpowers/plans/2026-11-19-cartographer-1.6.md`. Every package went through tests written first, the road and rail parity gates (15/15 and 8/8 identical), the fuel check, CI on macOS and Linux, a fresh review by a separate model with one fix pass, and live checks on the real VectorCraft.

## Work packages

| Package | Result |
|---|---|
| P1 setup | done (#18): `build.mjs` in a package, `start-vectorcraft.mjs` waits for the port and reports a failed start or a running VectorCraft without its port, the panel's "port in use" message |
| P2 panel and crossings | done (#19): shape-tool lines count as lines; a note for a track crossing a road with no crossing stored; gates on the approaching driver's right |
| P3 railway in metres | done (#20): metres everywhere a user looks, schema 2, 1.0 to 1.5 documents converted on read, no N-scale wording |
| P4 grade separation | done (#21): road over and track over |
| P5 release | this release |

## Owner decisions carried out

- Styles: medium-contrast and hand-drawn are good enough for now; defaults for high-contrast and black-white go to the 2.0 list (`docs/superpowers/plans/NEXT.md`).
- "Remove the N-scale references; the target print scale is sufficient." The NMRA sources are still cited in `docs/STANDARDS-RAIL.md`, because the rail shapes come from them.
- Right-hand traffic only: gates stand on the approaching driver's right, and there is no left-hand option.
- For 1.7: every turnout in a yard ladder gets its ties (in `NEXT.md`).

## Decisions taken (rulings), with what each costs if wrong

- On a Mac, a running VectorCraft gets 10 seconds to answer on its port before the start script says it was started without one (a busy app once missed the quick check). Cost if wrong: a 10-second wait before the message.
- The panel lists the document's lines with one walk of the layer tree, in document order (the order the plug-in numbers a road's pieces in), instead of one query per kind. Cost if wrong: none seen; it is also one call instead of five.
- Notes about missing crossings come first in the track half of the 30 kept notes.
- Lengths inside messages (for example the 30 mm the joint search allows) are said in metres by rewriting the message, not by changing the numbers, so the drawing stays identical to Track Layer's.
- Fuel: roads stay within 1.05 of Road Builder (road600 1.037 after removing two document copies the metre conversion added). Rail on a saved 1.6 document costs 0.6% more than 1.5; the fuel check's rail inputs carry no schema, so they also pay the one-time conversion of an old document (yard1 1.216, tracks2 1.267, limit 1.35). Cost if wrong: none for the owner's drawings; the check reads a little high.
- A stored record still holds numbers only; text such as "4000mm" is read as a real length when it is typed, and is stored as metres.
- The deck of a road bridge sits in the track's layer, above every road, so on the bridge it hides the road's lane markings and the drawn road line; clicking there selects the track. The road can be selected anywhere off the bridge. Cost if wrong: a bridge on a busy road looks plainer than the road beside it.

## Deferred (minor findings from the reviews)

- Any program listening on the control port counts as VectorCraft for the start script; the panel server's error handler stays attached after it starts; the package test copies only `scripts/`.
- The missing-crossing note rescans the document for every road and track pair; nothing checks that the live filter input carries the geometry of a shape-tool line (a headless build of a Line road and an Ellipse lake worked); the gate arm of a road anchored at one edge crosses nothing.
- A value typed in metres can lose about 0.0001 m once to rounding; `metres_text` does not convert the first number of a range such as "10-20 mm"; `find-joint` still returns its raw geometry numbers in model millimetres (the panel shows only the summary, which is in metres).
- The bridge deck does not wobble in the hand-drawn style; a treatment from a newer version gets a panel but no tie gap; a lopsided yard's rail-bridge parapet sits at the wider side's distance on both sides; no test yet for an edge-anchored road or a yard under a road bridge.
