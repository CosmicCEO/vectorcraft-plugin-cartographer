# Cartographer Panel Port Implementation Plan (Plan 3)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port Road Builder's companion panel so every road command (roads, lane changes, junctions, intersections, hubs, water, embankments, crossings) has its card, agent and tests again, with the 62 Road Builder panel tests passing unchanged, next to the settings card that Cartographer already has.

**Architecture:** Port first, generalise later. Road Builder's `Panel` class, providers, prompts, layers and page move over as they are; the settings card (built from a schema) stays and becomes one more card on the same page, with its own buttons and state so it never disturbs a proposal made from a selection. The cards for roads stay hand-written until the rail kind arrives (Plan 4), when a second kind justifies building them from schemas.

**Tech Stack:** Node 22 (zero dependencies), the native `cartographer-cli`, a fake VectorCraft for tests, Swift (macOS only, optional) for the Apple Intelligence helper.

**Spec:** `docs/superpowers/specs/2026-11-16-cartographer-design.md` (sections 6.7, 6.8). **Source being ported:** `/Volumes/Mingus/Developer/vectorcraft-road-builder/panel` at commit `ca577ca`. **Brief:** `docs/superpowers/plans/NEXT.md`.

## Global Constraints

- Plans 1 and 2's constraints still hold (Node 22, zero dependencies, scripts in Node, never push to `main`, never merge or tag, commit trailers `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>` and `Claude-Session: https://claude.ai/code/session_01HBq3ghaocqFgSq9fTWECkM`, plain prose in documents).
- **Parity:** the Road Builder panel tests (`providers.test.mjs` 17, `panel.test.mjs` 40, `server.test.mjs` 5) keep their names and expected values; they may change only import paths and names (`road-builder-cli` to `cartographer-cli`, `ROAD_BUILDER_*` to `CARTOGRAPHER_*`, port 8787 to 8789, plug-in id).
- Panel port is 8789; environment `CARTOGRAPHER_PROVIDERS` and `CARTOGRAPHER_CLI`; the user's providers file is `~/.config/vectorcraft-cartographer/providers.json`. The panel serves only `127.0.0.1`, checks Host on every request and Origin and `application/json` on every change, and shows document and agent text with `textContent`, never as HTML.
- The CLI exits 1 when `ok` is false (Plan 2): the panel reads the CLI's JSON from stdout whatever the exit code.
- The panel never touches the owner's open document in tests (fake VectorCraft only); the live check is the owner's step.
- Not in this plan: schema-built road cards (Plan 4), the legacy `.pmap` work (Plan 6), the demo video script.

## Review Focus

1. A CLI failure (exit code 1 with `{"ok": false, "error": ...}`) reaches the card as its plain message, never as a raw process error (Task 1).
2. Switching documents (the title changes) drops a pending proposal of every kind (road, junction, crossing, hub, lane change, water, crossing with water, embankment, settings) and reads the new document (Task 2).
3. A malformed address, a foreign Host or Origin, a wrong content type and an unknown action cannot crash the server or change anything (Task 3).
4. Text from the document or from an agent is never inserted as HTML: the page's script has no HTML sink, and an object named `<img src=x onerror=alert(1)>` appears as text (Task 3).
5. An agent that is not installed (the `claude` program, the Apple helper on Linux) is hidden or answers with a plain message, never a crash (Task 1).

---

## Tasks

### Task 1: Agents, prompts, changes and the CLI client

**Files:**
- Create: `panel/lib/prompt.mjs`, `panel/lib/providers.mjs`, `panel/lib/changes.mjs`, `panel/providers.json`
- Modify: `panel/lib/cli.mjs`
- Test: `panel/test/providers.test.mjs`, `panel/test/cli.test.mjs`

**Interfaces:**
- Consumes: Plan 2's CLI commands (`parse`, `edit`, `diff`, `parse-junction`, `parse-crossing`, `parse-hub`, `parse-transition`, `parse-water`, `parse-embankment`, `parse-wet`, `library`).
- Produces (names unchanged from Road Builder): `cli.mjs` exports `runCli`, `parseCrossSection`, `library`, `editCrossSection`, `parseJunction`, `diffCrossSection`, `parseCrossing`, `parseHub`, `parseTransition`, `parseWater`, `parseEmbankment`, `parseWet` and keeps Plan 1's `parseSettings` and `schema`; `providers.mjs` exports `loadProviders`, `ask`, `propose`, `proposeJunction`, `proposeCrossing`, `proposeHub`, `proposeTransition`, `proposeFeature`; `prompt.mjs` the system and user prompt functions and `extractJson`, `extractLine`; `changes.mjs` `describeChanges(before, after, what)`.

