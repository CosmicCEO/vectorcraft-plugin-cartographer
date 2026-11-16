# Next: Plan 2, the band engine and the Road Builder port

Written at the end of the foundation (Plan 1). Plan 2 implements spec section 10, Phase 2, and must take these spike results into account:

- **S1 (fuel):** one road builds up to about 1,500 points (15 km at 10 m) and is refused at 2,000, at about 64,000 fuel per point. Stay a light filter; do not use the `heavy` budget. Give the engine a per-kind fuel ceiling and a plain note when a build is near the limit. Starting ceilings are in `docs/spikes/S1-fuel.md`.
- **S3 (survival):** Save, Save As, reopen and PDF keep the plug-in data; SVG keeps names, shapes and spine direction but not the data. Keep names as `<kind>.<id>` and `gen.<kind>.<id>.<piece>.<role>`.
- **S4 (text):** a filter cannot create text objects. Nothing in Plan 2 needs text.
- **S2 (Linux):** no Linux headless CLI, so the Road Builder render comparison runs on macOS only; Linux CI runs tests only. The manual Linux checklist is still the owner's to run.

Scope of Plan 2: the band grammar (the Road Builder cross-section shorthand, generalised), the engine that draws bands along a line, roads, lane changes, junctions, intersections, scenarios, hub, gore, water, embankment and wet crossings, with every Road Builder Rust and panel test ported with its numbers unchanged (the parity gate), and the README images of Road Builder reproduced in headless renders. Plan 3 is the Track Layer port and the standards-based library.

Before writing Plan 2, re-read the design and ask the owner which of Road Builder's open items (a facts export for the simulation, a river width that changes along one line) are in scope.
