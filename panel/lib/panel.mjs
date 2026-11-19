// The panel's brain: follows the selection in VectorCraft, shows the selected road's description, turns words into a proposed
// change with an agent, and on Draw applies it (Describe Road, then Build Road) without losing the user's selection.
import { EventEmitter } from "node:events";
import { cardModel } from "./cards.mjs";
import { diffCrossSection, editCrossSection, parseCrossSection, parseCrossing, parseEmbankment, parseHub, parseJoint, parseJunction, parseSettings, parseSpec, parseTransition, parseWater, parseWet, findJoint, schema } from "./cli.mjs";
import { describeChanges, describeTrackChanges } from "./changes.mjs";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { PLUGIN_ID, SET_CROSSING, SET_CROSS_SECTION, SET_EMBANKMENT, SET_HUB, SET_SETTINGS, SET_TRANSITION, SET_JUNCTION, SET_JOINT, SET_LEVEL_CROSSING, SET_TRACKS, SET_WATER, SET_WET, SET_YARD, allPaths, classify, inspectState, junctionId, nodes, readSpec, segmentId, trackId, waterId } from "./doc.mjs";
import { organise, unlockOurs } from "./layers.mjs";
import { propose as proposeWith, proposeCrossing, proposeFeature, proposeHub, proposeJoint, proposeJunction, proposeTrack, proposeTransition } from "./providers.mjs";

const DEFAULT_TEXT = "lane back | lane fwd";
const DEFAULT_TRACK = { kind: "tracks", spec: { tracks: 1 } };
const NEEDS_MIX_NOTE = "Roads and tracks cannot be described together yet (not supported yet): select one or the other.";
const DEFAULT_CROSSING = { treatment: "signal", crosswalks: true, minor: null, over: null, diameter: 40, stub: "keep" };
const DEFAULT_HUB = { treatment: "signal", crosswalks: true, minor: null, stub: "keep" };
const CROSSING_KEYS = ["treatment", "crosswalks", "radius", "minor", "over", "main", "diameter", "stub", "pocket", "pocket_length", "pocket_taper", "slip", "slip_radius", "opening", "gore", "gore_length", "accel_length", "accel_taper", "beacon", "gates"];
const DEFAULT_TRANSITION = { change: "add", side: "right", direction: "fwd", count: 1 };
const HUB_KEYS = ["treatment", "crosswalks", "radius", "minor", "stub"];
const DEFAULT_WATER = { kind: "auto", width: 8, flow: "fwd", side: "right", bank_left: 0, bank_right: 0, slope: 2 };
const WATER_KEYS = ["kind", "name", "width", "flow", "speed", "side", "bank_left", "bank_right", "slope"];
const DEFAULT_EMBANK = { height: 2, slope: 2, sides: "both", ramp: 15 };
const EMBANK_KEYS = ["height", "slope", "from", "to", "sides", "ramp"];
const DEFAULT_WET = { treatment: "bridge" };
const WET_KEYS = ["treatment", "deck_height", "abutment", "shape", "count", "size", "fill_height", "slope"];

/** A description's fields with the empty ones left out, and "kind: auto" (the shape decides) dropped, ready to send to the plug-in. */
const tidy = (spec) => Object.fromEntries(Object.entries(spec).filter(([k, v]) => v !== null && v !== undefined && v !== "" && !(k === "kind" && v === "auto")));

export class Panel extends EventEmitter {
  constructor({ control, providers }) {
    super();
    this.control = control;
    this.providers = providers ?? { default: null, providers: {} };
    this.provider = this.providers.default;
    this.state = { connected: false, error: null, doc: null, selection: [], notes: [], target: null, road: null, track: null, joint: null, level: null, junction: null, crossing: null, hub: null, water: null, wet: null, embank: null, settings: null, settingsPending: null, mode: "road", pending: null, busy: null, message: null };
    this.last = { revision: null, selection: "", title: null };
    this.schema = null;
    this.settingsOpen = false; // the settings card is read from a saved copy only while it is open
    this.spec = null;
    this.queue = Promise.resolve();
    this.timer = null;
  }