- [ ] **Step 1: Copy `providers.test.mjs`** with the allowed renames. **Write `panel/test/cli.test.mjs`** with `a_failing_cli_reply_is_returned_not_thrown` (`runCli(["parse-settings", "{\"target_print_scale\":\"1:0\"}"])` resolves to an object with `ok === false` and an `error` containing `1:N`, though the process exits 1) and `a_missing_cli_names_the_fix` (with `CARTOGRAPHER_CLI` pointing at a missing file the rejection says `cartographer-cli not found` and `node scripts/build.mjs`).
- [ ] **Step 2: Run** `node --test panel/test/providers.test.mjs panel/test/cli.test.mjs`. Expected: FAIL (modules and exports missing; `runCli` rejects on exit code 1).
- [ ] **Step 3: Copy `prompt.mjs`, `providers.mjs`, `changes.mjs` and `providers.json`**, apply the renames (`CARTOGRAPHER_PROVIDERS`, the config folder, labels), and port Road Builder's CLI wrappers into `cli.mjs`. In `runCli` resolve with the parsed stdout whenever stdout holds JSON, whatever the exit code; reject only when there is no stdout.
- [ ] **Step 4: Run both test files.** Expected: PASS (17 ported plus 2 new).
- [ ] **Step 5: Commit** (`Port the agents, prompts and the CLI client`).

### Task 2: Document reading, layers and the Panel

**Files:**
- Create: `panel/lib/layers.mjs`
- Modify: `panel/lib/doc.mjs`, `panel/lib/panel.mjs`, `panel/test/fake-vectorcraft.mjs`, `panel/test/panel.test.mjs`
- Test: `panel/test/panel.test.mjs`

**Interfaces:**
- Consumes: Task 1's modules.
- Produces: `doc.mjs` adds Road Builder's `segmentId`, `junctionId`, `waterId`, `shapeOwner`, `classify`, `nodes`, `allPaths`, `readSpec` and the constants `SET_CROSS_SECTION`, `SET_JUNCTION`, `SET_CROSSING`, `SET_HUB`, `SET_TRANSITION`, `SET_WATER`, `SET_EMBANKMENT`, `SET_WET` next to `SET_SETTINGS` and `PLUGIN_ID` (`com.cosmicceo.cartographer.build`); `layers.mjs` exports `organise`, `unlockOurs`, `ownerOf`; `Panel` has the constructor `{ control, providers }` (providers optional: without it the agent features answer with a plain message) and Road Builder's methods (`start`, `stop`, `tick`, `propose`, `edit`, `setText`, `selectLine`, `setMode`, `match`, `editJunction`, `editCrossing`, `editTransition`, `removeTransition`, `editHub`, `editWater`, `editWet`, `editEmbank`, `removeEmbank`, `cancel`, `draw`, `snapshot`, `setAgent`) plus `editSettings(patch)`, `drawSettings()`, `cancelSettings()`; the state gains `settings: { values, summary, card } | null` and `settingsPending: { settings, summary } | null`.

