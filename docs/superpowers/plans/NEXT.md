# Next: 1.7 and 2.0 (draft for the owner, 2026-11-19)

Status: 1.6.0 released. Deferred minors from each release are in `docs/RELEASE-1.5.md` and `docs/RELEASE-1.6.md`. This page is the proposed order; the owner confirms or changes it before a plan is written.

## 1.7: small, soon

- **Turnout ties in a yard ladder** (owner): the first turnout of a ladder has all its ties; the turnouts after it have none between the switch and the spur. Every turnout gets its ties.
- The start script already names the panel as the next step (merged after 1.6.0, PR #23); 1.7 ships it.
- Bridge minors from 1.6 that show on paper: the road bridge deck stays straight under a hand-drawn road; a lopsided yard's rail-bridge parapets sit at the wider side's distance on both sides.

## 2.0: the Cartographer app, then the map

### A. The app (owner decision: Tauri)

Goal: no terminal. Double-click Cartographer and work.

- A Tauri app (Rust, the toolkit the plug-in already uses) with one window showing the panel.
- The panel server ships inside the app as a single program with Node built in (Node's single executable application), so nothing else needs installing. The app picks the port; nobody types an address.
- The app finds VectorCraft, starts it with its control port (or says plainly that a running VectorCraft must be quit once), and stops the panel when the app quits.
- First run installs or updates the plug-in in VectorCraft and checks the setup (today's `install.mjs` and `check.mjs`, as buttons with plain results).
- Menu: Check setup, Reinstall plug-in, Open the guide, About (version).
- macOS: a signed and notarized `.dmg`; Linux: an AppImage. Built and published by the release workflow, like the packages today.
- The terminal scripts stay for development and for anyone who prefers them.
- Risks: signing needs an Apple Developer ID (question 1); Tauri's window on Linux (WebKitGTK) differs a little from Safari's, so the panel is tested in both; the app is about 100 MB (Node).

### B. API once (with A, since the panel changes anyway)

- One `set_feature` command instead of the per-kind commands; cards built from the plug-in's schemas instead of hand-written ones. An API change: 1.x documents still open (records are not changed).

### C. A finished printed map

- Labels, scale bar, legend and north arrow (the panel makes text through VectorCraft's own commands; a filter cannot, spike S4).
- Junction radii and setbacks that grow with the print scale; minimum sizes for symbols.
- Style defaults for high-contrast and black-white (owner, 1.6), and black-white hatching (spike S6: the panel creates the pattern swatches first).

### D. More of the world (2.1 or later)

- Areas: trails, forests, swamps, estuaries (design section 6.3).
- Half barriers and signals at level crossings.
- Rail cost: the shared output still copies each path (`docs/PARITY-RAIL.md`).

## Questions for the owner

1. Mac signing: an Apple Developer ID (99 USD a year) lets the app open with a double-click. Without it macOS blocks the first launch; the app must then be opened once with right-click, Open (or allowed in System Settings).
2. Linux: needed in 2.0, or Mac first and Linux in 2.1?
3. Order inside 2.0: A (app) first, B with it, then C? Or is the printed map (C) more urgent than the app?
4. The Ross Bridge Parkway map (traffic-incidents-web) was the reason for the plug-ins: should 2.0 include moving that map onto Cartographer, or does it stay separate?