  /** Runs `fn` after everything queued before it (the poll and user actions must not overlap). */
  #serial(fn) {
    const run = this.queue.then(fn, fn);
    this.queue = run.catch(() => {});
    return run;
  }

  start(intervalMs = 400) {
    this.timer = setInterval(() => this.tick(), intervalMs);
    return this.tick();
  }

  stop() {
    clearInterval(this.timer);
  }

  tick(force = false) {
    if (this.state.busy) return Promise.resolve();
    return this.#serial(() => this.#refresh(force));
  }

  #set(patch) {
    const before = JSON.stringify(this.snapshot());
    Object.assign(this.state, patch);
    if (JSON.stringify(this.snapshot()) !== before) this.emit("change", this.snapshot());
  }

  /** Reads the document again; a failure the page cannot show otherwise (the helper crashed) becomes its error line, and the next poll tries again. */
  async #refresh(force) {
    try {
      await this.#refreshInner(force);
    } catch (e) {
      this.last = { revision: null, selection: "", title: null };
      this.#set({ connected: this.control.connected, error: e.message });
    }
  }

  async #refreshInner(force) {
    let s;
    try {
      s = await inspectState(this.control);
    } catch (e) {
      const noDoc = /no (active )?document/i.test(e.message);
      this.last = { revision: null, selection: "", title: null };
      return this.#set({ connected: !noDoc && this.control.connected, error: noDoc ? "Open or create a document in VectorCraft." : e.message, doc: null, selection: [], target: null, road: null, track: null, joint: null, level: null, junction: null, crossing: null, hub: null, water: null, wet: null, embank: null, settings: null, settingsPending: null, pending: null });
    }
    const key = s.selection.join(",");
    // A different document (its title changed) is read again and drops every proposal made for the old one.
    const switched = s.title !== this.last.title;
    const settingsStale = switched || s.revision !== this.last.revision || !this.state.settings;
    const changed = force || switched || key !== this.last.selection || s.revision !== this.last.revision;
    if (!changed) return this.#set({ connected: true, error: null });
    const selectionChanged = key !== this.last.selection;
    this.last = { revision: s.revision, selection: key, title: s.title };
    if (switched) this.spec = null; // the old document's build notes must not show in the new one

    let sel = (await nodes(this.control, s.selection)).map((n) => ({ id: n.id, name: n.name ?? "", kind: n.kind, anchors: n.anchors, ...classify(n) }));
    // Clicking anywhere on a built road selects one of its generated shapes. For the panel that means "this road": swap each shape for the
    // line it was built from (the shape's piece number is the Nth line of that road, in document order).
    if (sel.some((n) => /-shape$/.test(n.role))) {
      const paths = await allPaths(this.control);
      const seen = new Set();
      const road = (id, piece = 1) => paths.filter((p) => segmentId(p.name) === id)[piece - 1];
      sel = (await Promise.all(sel.map(async (n) => {
        if (!/-shape$/.test(n.role)) return [n];
        let spines = [];
        if (n.role === "road-shape") spines = [road(n.segment, n.piece)];
        else if (n.role === "junction-shape") spines = [paths.find((p) => junctionId(p.name) === n.segment)];
        else if (n.role === "track-shape") spines = [paths.find((p) => trackId(p.name) === n.track)];
        else if (n.role === "crossing-shape") spines = ((this.spec = await readSpec(this.control)).crossings?.[n.segment]?.roads ?? []).map((r) => road(r));
        else if (n.role === "water-shape") spines = [paths.find((p) => waterId(p.name) === n.segment)];
        else if (n.role === "wet-shape") {
          const w = (this.spec = await readSpec(this.control)).wets?.[n.segment];
          spines = [road(w?.road), paths.find((p) => waterId(p.name) === w?.water)];
        } else spines = ((this.spec = await readSpec(this.control)).hubs?.[n.segment]?.roads ?? []).map((r) => road(r));
        return spines.filter((p) => p && !seen.has(p.id)).map((p) => (seen.add(p.id), { id: p.id, name: p.name, kind: "Path", role: trackId(p.name) ? "track" : waterId(p.name) ? "water" : junctionId(p.name) ? "junction" : "road", segment: segmentId(p.name) ?? junctionId(p.name) ?? waterId(p.name), track: trackId(p.name) ?? undefined, viaShape: true }));
      }))).flat();
    }
    const segments = [...new Set(sel.filter((n) => n.role === "road").map((n) => n.segment))];
    let target = null;
    const waters = [...new Set(sel.filter((n) => n.role === "water").map((n) => n.segment))];
    const trackLines = sel.filter((n) => n.role === "track");
    const trackIds = [...new Set(trackLines.map((n) => trackId(n.name)))];
    const allTrack = trackLines.length > 0 && trackLines.length === sel.length;
    // A track line mixed with a road, junction or water line is not something the panel can describe yet.
    // One road line and one track line: a level crossing. Any other mix of tracks with roads, junctions or water is not supported yet.
    const roadIds = [...new Set(sel.filter((n) => n.role === "road").map((n) => n.segment))];
    const level = trackIds.length === 1 && roadIds.length === 1 && sel.every((n) => n.role === "road" || n.role === "track");
    const mixed = !level && trackLines.length > 0 && sel.some((n) => ["road", "junction", "water"].includes(n.role));
    if (level) target = { kind: "level", road: roadIds[0], track: trackIds[0], ids: sel.map((n) => n.id) };
    else if (allTrack && trackIds.length === 1) target = { kind: "track", id: trackIds[0], ids: sel.map((n) => n.id) };
    else if (allTrack && trackIds.length === 2) target = { kind: "joint", lines: [...trackIds].sort(), ids: sel.map((n) => n.id) };
    else if (waters.length === 1 && !segments.length && sel.every((n) => n.role === "water")) target = { kind: "water", segment: waters[0], ids: sel.map((n) => n.id) };
    else if (waters.length === 1 && segments.length === 1 && sel.every((n) => n.role === "road" || n.role === "water")) target = { kind: "wet", road: segments[0], water: waters[0], ids: sel.map((n) => n.id) };
    else if (segments.length === 1 && sel.every((n) => n.role === "road")) target = { kind: "road", segment: segments[0], ids: sel.map((n) => n.id) };
    else if (segments.length === 2 && sel.every((n) => n.role === "road")) target = { kind: "crossing", roads: [...segments].sort(), ids: sel.map((n) => n.id) };
    else if (segments.length >= 3 && sel.every((n) => n.role === "road")) target = { kind: "hub", roads: [...segments].sort(), ids: sel.map((n) => n.id) };
    else if (sel.every((n) => n.role === "junction") && sel.length && new Set(sel.map((n) => n.segment)).size === 1) target = { kind: "junction", segment: sel[0].segment, ids: sel.map((n) => n.id) };
    else if (sel.length === 1 && sel[0].role === "line") target = { kind: "line", ids: [sel[0].id] };
    // A plain line (or shape) with one track: the user wants a crossing, but the line must be a road first.
    else if (trackIds.length === 1 && sel.every((n) => n.role === "track" || n.role === "line")) target = { kind: "other", plainWithTrack: true, ids: sel.map((n) => n.id) };
    else if (sel.length) target = { kind: "other", ids: sel.map((n) => n.id) };

    // A track (or a plain line described as one): what is stored for it, checked by the CLI, and the card built from the schema.
    let track = null;
    if (target?.kind === "track" || (target?.kind === "line" && this.state.mode === "track")) {
      this.spec = await readSpec(this.control);
      const yard = target.id ? this.spec.yards?.[target.id] : null;
      const stored = yard ? { kind: "yard", spec: yard } : target.id && this.spec.tracks?.[target.id] ? { kind: "tracks", spec: this.spec.tracks[target.id] } : null;
      const shown = stored ?? DEFAULT_TRACK;
      const parsed = await parseSpec(shown.kind, shown.spec);
      const spec = parsed.ok ? parsed.spec : shown.spec;
      track = { id: target.id ?? null, described: !!stored, kind: shown.kind, spec, summary: parsed.ok ? parsed.summary : "", min_line_m: parsed.min_line_m ?? null, card: await this.#card(shown.kind, spec) };
    }
    // Two track lines selected: how do they meet, and is there a joint stored for them?
    let joint = null;
    if (target?.kind === "joint") {
      this.spec = await readSpec(this.control);
      const [a, b] = target.lines;
      const found = this.spec.lines?.[a] && this.spec.lines?.[b] ? await findJoint(this.spec.lines[a], this.spec.lines[b], a, b) : { ok: false, error: "The panel could not read both lines from the document." };
      const stored = Object.entries(this.spec.joints ?? {}).find(([, j]) => j.lines?.length === 2 && j.lines.includes(a) && j.lines.includes(b));
      const parsed = stored ? await parseJoint(JSON.stringify(stored[1])) : null;
      const spec = parsed?.ok ? parsed.joint : { kind: found.ok ? found.kind : "crossing" };
      joint = { id: stored?.[0] ?? null, lines: [a, b], found, described: !!stored, spec, summary: parsed?.ok ? parsed.summary : "", card: await this.#card("joint", spec) };
    }
    // A road and a track: the stored level crossing between them, or the default, and its card.
    let levelState = null;
    if (target?.kind === "level") {
      this.spec = await readSpec(this.control);
      const stored = Object.entries(this.spec.level_crossings ?? {}).find(([, v]) => v.road === target.road && v.track === target.track);
      const spec = { treatment: stored?.[1]?.treatment ?? "level" };
      levelState = { id: stored?.[0] ?? null, road: target.road, track: target.track, described: !!stored, spec, card: await this.#card("level_crossing", spec) };
    }
    let road = null;
    if (target?.kind === "road") {
      this.spec = await readSpec(this.control);
      const text = this.spec.segments?.[target.segment]?.cross_section ?? null;
      const parsed = await parseCrossSection(text ?? DEFAULT_TEXT);
      const total = (await allPaths(this.control)).filter((p) => p.name === `road.${target.segment}` || (target.segment === "main" && p.name === "road")).length;
      road = { segment: target.segment, described: text !== null, text: text ?? DEFAULT_TEXT, items: parsed.items ?? [], total_width: parsed.total_width, anchor: this.spec.segments?.[target.segment]?.anchor ?? "auto", pieces: { selected: target.ids.length, total } };
      // Which line of the road is selected (for a lane change: transitions belong to one line), and the lane changes stored for it.
      const lines = (await allPaths(this.control)).filter((p) => segmentId(p.name) === target.segment);
      road.piece = target.ids.length === 1 ? lines.findIndex((p) => p.id === target.ids[0]) + 1 || null : null;
      road.transitions = [];
      for (const e of this.spec.transitions?.[target.segment] ?? []) {
        if (road.piece && (e.piece ?? 1) !== road.piece) continue;
        const r = await parseTransition(JSON.stringify(e), road.text);
        road.transitions.push({ spec: e, summary: r.ok ? r.summary : "", after: r.ok ? r.after : "" });
      }
    }
    let junction = null;
    if (target?.kind === "junction") {
      this.spec = await readSpec(this.control);
      const stored = this.spec.junctions?.[target.segment] ?? null;
      const parsed = stored ? await parseJunction(JSON.stringify(stored)) : null;
      junction = { id: target.segment, described: !!stored, spec: parsed?.ok ? parsed.junction : { legs: [], control: "none", crosswalks: false, radius: 6, kind: "match" }, summary: parsed?.ok ? parsed.summary : "road carried across the gap (not described yet)" };
    }
    let crossing = null;
    if (target?.kind === "crossing") {
      this.spec = await readSpec(this.control);
      const [a, b] = target.roads;
      const stored = Object.entries(this.spec.crossings ?? {}).find(([, c]) => c.roads?.length === 2 && c.roads.includes(a) && c.roads.includes(b));
      const parsed = await parseCrossing(JSON.stringify({ ...(stored?.[1] ?? DEFAULT_CROSSING), roads: [a, b] }));
      crossing = { id: stored?.[0] ?? null, roads: [a, b], described: !!stored, spec: parsed.ok ? parsed.crossing : { ...DEFAULT_CROSSING, roads: [a, b] }, summary: parsed.ok ? parsed.summary : "" };
    }
    let hub = null;
    if (target?.kind === "hub") {
      this.spec = await readSpec(this.control);
      const same = (h) => h.roads?.length === target.roads.length && target.roads.every((r) => h.roads.includes(r));
      const stored = Object.entries(this.spec.hubs ?? {}).find(([, h]) => same(h));
      const parsed = await parseHub(JSON.stringify({ ...(stored?.[1] ?? DEFAULT_HUB), roads: target.roads }));
      hub = { id: stored?.[0] ?? null, roads: target.roads, described: !!stored, spec: parsed.ok ? parsed.hub : { ...DEFAULT_HUB, roads: target.roads }, summary: parsed.ok ? parsed.summary : "" };
    }
    let water = null;
    if (target?.kind === "water" || (target?.kind === "line" && this.state.mode === "water")) {
      this.spec = await readSpec(this.control);
      const stored = target.kind === "water" ? this.spec.waters?.[target.segment] ?? null : null;
      const parsed = stored ? await parseWater(JSON.stringify(stored)) : null;
      water = { id: target.segment ?? null, described: !!stored, spec: parsed?.ok ? parsed.water : DEFAULT_WATER, summary: parsed?.ok ? parsed.summary : "not described yet: a closed shape is still water, an open line a creek or river" };
    }
    let wet = null;
    if (target?.kind === "wet") {
      this.spec = await readSpec(this.control);
      const stored = Object.entries(this.spec.wets ?? {}).find(([, w]) => w.road === target.road && w.water === target.water);
      const parsed = await parseWet(JSON.stringify({ ...(stored?.[1] ?? DEFAULT_WET), road: target.road, water: target.water }));
      wet = { id: stored?.[0] ?? null, road: target.road, water: target.water, described: !!stored, spec: parsed.ok ? parsed.wet : { ...DEFAULT_WET, road: target.road, water: target.water }, summary: parsed.ok ? parsed.summary : parsed.error };
    }
    let embank = null;
    if (target?.kind === "road" && this.state.mode === "fill" && road?.piece) {
      embank = { piece: road.piece, list: [] };
      for (const e of this.spec.embankments?.[target.segment] ?? []) {
        if ((e.piece ?? 1) !== road.piece) continue;
        const r = await parseEmbankment(JSON.stringify(e));
        embank.list.push({ spec: e, summary: r.ok ? r.summary : "" });
      }
    }
    // A proposal belongs to what was selected when it was made.
    const keep = this.state.pending && !selectionChanged && !switched;
    // The document settings card: read only while it is open, and only when the document or its revision changed.
    const reread = this.settingsOpen && (force || settingsStale);
    const settings = reread ? await this.#readSettings() : this.state.settings;
    // A settings proposal is dropped when the stored settings are no longer the ones it was made from (another document, an undo).
    let settingsPending = switched ? null : this.state.settingsPending;
    if (reread && settingsPending && JSON.stringify(settings?.values) !== JSON.stringify(settingsPending.base)) settingsPending = null;
    this.#set({ connected: true, error: null, doc: { title: s.title, revision: s.revision }, notes: this.spec?.notes ?? [], selection: sel, target, road, track, joint, level: levelState, junction, crossing, hub, water, wet, embank, settings, settingsPending, pending: keep ? this.state.pending : null, message: mixed ? NEEDS_MIX_NOTE : keep ? this.state.message : null });
  }

  /** Words to a proposed cross-section (not applied until Draw). */
  propose(description, providerId) {
    return this.#serial(async () => {
      if (!Object.keys(this.providers.providers).length) return this.#set({ message: "No agent is set up. See docs/GETTING-STARTED.md (agents)." });
      const t = this.state.target;
      if (!t || t.kind === "other") return this.#set({ message: "Select one line (or one road) first." });
      const first = providerId ?? this.provider;
      if (!this.providers.providers[first]) return this.#set({ message: `Unknown agent "${providerId}".` });
      // The chosen agent first; if it cannot give a valid answer, the one named in its `fallback`.
      const order = [first];
      const fb = this.providers.providers[first].fallback;
      if (fb && fb !== first && this.providers.providers[fb] && this.#available(this.providers.providers[fb])) order.push(fb);
      this.#set({ busy: "thinking", message: null });
      try {
        let failure = null;
        for (const [n, id] of order.entries()) {
          const r = await this.#ask(t, description, this.providers.providers[id], id);
          if (r.pending) {
            const label = (x) => this.providers.providers[x].label ?? x;
            const said = r.message ?? (n > 0 ? null : null);
            return this.#set({ pending: r.pending, message: n > 0 ? `${label(first)} could not give a valid answer, so ${label(id)} answered.${said ? ` ${said}` : ""}` : said ?? null });
          }
          failure = r.message;
        }
        this.#set({ message: failure });
      } catch (e) {
        this.#set({ message: e.message });
      } finally {
        this.#set({ busy: null });
      }
    });
  }

  /** The card for a kind of description ("tracks", "yard", "joint"), built from the CLI's schema (read once). */
  async #card(kind, values) {
    this.schemas ??= {};
    this.schemas[kind] ??= await schema(kind);
    return cardModel(this.schemas[kind], values);
  }

  #isTrack(t) {
    return t?.kind === "track" || (t?.kind === "line" && this.state.mode === "track");
  }

  /** One agent's attempt for the current target: {pending, message?} when it gave a valid proposal, else {message}. */
  async #ask(t, description, provider, id) {
    const base = { source: "agent", agent: id, description };
    if (this.#isTrack(t)) {
      const tr = this.state.track;
      const current = tr?.described ? { kind: tr.kind, spec: tr.spec } : null;
      const r = await proposeTrack({ description, current, provider });
      return r.ok ? { pending: { kind: r.kind, spec: r.spec, summary: r.summary, min_line_m: r.min_line_m ?? null, changes: describeTrackChanges(current, r), ...base } } : { message: r.error };
    }
    if (t.kind === "joint") {
      const j = this.state.joint;
      const r = await proposeJoint({ description, current: j.described ? j.spec : null, lines: j.lines, found: j.found, provider });
      if (!r.ok) return { message: r.error };
      const before = j.described ? j.spec : null;
      const changes = !before ? [`new: ${r.summary}`] : JSON.stringify(before) === JSON.stringify(r.spec) ? ["no change"] : [`from ${j.summary} to ${r.summary}`];
      return { pending: { kind: "joint", spec: r.spec, summary: r.summary, changes, ...base } };
    }
    if (this.#isWater(t)) {
      const w = this.state.water;
      const r = await proposeFeature({ kind: "water", description, current: w?.described ? w.spec : null, provider });
      return r.ok ? { pending: { kind: "water", water: r.water, summary: r.summary, changes: describeChanges(w?.described ? w.spec : null, r.water, "water feature"), ...base } } : { message: r.error };
    }
    if (t.kind === "wet") {
      const w = this.state.wet;
      const extra = { road: w.road, water: w.water };
      const r = await proposeFeature({ kind: "wet", description, current: w.described ? w.spec : null, extra, head: `The road is "${w.road}" and the water feature is "${w.water}".`, provider });
      return r.ok ? { pending: { kind: "wet", wet: r.wet, summary: r.summary, changes: describeChanges(w.described ? w.spec : null, r.wet, "water crossing"), ...base } } : { message: r.error };
    }
    if (t.kind === "road" && this.state.mode === "fill") {
      const draft = this.state.pending?.kind === "embank" ? this.state.pending.embank : null;
      const r = await proposeFeature({ kind: "embank", description, current: draft, provider });
      return r.ok ? { pending: { kind: "embank", embank: { ...r.embankment, piece: this.state.road.piece ?? 1 }, summary: r.summary, changes: [r.summary], ...base } } : { message: r.error };
    }
    if (t.kind === "hub") {
      const h = this.state.hub;
      const r = await proposeHub({ description, current: h?.described ? h.spec : null, roads: h.roads, provider });
      return r.ok ? { pending: { kind: "hub", hub: r.hub, summary: r.summary, changes: describeChanges(h?.described ? h.spec : null, r.hub), ...base } } : { message: r.error };
    }
    if (t.kind === "crossing") {
      const c = this.state.crossing;
      const r = await proposeCrossing({ description, current: c?.described ? c.spec : null, roads: c.roads, provider });
      return r.ok ? { pending: { kind: "crossing", crossing: r.crossing, summary: r.summary, changes: describeChanges(c?.described ? c.spec : null, r.crossing), ...base } } : { message: r.error };
    }
    if (t.kind === "junction" || (t.kind === "line" && this.state.mode === "connection")) {
      const j = this.state.junction;
      const current = j?.described ? j.spec : null;
      const r = await proposeJunction({ description, current, provider });
      return r.ok ? { pending: { kind: "junction", junction: r.junction, summary: r.summary, changes: describeChanges(current, r.junction, "junction"), ...base } } : { message: r.error };
    }
    if (t.kind === "road" && this.state.mode === "transition") {
      const road = this.state.road;
      const draft = this.state.pending?.kind === "transition" ? this.state.pending.transition : null;
      const r = await proposeTransition({ description, current: draft, crossSection: road.text, provider });
      return r.ok ? { pending: { kind: "transition", transition: { ...r.transition, piece: road.piece ?? 1 }, summary: r.summary, after: r.after, changes: [r.summary, `after it the road is: ${r.after}`], ...base } } : { message: r.error };
    }
    const current = this.state.road?.described ? this.state.road.text : null;
    const r = await proposeWith({ description, current, provider });
    if (!r.ok) return { message: r.error };
    return { pending: { ...r, changes: await this.#roadChanges(current, r), ...base }, message: r.notes?.length ? `The agent read: ${r.notes.join("; ")}` : null };
  }

  /** Plain words for what a proposed cross-section changes compared with the current one (or what it is, for a new road). */
  async #roadChanges(current, proposed) {
    if (!current) return [`a new road: ${proposed.items?.length ?? 0} elements, ${proposed.total_width} m wide`];
    const d = await diffCrossSection(current, proposed.canonical);
    return d.ok ? (d.changes.length ? d.changes : ["no change"]) : [];
  }

  /** Is an agent usable here? An optional command-line agent is hidden when its program is not installed. */
  #available(p) {
    if (!p.optional || p.kind !== "cli" || !p.command) return true;
    return existsSync(resolve(import.meta.dirname, "../..", p.command)) || existsSync(p.command);
  }

  /** Edits the proposed (or current) cross-section: ["delete", i] | ["move", from, to] | ["width", i, m] | ["insert", i, "text"]. */
  edit(op) {
    return this.#serial(async () => {
      const base = this.state.pending?.canonical ?? this.state.road?.text;
      if (!base) return this.#set({ message: "Select a road or a line first." });
      const r = await editCrossSection(base, op);
      this.#set(r.ok ? { pending: { ...r, source: "edit", changes: await this.#roadChanges(this.state.road?.described ? this.state.road.text : null, r) }, message: null } : { message: r.error });
    });
  }

  /** The user typed the shorthand themselves. */
  setText(text) {
    return this.#serial(async () => {
      const r = await parseCrossSection(text);
      this.#set(r.ok ? { pending: { ...r, source: "typed", changes: await this.#roadChanges(this.state.road?.described ? this.state.road.text : null, r) }, message: r.notes?.length ? `Read as: ${r.notes.join("; ")}` : null } : { message: r.error });
    });
  }

  /** Puts the road's (or junction's) own line in VectorCraft's selection, ready to move, reshape or delete. */
  selectLine() {
    return this.#serial(async () => {
      const t = this.state.target;
      if (!t || (t.kind !== "road" && t.kind !== "junction" && t.kind !== "track")) return this.#set({ message: "Select a road, a junction or a track first." });
      await this.control.run("select.set", { ids: t.ids });
      await this.#refresh(true);
      this.#set({ message: `The line is selected in VectorCraft: move or reshape its points to change the ${t.kind === "track" ? "track" : "road"}.` });
    });
  }

  /** What a plain line is: a road, a connection between two roads, water, or railway track. */
  setMode(mode) {
    if (mode !== "road" && mode !== "connection" && mode !== "transition" && mode !== "water" && mode !== "fill" && mode !== "track") return false;
    const was = this.state.mode;
    this.#set({ mode, pending: null, message: null });
    // A plain line described as track shows the track card, which is read from the document: look again now.
    if (mode === "track" || was === "track") this.tick(true);
    return true;
  }

  /** "Connect and match": the line just carries the road across the gap, no side roads. */
  match() {
    return this.#serial(async () => {
      const r = await parseJunction(JSON.stringify({ legs: [], control: "none", crosswalks: false }));
      this.#set({ pending: { kind: "junction", junction: r.junction, summary: r.summary, source: "match" }, message: null });
    });
  }

  /** Quick choices for the selected track or yard: any field of its card ({tracks, spacing, ballast: "on"|"off", mains, left, …}). An empty value leaves the field out. Checked by the CLI first; nothing is applied until Draw. */
  editTrack(patch) {
    return this.#serial(async () => {
      const tr = this.state.track;
      if (!tr || !this.#isTrack(this.state.target)) return this.#set({ message: "Select a track line first." });
      const pending = ["tracks", "yard"].includes(this.state.pending?.kind) ? this.state.pending : null;
      const base = pending ?? { kind: tr.kind, spec: tr.spec };
      const next = { ...base.spec };
      for (const [k, v] of Object.entries(patch)) next[k] = v === "on" ? true : v === "off" ? false : v;
      for (const k of Object.keys(next)) if (next[k] === null || next[k] === undefined || next[k] === "") delete next[k];
      const r = await parseSpec(base.kind, next);
      this.#set(r.ok ? { pending: { kind: r.kind, spec: r.spec, summary: r.summary, min_line_m: r.min_line_m ?? null, source: "edit", changes: describeTrackChanges({ kind: tr.kind, spec: tr.spec }, r) }, message: null } : { message: r.error });
    });
  }

  /** The level crossing's card: {treatment}. Checked against the card's own choices; nothing is applied until Draw. */
  editLevel(patch) {
    return this.#serial(async () => {
      const l = this.state.level;
      if (!l || this.state.target?.kind !== "level") return this.#set({ message: "Select one road line and one track line first." });
      const options = l.card.fields.find((f) => f.key === "treatment")?.options ?? [];
      const treatment = patch.treatment ?? (this.state.pending?.kind === "level" ? this.state.pending.spec.treatment : l.spec.treatment);
      if (!options.includes(treatment)) return this.#set({ message: `The crossing must be ${options.join(" or ")}.` });
      const summary = CROSSING_WORDS[treatment] ?? "a level crossing";
      const changes = l.described ? (treatment === l.spec.treatment ? ["no change"] : [`from ${l.spec.treatment} to ${treatment}`]) : [`new: ${summary}`];
      this.#set({ pending: { kind: "level", spec: { treatment }, summary, source: "edit", changes }, message: null });
    });
  }

  /** Quick choices for the selected joint: any of {kind, main, frog}. Checked by the CLI first; nothing is applied until Draw. */
  editJoint(patch) {
    return this.#serial(async () => {
      const j = this.state.joint;
      if (!j || this.state.target?.kind !== "joint") return this.#set({ message: "Select the two track lines first." });
      const base = this.state.pending?.kind === "joint" ? this.state.pending.spec : j.spec;
      const next = { ...base, ...patch };
      delete next.lines;
      for (const k of Object.keys(next)) if (next[k] === null || next[k] === undefined || next[k] === "") delete next[k];
      const r = await parseJoint(JSON.stringify(next));
      if (!r.ok) return this.#set({ message: r.error });
      if (r.joint.main && !j.lines.includes(r.joint.main)) return this.#set({ message: `\`main\` must be one of the two lines, ${j.lines[0]} or ${j.lines[1]}.` });
      const before = j.described ? j.spec : null;
      const changes = !before ? [`new: ${r.summary}`] : JSON.stringify(before) === JSON.stringify(r.joint) ? ["no change"] : [`from ${j.summary} to ${r.summary}`];
      this.#set({ pending: { kind: "joint", spec: r.joint, summary: r.summary, source: "edit", changes }, message: null });
    });
  }

  /** Adjusts the proposed (or current) junction: {control, crosswalks, radius} or {removeLeg: i}. */
  editJunction(patch) {
    return this.#serial(async () => {
      const base = this.state.pending?.junction ?? this.state.junction?.spec;
      if (!base) return this.#set({ message: "Select a junction or a connecting line first." });
      const next = { ...base, legs: [...(base.legs ?? [])] };
      for (const k of ["control", "crosswalks", "radius"]) if (patch[k] !== undefined) next[k] = patch[k];
      if (Number.isInteger(patch.removeLeg)) next.legs.splice(patch.removeLeg, 1);
      const r = await parseJunction(JSON.stringify(next));
      this.#set(r.ok ? { pending: { kind: "junction", junction: r.junction, summary: r.summary, source: "edit", changes: describeChanges(this.state.junction?.described ? this.state.junction.spec : null, r.junction, "junction") }, message: null } : { message: r.error });
    });
  }

  /** Quick choices for the selected crossing: {treatment, crosswalks, pocket, slip, …: any field of an intersection description}. Starts a proposal from the current (or default) description. */
  editCrossing(patch) {
    return this.#serial(async () => {
      const base = this.state.pending?.crossing ?? this.state.crossing?.spec;
      if (!base || this.state.target?.kind !== "crossing") return this.#set({ message: "Select the two crossing roads first." });
      const next = { ...base, roads: this.state.crossing.roads };
      for (const k of CROSSING_KEYS) if (patch[k] !== undefined) next[k] = patch[k];
      const r = await parseCrossing(JSON.stringify(next));
      const stored = this.state.crossing.described ? this.state.crossing.spec : null;
      this.#set(r.ok ? { pending: { kind: "crossing", crossing: r.crossing, summary: r.summary, source: "edit", changes: describeChanges(stored, r.crossing) }, message: null } : { message: r.error });
    });
  }

  /** Quick choices for a lane change on the selected line: {change, side, direction, count, at, length, speed}. Starts from the draft (or a default). */
  editTransition(patch) {
    return this.#serial(async () => {
      const road = this.state.road;
      if (this.state.target?.kind !== "road" || !road?.piece) return this.#set({ message: "Select one line of one road first." });
      const base = this.state.pending?.kind === "transition" ? this.state.pending.transition : DEFAULT_TRANSITION;
      const next = { ...base, ...patch, piece: road.piece };
      for (const k of ["at", "length"]) if (next[k] === "" || Number.isNaN(next[k])) next[k] = null;
      const r = await parseTransition(JSON.stringify(next), road.text);
      this.#set(r.ok ? { pending: { kind: "transition", transition: r.transition, summary: r.summary, after: r.after, changes: [r.summary, `after it the road is: ${r.after}`], source: "edit" }, message: null } : { message: r.error });
    });
  }

  /** Takes away the `index`th lane change stored for the selected line. */
  removeTransition(index) {
    return this.#serial(async () => {
      const t = this.state.target;
      const e = this.state.road?.transitions?.[index];
      if (t?.kind !== "road" || !e) return this.#set({ message: "That lane change is not there any more." });
      this.#set({ busy: "drawing", message: null });
      try {
        await this.control.run("select.set", { ids: t.ids });
        await this.control.run(SET_TRANSITION, { params: { description: JSON.stringify({ ...e.spec, remove: true }) } });
        await this.#build(t.ids);
        this.spec = await readSpec(this.control);
        this.#set({ pending: null, notes: this.spec.notes ?? [], message: "Lane change removed." });
      } catch (err) {
        this.#set({ message: `Could not remove it: ${err.message}` });
      } finally {
        this.#set({ busy: null });
        await this.#refresh(true);
      }
    });
  }

  /** Quick choices for three or more selected roads: {treatment, crosswalks, radius, minor, stub}. */
  editHub(patch) {
    return this.#serial(async () => {
      const base = this.state.pending?.hub ?? this.state.hub?.spec;
      if (!base || this.state.target?.kind !== "hub") return this.#set({ message: "Select the roads that meet first." });
      const next = { ...base, roads: this.state.hub.roads };
      for (const k of HUB_KEYS) if (patch[k] !== undefined) next[k] = patch[k];
      const r = await parseHub(JSON.stringify(next));
      const stored = this.state.hub.described ? this.state.hub.spec : null;
      this.#set(r.ok ? { pending: { kind: "hub", hub: r.hub, summary: r.summary, source: "edit", changes: describeChanges(stored, r.hub) }, message: null } : { message: r.error });
    });
  }

  #isWater(t) {
    return t?.kind === "water" || (t?.kind === "line" && this.state.mode === "water");
  }

  /** Quick choices for the selected water: any of {kind, name, width, flow, speed, side, bank_left, bank_right, slope}. Empty (null) numbers are left out. */
  editWater(patch) {
    return this.#serial(async () => {
      const base = this.state.pending?.kind === "water" ? this.state.pending.water : this.state.water?.spec;
      if (!base || !this.#isWater(this.state.target)) return this.#set({ message: "Select a water line or shape first." });
      const next = { ...base };
      for (const k of WATER_KEYS) if (patch[k] !== undefined) next[k] = patch[k];
      const sent = tidy(next);
      const r = sent.kind ? await parseWater(JSON.stringify(sent)) : { ok: true, water: next, summary: "water (the kind follows the shape: a closed shape is still water, an open line a creek or river)" };
      const stored = this.state.water.described ? this.state.water.spec : null;
      this.#set(r.ok ? { pending: { kind: "water", water: sent.kind ? r.water : next, summary: r.summary, source: "edit", changes: describeChanges(stored, sent.kind ? r.water : next, "water feature") }, message: null } : { message: r.error });
    });
  }

  /** Quick choices for the selected road and water: any of {treatment, deck_height, abutment, shape, count, size, fill_height, slope}. */
  editWet(patch) {
    return this.#serial(async () => {
      const w = this.state.wet;
      if (!w || this.state.target?.kind !== "wet") return this.#set({ message: "Select one line of a road and one water line first." });
      const next = { ...(this.state.pending?.kind === "wet" ? this.state.pending.wet : w.spec), road: w.road, water: w.water };
      for (const k of WET_KEYS) if (patch[k] !== undefined) next[k] = patch[k];
      const r = await parseWet(JSON.stringify(tidy(next)));
      this.#set(r.ok ? { pending: { kind: "wet", wet: r.wet, summary: r.summary, source: "edit", changes: describeChanges(w.described ? w.spec : null, r.wet, "water crossing") }, message: null } : { message: r.error });
    });
  }

  /** Quick choices for road on fill along the selected line: {height, slope, from, to, sides, ramp}. Starts from the draft (or a default). */
  editEmbank(patch) {
    return this.#serial(async () => {
      const road = this.state.road;
      if (this.state.target?.kind !== "road" || !road?.piece) return this.#set({ message: "Select one line of one road first." });
      const next = { ...(this.state.pending?.kind === "embank" ? this.state.pending.embank : DEFAULT_EMBANK), piece: road.piece };
      for (const k of EMBANK_KEYS) if (patch[k] !== undefined) next[k] = patch[k];
      const r = await parseEmbankment(JSON.stringify(tidy(next)));
      this.#set(r.ok ? { pending: { kind: "embank", embank: { ...r.embankment, piece: road.piece }, summary: r.summary, changes: [r.summary], source: "edit" }, message: null } : { message: r.error });
    });
  }

  /** Takes away the `index`th embankment stored for the selected line. */
  removeEmbank(index) {
    return this.#serial(async () => {
      const t = this.state.target;
      const e = this.state.embank?.list?.[index];
      if (t?.kind !== "road" || !e) return this.#set({ message: "That embankment is not there any more." });
      this.#set({ busy: "drawing", message: null });
      try {
        await this.control.run("select.set", { ids: t.ids });
        await this.control.run(SET_EMBANKMENT, { params: { description: JSON.stringify({ ...e.spec, remove: true }) } });
        await this.#build(t.ids);
        this.spec = await readSpec(this.control);
        this.#set({ pending: null, notes: this.spec.notes ?? [], message: "Embankment removed." });
      } catch (err) {
        this.#set({ message: `Could not remove it: ${err.message}` });
      } finally {
        this.#set({ busy: null });
        await this.#refresh(true);
      }
    });
  }

  /** The settings card's content from the saved copy: stored settings that cannot be read show the defaults and say why (`problem`). */
  async #readSettings() {
    try {
      this.schema ??= await schema("settings");
      const stored = (await readSpec(this.control)).settings ?? {};
      let parsed = await parseSettings(JSON.stringify(stored));
      let problem = null;
      if (!parsed.ok) {
        problem = `Stored settings cannot be read (${parsed.error}); showing the defaults. Draw to replace them.`;
        parsed = await parseSettings("{}");
      }
      return { values: parsed.settings, summary: parsed.summary, card: cardModel(this.schema, parsed.settings), problem };
    } catch (e) {
      return { values: {}, summary: "", card: { fields: [] }, problem: `The settings could not be read: ${e.message}` };
    }
  }

  /** Shows the settings card: reads the document's settings now, and keeps them current until `closeSettings`. */
  openSettings() {
    return this.#serial(async () => {
      this.settingsOpen = true;
      // Without a document there is nothing to read; the card fills in when one appears.
      this.#set({ settings: this.state.doc ? await this.#readSettings() : null });
    });
  }

  closeSettings() {
    return this.#serial(async () => {
      this.settingsOpen = false;
      this.#set({ settings: null, settingsPending: null });
    });
  }

  /** Quick choices for the document settings: any of {target_print_scale, style, seed, north_deg}. A change is checked by the CLI and previewed; nothing is applied until `drawSettings`. */
  editSettings(patch) {
    return this.#serial(async () => {
      const base = this.state.settingsPending?.settings ?? this.state.settings?.values;
      if (!base) return this.#set({ message: "Open or create a document in VectorCraft first." });
      // An emptied field keeps the stored value (null would otherwise mean "back to the default").
      const given = Object.fromEntries(Object.entries(patch).filter(([, v]) => v !== null && v !== undefined && v !== ""));
      const r = await parseSettings(JSON.stringify({ ...base, ...given }));
      // `base` is what the proposal was made from; Draw sends only what differs from it.
      this.#set(r.ok ? { settingsPending: { settings: r.settings, summary: r.summary, base: this.state.settingsPending?.base ?? base }, message: null } : { message: r.error });
    });
  }

  cancelSettings() {
    return this.#serial(async () => this.#set({ settingsPending: null, message: null }));
  }

  /** Applies the previewed settings with the plug-in's command, then reads the document again. */
  drawSettings() {
    return this.#serial(async () => {
      const pending = this.state.settingsPending;
      if (!pending) return this.#set({ message: "Nothing to change." });
      const changed = Object.fromEntries(Object.entries(pending.settings).filter(([k, v]) => JSON.stringify(v) !== JSON.stringify(pending.base?.[k])));
      if (!Object.keys(changed).length) return this.#set({ settingsPending: null, message: "Nothing to change." });
      const selected = this.state.selection.map((n) => n.id);
      this.#set({ busy: "drawing", message: null });
      let message = "Drawn.";
      let saved = false;
      try {
        await this.control.run("select.all");
        await this.control.run(SET_SETTINGS, { params: { description: JSON.stringify(changed) } });
        saved = true;
        // A new style or print scale is built at once (the design: changing it rebuilds everything), when there is anything to build.
        const paths = await allPaths(this.control);
        if (paths.some((p) => segmentId(p.name) || junctionId(p.name) || waterId(p.name) || trackId(p.name))) await this.#build(selected);
        else if (selected.length) await this.control.run("select.set", { ids: selected }); // the user's selection comes back
        this.#set({ settingsPending: null });
      } catch (e) {
        // Stored but not rebuilt: the proposal is done with; say what happened to each half.
        message = saved ? `Settings saved; the rebuild failed: ${e.message}` : `Could not draw: ${e.message}`;
        if (saved) this.#set({ settingsPending: null });
      } finally {
        this.#set({ busy: null });
        await this.#refresh(true);
        this.#set({ message });
      }
    });
  }

  cancel() {
    return this.#serial(async () => this.#set({ pending: null, message: null }));
  }

  /** Draw: apply the proposal (if any), then rebuild every road, and give the user their selection back. */
  draw({ scope } = {}) {
    return this.#serial(async () => {
      const t = this.state.target;
      const pending = this.state.pending;
      if (!t || t.kind === "other") return this.#set({ message: "Select one line (or one road) first." });
      this.#set({ busy: "drawing", message: null });
      let message = null;
      try {
        if (this.#isTrack(t)) {
          // A proposal, else (for a plain line, which must become a track) what the card shows; a track that is already described is only rebuilt.
          const chosen = ["tracks", "yard"].includes(pending?.kind) ? pending : t.kind === "line" ? this.state.track : null;
          if (chosen) {
            await this.control.run("select.set", { ids: t.ids });
            await this.control.run(chosen.kind === "yard" ? SET_YARD : SET_TRACKS, { params: { description: JSON.stringify(chosen.spec) } });
          }
        } else if (t.kind === "level") {
          // A proposal, else (for a road and track with nothing stored) what the card shows; a stored crossing is only rebuilt.
          const spec = pending?.kind === "level" ? pending.spec : this.state.level.described ? null : this.state.level.spec;
          if (spec) {
            await this.control.run("select.set", { ids: t.ids });
            await this.control.run(SET_LEVEL_CROSSING, { params: { description: JSON.stringify(spec) } });
          }
        } else if (t.kind === "joint") {
          // A proposal, else (for two lines with no stored joint) what the card shows; a stored joint is only rebuilt.
          const spec = pending?.kind === "joint" ? pending.spec : this.state.joint.described ? null : this.state.joint.spec;
          if (spec) {
            await this.control.run("select.set", { ids: t.ids });
            await this.control.run(SET_JOINT, { params: { description: JSON.stringify(spec) } });
          }
        } else if (this.#isWater(t)) {
          const spec = tidy(pending?.kind === "water" ? pending.water : this.state.water.spec);
          await this.control.run("select.set", { ids: t.ids });
          await this.control.run(SET_WATER, { params: { description: JSON.stringify(spec) } });
        } else if (t.kind === "wet") {
          const spec = tidy(pending?.kind === "wet" ? pending.wet : this.state.wet.spec);
          await this.control.run("select.set", { ids: t.ids });
          await this.control.run(SET_WET, { params: { description: JSON.stringify(spec) } });
        } else if (pending?.kind === "embank") {
          await this.control.run("select.set", { ids: t.ids });
          await this.control.run(SET_EMBANKMENT, { params: { description: JSON.stringify(tidy(pending.embank)) } });
        } else if (t.kind === "hub") {
          const spec = pending?.kind === "hub" ? pending.hub : this.state.hub.spec;
          await this.control.run("select.set", { ids: t.ids });
          await this.control.run(SET_HUB, { params: { description: JSON.stringify(spec) } });
        } else if (t.kind === "crossing") {
          const spec = pending?.kind === "crossing" ? pending.crossing : this.state.crossing.spec;
          await this.control.run("select.set", { ids: t.ids });
          await this.control.run(SET_CROSSING, { params: { description: JSON.stringify(spec) } });
        } else if (pending?.kind === "transition") {
          await this.control.run("select.set", { ids: t.ids });
          await this.control.run(SET_TRANSITION, { params: { description: JSON.stringify(pending.transition) } });
        } else if (pending?.kind === "junction") {
          await this.control.run("select.set", { ids: t.ids });
          await this.control.run(SET_JUNCTION, { params: { description: JSON.stringify(pending.junction) } });
        } else if (pending) {
          await this.control.run("select.set", { ids: t.ids });
          const params = { cross_section: pending.canonical };
          if (scope === "selected") params.scope = "selected";
          await this.control.run(SET_CROSS_SECTION, { params });
        }
        await this.#build(t.ids);
        this.spec = await readSpec(this.control);
        message = pending || ["crossing", "hub", "water", "wet"].includes(t.kind) ? "Drawn." : "Rebuilt.";
        this.#set({ pending: null, notes: this.spec.notes ?? [], message });
      } catch (e) {
        message = `Could not draw: ${e.message}`;
        this.#set({ message });
      } finally {
        this.#set({ busy: null });
        await this.#refresh(true);
        // Reading the document again clears a message that belongs to an older selection; this one is the answer to Draw, so say it again.
        if (message) this.#set({ message });
      }
    });
  }

  /** Rebuilds every road. Our layers are shown and unlocked for the build (a hidden spine would make its road vanish), the built shapes are
   *  sorted back into each road's layers, and the user's hiding and locking is put back. */
  async #build(restoreIds) {
    const restore = await unlockOurs(this.control);
    try {
      await this.control.run("select.all");
      await this.control.run("plugin.run", { id: PLUGIN_ID });
      await organise(this.control);
    } finally {
      await restore();
    }
    await this.control.run("select.set", { ids: restoreIds });
  }

  snapshot() {
    return { ...this.state, agents: { default: this.provider, list: Object.entries(this.providers.providers).filter(([, p]) => this.#available(p)).map(([id, p]) => ({ id, label: p.label ?? id, kind: p.kind })) } };
  }

  setAgent(id) {
    if (!this.providers.providers[id]) return false;
    this.provider = id;
    this.emit("change", this.snapshot());
    return true;
  }
}

/** How each treatment of a road meeting a track is said (the same words as panel/public/app.js). */
const CROSSING_WORDS = { level: "a level crossing", gated: "a gated level crossing", "road over": "a road bridge over the track", "track over": "a rail bridge over the road" };