- [ ] **Step 1: Copy `panel.test.mjs` (40 tests) and the fake VectorCraft** from Road Builder with the renames, and replace Plan 1's `panel.test.mjs` tests by these settings tests, renamed for the new method names and state key: `the settings card shows defaults when nothing is stored`, `a bad scale is refused before it reaches VectorCraft`, `a choice is previewed, then Draw stores it` (`editSettings`, then `drawSettings` records exactly `select.all` and `set_settings`), `Cancel drops a proposal and Draw with nothing proposed does nothing`, `stored settings are shown, and a second change keeps the first`, `no open document is reported plainly`, `VectorCraft not running is reported plainly`. Add `switching documents drops every kind of proposal`: for each of an edit of the road text, `editJunction`, `editCrossing`, `editHub`, `editTransition`, `editWater`, `editWet`, `editEmbank` and `editSettings` that is possible in a fake document, then change the fake document's title, `tick()`, and assert `pending === null` and `settingsPending === null` and that the settings values are the new document's.
- [ ] **Step 2: Run** `node --test panel/test/panel.test.mjs`. Expected: FAIL (modules and methods missing).
- [ ] **Step 3: Port `layers.mjs` and extend `doc.mjs`;** replace Plan 1's `panel.mjs` by Road Builder's `Panel`, then add the settings parts: read the stored settings when the revision or the document title changed (not on every tick), build `settings.card` with `cardModel(schema("settings"), values)`, `editSettings` checks the change with `parseSettings` (a bad value sets `message` and leaves `settingsPending` alone), `drawSettings` runs `select.all` then `SET_SETTINGS` and reads again, `cancelSettings` clears. In the refresh, a changed title clears `pending` and `settingsPending` and forces a re-read of everything.
- [ ] **Step 4: Run** `node --test panel/test/panel.test.mjs`. Expected: PASS (40 ported plus 8 new).
- [ ] **Step 5: Commit** (`Port the Panel, layers and document reading; settings alongside`).

### Task 3: Server and page

**Files:**
- Modify: `panel/server.mjs`, `panel/public/index.html`, `panel/public/app.js`, `panel/public/style.css`, `panel/test/server.test.mjs`
- Create: `panel/test/page.test.mjs`

**Interfaces:**
- Consumes: Task 2's `Panel`.
- Produces: the server's actions are Road Builder's (`propose`, `edit`, `text`, `draw`, `cancel`, `selectline`, `mode`, `match`, `junction`, `crossing`, `hub`, `transition`, `removetransition`, `water`, `wet`, `embank`, `removeembank`, `agent`) plus `settings`, `drawsettings`, `cancelsettings`; `createServer({ panel, port })` unchanged; the settings card sits at the bottom of the page and is shown whenever a document is open.

- [ ] **Step 1: Copy `server.test.mjs`** (5 tests, title `Cartographer`) and keep Plan 1's extra server tests (`a malformed address cannot crash the server`, `propose and draw settings over HTTP` renamed to use `drawsettings`). **Write `page.test.mjs`:** `the page script has no HTML sink` (the text of `app.js` has none of `innerHTML`, `outerHTML`, `insertAdjacentHTML`, `document.write`, `eval(`, `new Function`); `markup in a name is passed through as plain data` (a fake document whose selected path is named `<img src=x onerror=alert(1)>` gives `snapshot().selection[0].name` equal to that exact string, so the page's `textContent` is what keeps it inert); `the page is served with a policy that forbids inline script` (the response for `/` has a `content-security-policy` header containing `script-src 'self'`, `object-src 'none'` and `base-uri 'none'`); `an unknown action is a 404 and an inherited key is not an action` (`POST /api/constructor` and `/api/toString` answer 404).
- [ ] **Step 2: Run** `node --test panel/test/server.test.mjs panel/test/page.test.mjs`. Expected: FAIL.
- [ ] **Step 3: Port the server and the page.** Copy Road Builder's `server.mjs` actions and `public/` files (title `Cartographer`, header `Cartographer`); keep the `new URL` inside the `try`; look the action up with `Object.hasOwn`; send `content-security-policy: script-src 'self'; object-src 'none'; base-uri 'none'` with every page response (it only forbids inline script and plug-ins; styles are left alone because the page sets them from script); add the three settings actions and put Plan 1's settings card (its `control` builder and schema-driven fields) at the bottom of `index.html` and `app.js`, using the `settingsPending` state and the `drawsettings` and `cancelsettings` actions.
- [ ] **Step 4: Run** the panel tests (`node --test panel/test/*.test.mjs`). Expected: PASS (all).
- [ ] **Step 5: Commit** (`Port the server and the page; settings card at the bottom`).

### Task 4: Optional helpers and the live check

**Files:**
- Create: `panel/apple-fm/main.swift`, `scripts/live-check.mjs`, `examples/story1.svg`, `docs/PANEL-CHECK.md`
- Modify: `scripts/build.mjs`, `scripts/lib/checks.mjs`, `scripts/check.mjs`
- Test: `scripts/test/applefm.test.mjs`

