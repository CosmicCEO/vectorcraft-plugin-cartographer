// The panel's brain: follows the document in VectorCraft, shows the settings card, previews a change (checked by the native CLI), and on Draw
// applies it with the plug-in's command.
import { EventEmitter } from "node:events";
import { cardModel } from "./cards.mjs";
import { parseSettings, schema } from "./cli.mjs";
import { SET_SETTINGS, inspectState, readData } from "./doc.mjs";

export class Panel extends EventEmitter {
  constructor({ control }) {
    super();
    this.control = control;
    this.state = { connected: false, error: null, doc: null, settings: null, pending: null, busy: null, message: null };
    this.last = { revision: null };
    this.schema = null;
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

  snapshot() {
    return { ...this.state };
  }

  async #refresh(force) {
    let s;
    try {
      s = await inspectState(this.control);
    } catch (e) {
      const noDoc = /no (active )?document/i.test(e.message);
      this.last = { revision: null };
      return this.#set({ connected: !noDoc && this.control.connected, error: noDoc ? "Open or create a document in VectorCraft." : e.message, doc: null, settings: null, pending: null });
    }
    if (!force && s.revision === this.last.revision && this.state.settings) return this.#set({ connected: true, error: null });
    this.last = { revision: s.revision };
    try {
      this.schema ??= await schema("settings");
      const stored = (await readData(this.control)).settings ?? {};
      const parsed = await parseSettings(JSON.stringify(stored));
      if (!parsed.ok) return this.#set({ connected: true, error: parsed.error, doc: { title: s.title, revision: s.revision }, settings: null });
      this.#set({ connected: true, error: null, doc: { title: s.title, revision: s.revision }, settings: { values: parsed.settings, summary: parsed.summary, card: cardModel(this.schema, parsed.settings) } });
    } catch (e) {
      this.#set({ connected: true, error: e.message });
    }
  }

  /** Quick choices: any of {target_print_scale, style, seed, north_deg}. A change is checked by the CLI and previewed; nothing is applied until Draw. */
  editSettings(patch) {
    return this.#serial(async () => {
      const base = this.state.pending?.settings ?? this.state.settings?.values;
      if (!base) return this.#set({ message: "Open or create a document in VectorCraft first." });
      const r = await parseSettings(JSON.stringify({ ...base, ...patch }));
      this.#set(r.ok ? { pending: { settings: r.settings, summary: r.summary }, message: null } : { message: r.error });
    });
  }

  cancel() {
    return this.#serial(async () => this.#set({ pending: null, message: null }));
  }

  /** Applies the previewed change with the plug-in's command, then reads the document again. */
  draw() {
    return this.#serial(async () => {
      const pending = this.state.pending;
      if (!pending) return this.#set({ message: "Nothing to change." });
      this.#set({ busy: "drawing", message: null });
      try {
        await this.control.run("select.all");
        await this.control.run(SET_SETTINGS, { params: { description: JSON.stringify(pending.settings) } });
        this.#set({ pending: null, message: "Drawn." });
      } catch (e) {
        this.#set({ message: `Could not draw: ${e.message}` });
      } finally {
        this.#set({ busy: null });
        await this.#refresh(true);
      }
    });
  }
}
