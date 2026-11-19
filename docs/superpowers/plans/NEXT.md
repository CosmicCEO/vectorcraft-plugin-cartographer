# Next: 1.7 to 2.5 (agreed with the owner, 2026-11-19)

Status: 1.6.0 released. Deferred minors from each release are in `docs/RELEASE-1.5.md` and `docs/RELEASE-1.6.md`. This page is the agreed order; each release gets its own plan before work starts.

## 1.7: small, soon

- **Turnout ties in a yard ladder** (owner): the first turnout of a ladder has all its ties; the turnouts after it have none between the switch and the spur. Every turnout gets its ties.
- Already merged after 1.6.0, shipped in 1.7: the start script names the panel as the next step (#23); a plain line or shape selected with a track says to make it a road first (#25).
- Bridge minors from 1.6 that show on paper: the road bridge deck stays straight under a hand-drawn road; a lopsided yard's rail-bridge parapets sit at the wider side's distance on both sides.

## Owner decisions (2026-11-19)

1. No Apple Developer ID: the Mac app is not signed. The first launch is opened once with right-click, Open (or allowed in System Settings); the guide says so.
2. Linux in 2.1; 2.0 is the Mac app.
3. The printed map is 2.5, after some optimization releases.
4. The Ross Bridge Parkway map is no longer a reason for this work: Cartographer stands on its own.

## 2.0: the Cartographer app (Mac)

Goal: no terminal. Double-click Cartographer and work.

- A Tauri app (Rust, the toolkit the plug-in already uses) with one window showing the panel.
- The panel server ships inside the app as a single program with Node built in (Node's single executable application), so nothing else needs installing. The app picks the port; nobody types an address.
- The app finds VectorCraft, starts it with its control port (or says plainly that a running VectorCraft must be quit once), and stops the panel when the app quits.
- First run installs or updates the plug-in in VectorCraft and checks the setup (today's `install.mjs` and `check.mjs`, as buttons with plain results).
- Menu: Check setup, Reinstall plug-in, Open the guide, About (version).
- An unsigned `.dmg`, built and published by the release workflow like the packages today; the guide shows the one-time right-click, Open.
- The terminal scripts stay for development and for anyone who prefers them.
- With it, the API change best made once: one `set_feature` command instead of the per-kind commands, and cards built from the plug-in's schemas instead of hand-written ones. 1.x documents still open (records are not changed).
- Risks: macOS may warn more strongly about an unsigned app in later versions; the app is about 100 MB (Node).

## 2.1: Linux app

- The same app as an AppImage; the panel tested in Tauri's Linux window (WebKitGTK), which differs a little from Safari's.

## 2.2 to 2.4: optimization (to be detailed with the owner)

- Rail cost: the shared output still copies each path (`docs/PARITY-RAIL.md`); rail builds are 1.2 to 1.27 times Track Layer's.
- Larger drawings: the point limit (a plain road builds to about 19,450 points) and build time on big documents.
- Deferred minors from `docs/RELEASE-1.5.md` and `docs/RELEASE-1.6.md`.

## 2.5: a finished printed map

- Labels, scale bar, legend and north arrow (the panel makes text through VectorCraft's own commands; a filter cannot, spike S4).
- Junction radii and setbacks that grow with the print scale; minimum sizes for symbols.
- Style defaults for high-contrast and black-white (owner, 1.6), and black-white hatching (spike S6: the panel creates the pattern swatches first).
- **Areas with fill patterns and a key** (owner, 2026-11-19: "it's important to have a key and fill pattern for effectiveness"). A closed shape is named as an area in the panel, as a lake is today. One table gives every area kind its pattern swatch (colour and mark); the panel creates the swatches in the document before a build (spike S6), the plug-in fills with them, and the key shows the same swatch beside each kind used on the map, so map and key always match. Each kind also has a black-white hatching. Colours follow OpenStreetMap Carto (CC0). The ten kinds, from https://wiki.openstreetmap.org/wiki/OpenStreetMap_Carto/Areas:

| # | Area | OSM tag | Pattern |
|---|---|---|---|
| 1 | Building | `building=*` | tan, darker outline |
| 2 | Parking | `amenity=parking` | grey |
| 3 | Woodland / forest | `natural=wood`, `landuse=forest` (leaf type optional) | green, tree marks |
| 4 | Park / grass | `leisure=park`, `landuse=grass` | light green |
| 5 | Wetland (marsh, swamp, reedbed) | `natural=wetland` + `wetland=*` | pale, marsh strokes |
| 6 | Beach / sand | `natural=beach`, `natural=sand` | sand, dots |
| 7 | Farmland (orchard optional) | `landuse=farmland`, `landuse=orchard` | pale; orchard in rows of dots |
| 8 | Land use | `landuse=residential/commercial/industrial` | three faint tints |
| 9 | Railway station and platform | `railway=station`, `railway=platform` | station tint; platforms along the track |
| 10 | School grounds | `amenity=school` | pale, outlined |

- The key (legend) lists the roads, tracks, water and areas actually used, with the same swatches and line styles as the map.

## Later (not scheduled)

- Trails and estuaries (design section 6.3); more area kinds after the first ten.
- Half barriers and signals at level crossings.

## Open question

- Which optimization matters most to you for 2.2 to 2.4: build speed and cost, bigger drawings, or the panel's ease of use?