**Interfaces:**
- Produces: `scripts/lib/applefm.mjs` exports `appleHelperPlan(platform: string, hasSwiftc: boolean): { build: string[] } | null` (the command to build the helper, or null when it cannot be built here); `build.mjs` builds `target/apple-fm-agent` on macOS when `swiftc` exists and says in one line when it does not (never an error).

- [ ] **Step 1: Write the failing test** `applefm.test.mjs`: `the helper builds only on macOS with swiftc` (`appleHelperPlan("darwin", true).build` equals `["swiftc", "-parse-as-library", "-O", "-o", "target/apple-fm-agent", "panel/apple-fm/main.swift"]`; `("darwin", false)`, `("linux", true)` give `null`).
- [ ] **Step 2: Run** `node --test scripts/test/applefm.test.mjs`. Expected: FAIL.
- [ ] **Step 3: Implement `applefm.mjs`;** copy `main.swift` and `story1.svg` unchanged; port `live-check.mjs` (panel URL `http://127.0.0.1:8789`, VectorCraft port 7979, the example path `../examples/story1.svg`) and extend `build.mjs` to run the helper plan; add an optional line to the install check for the helper (a note, never a failure). Write `docs/PANEL-CHECK.md`: the manual steps for the owner on macOS and on Linux (start VectorCraft with its control port, `node scripts/build.mjs`, `node scripts/install.mjs`, `node panel/server.mjs`, `node scripts/live-check.mjs out`), what each step should show, and where to write results. Verify on this Mac that the helper builds and `target/apple-fm-agent --check` prints `{"available": true}`.
- [ ] **Step 4: Run** `node --test scripts/test/*.test.mjs`. Expected: PASS.
- [ ] **Step 5: Commit** (`Apple Intelligence helper, live check and the panel check document`).

### Task 5: Documentation, gate and handoff

**Files:**
- Modify: `README.md`, `CHANGELOG.md`, `CLAUDE.md`, `docs/GETTING-STARTED.md`, `docs/SCENARIOS.md`, `docs/superpowers/plans/NEXT.md`

- [ ] **Step 1: Update the documents:** README status 0.3.0 (the panel with every road card and agent; the settings card), CHANGELOG `0.3.0 Panel`, CLAUDE.md (panel layout, the settings state kept apart from `pending`, the CLI exit-code rule, the layers rule, the no-HTML-sink rule), GETTING-STARTED (agents: the `claude` program, optional Apple helper, the providers file), SCENARIOS (remove the "panel not ported" note), and replace `NEXT.md` with the brief for Plan 4 (Track Layer and the rail kind: the registry, `set_feature`, schema-built cards, the standards-based library).
- [ ] **Step 2: Run the full gate:** `cargo +stable test --release --workspace`, `node --test panel/test/*.test.mjs scripts/test/*.test.mjs`, `node scripts/check.mjs`, `node scripts/standards.mjs check`, and `node scripts/parity.mjs <cli> <road_builder_dir>` (the road output must still be identical). Push the branch; confirm CI green on both systems.
- [ ] **Step 3: Open a draft pull request** to `main` listing the test totals and the calls below; do not merge. State clearly that the live check against a real VectorCraft is the owner's step.

## Calls made in this plan (check these)

- The road cards stay hand-written and ported as they are; they become schema-built when rail arrives (Plan 4).
- The settings card keeps its own state (`settingsPending`) and its own Draw and Cancel buttons, so a selection-driven proposal is never disturbed.
- `Panel` accepts missing providers (agent features answer with a plain message) so the settings tests and tests of other features need no agent.
- `demo/` (the recorded demo video script) is not ported; `live-check.mjs` and `examples/story1.svg` are.
- The Apple helper builds only on macOS with `swiftc`; the command was checked on this Mac (`swiftc -parse-as-library -O -o <out> panel/apple-fm/main.swift`; `--check` prints `{"available": true}`).
- A changed document title is how the panel tells documents apart (Plan 1's rule), so two documents with the same title and revision are not distinguished.
- Version `0.3.0`.

## Later plans

Plan 4: Track Layer and the rail kind (registry, `set_feature`, schema-built cards, standards-based library). Plan 5: styles and the target print scale. Plan 6: legacy `.pmap`, migration, release 1.0 and deprecation of the old repos.
