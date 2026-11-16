import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import net from "node:net";
import vm from "node:vm";
import { join } from "node:path";
import { after, test } from "node:test";
import { Control } from "../lib/control.mjs";
import { Panel } from "../lib/panel.mjs";
import { createServer } from "../server.mjs";
import { fakeVectorCraft } from "./fake-vectorcraft.mjs";

const cleanups = [];
after(() => cleanups.forEach((f) => f()));

const freePort = () => new Promise((ok) => { const s = net.createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => ok(p)); }); });

async function start(options) {
  const vc = await fakeVectorCraft(options ?? { paths: [{ id: 5, name: "" }], selection: [5] });
  const control = new Control({ port: vc.port });
  const panel = new Panel({ control });
  await panel.tick();
  const port = await freePort();
  const server = createServer({ panel, port });
  await new Promise((ok) => server.listen(port, "127.0.0.1", ok));
  cleanups.push(() => { server.closeAllConnections(); server.close(); control.close(); vc.close(); });
  return { base: `http://127.0.0.1:${port}`, panel };
}

test("the page script has no HTML sink", () => {
  const script = readFileSync(join(import.meta.dirname, "../public/app.js"), "utf8");
  for (const sink of ["innerHTML", "outerHTML", "insertAdjacentHTML", "document.write", "eval(", "new Function"]) {
    assert.ok(!script.includes(sink), `app.js uses ${sink}`);
  }
});

test("markup in a name is passed through as plain data, for the page's textContent to show", async () => {
  const name = "<img src=x onerror=alert(1)>";
  const { panel } = await start({ paths: [{ id: 5, name }], selection: [5] });
  assert.equal(panel.snapshot().selection[0].name, name);
});

test("the page is served with a policy that forbids inline script", async () => {
  const { base } = await start();
  const policy = (await fetch(`${base}/`)).headers.get("content-security-policy") ?? "";
  for (const part of ["script-src 'self'", "object-src 'none'", "base-uri 'none'"]) assert.ok(policy.includes(part), `${part} in ${policy}`);
});

test("an unknown action is a 404 and an inherited key is not an action", async () => {
  const { base } = await start();
  for (const name of ["constructor", "toString", "__proto__", "nothing"]) {
    const r = await fetch(`${base}/api/${name}`, { method: "POST", headers: { "content-type": "application/json", origin: base }, body: "{}" });
    assert.equal(r.status, 404, name);
  }
});

// Runs the page script against a stub DOM and feeds it real snapshots: a state that makes the script throw (a missing element, a bad field) fails here.
function runPage() {
  const byId = new Map();
  const made = [];
  const stub = (tag = "div") => {
    const t = { tagName: tag, hidden: false, value: "", textContent: "", className: "", children: [], options: [], open: false };
    const p = new Proxy(t, {
      get(target, key) {
        if (key in target) return target[key];
        if (key === "classList") return { toggle() {}, add() {}, remove() {} };
        if (["append", "replaceChildren", "prepend", "remove", "setAttribute", "addEventListener", "focus"].includes(key)) return () => {};
        if (key === "style" || key === "dataset") return {};
        return undefined;
      },
      set(target, key, value) {
        target[key] = value;
        if (key === "id") byId.set(value, p); // an element the script made and named can be found by that name
        return true;
      },
    });
    made.push(p);
    return p;
  };
  const ids = new Set([...readFileSync(join(import.meta.dirname, "../public/index.html"), "utf8").matchAll(/\sid="([^"]+)"/g)].map((m) => m[1]));
  const events = { source: null };
  const sandbox = {
    document: { getElementById: (id) => { if (!ids.has(id) && !byId.has(id)) throw new Error(`app.js asks for #${id}, which index.html does not have`); if (!byId.has(id)) byId.set(id, stub()); return byId.get(id); }, createElement: (tag) => stub(tag), activeElement: null, body: { style: {} }, querySelectorAll: () => [], querySelector: () => null },
    location: { search: "" },
    URLSearchParams,
    EventSource: class { constructor() { events.source = this; } },
    fetch: async () => ({ ok: true, json: async () => ({}) }),
    prompt: () => null,
    console,
    setTimeout,
    Number, String, JSON, Math, Object, Array, Set, Map, Promise,
  };
  vm.runInNewContext(readFileSync(join(import.meta.dirname, "../public/app.js"), "utf8"), sandbox);
  return { send: (snapshot) => events.source.onmessage({ data: JSON.stringify(snapshot) }), byId };
}

const TWO = "lane back | lane fwd";
const SEGMENTS = { segments: { a: { cross_section: TWO, anchor: "auto" }, b: { cross_section: TWO, anchor: "auto" } } };

test("the page renders a plain line, a road, an intersection and the open settings card without error", async () => {
  const page = runPage();
  const cases = [
    { paths: [{ id: 5, name: "" }], selection: [5] },
    { paths: [{ id: 2, name: "road.a" }], selection: [2], spec: SEGMENTS },
    { paths: [{ id: 2, name: "road.a" }, { id: 3, name: "road.b" }], selection: [2, 3], spec: SEGMENTS },
    { paths: [{ id: 2, name: "road.a" }, { id: 3, name: "water.c" }], selection: [2, 3], spec: { ...SEGMENTS, waters: { c: { kind: "flowing", width: 8 } } } },
    { paths: [], selection: [], hasDocument: false },
  ];
  for (const options of cases) {
    const { panel } = await start(options);
    await panel.openSettings();
    await panel.editSettings({ style: "hand-drawn" });
    page.send(panel.snapshot());
  }
  assert.equal(page.byId.get("settingsCard").open, false);
  const { panel } = await start();
  await panel.openSettings();
  page.send(panel.snapshot());
  assert.match(page.byId.get("ssummary").textContent, /style medium-contrast/);
});
