// Drives the styles and the target print scale through the panel's settings card against a live VectorCraft (started with --control).
//   node scripts/start-vectorcraft.mjs ; node scripts/install.mjs ; node panel/server.mjs &   then:   node scripts/live-style-check.mjs [out-folder]
// It works in a NEW document, needs no agent, and exports one PNG and one SVG per style into the folder.
import "./lib/require-node.mjs";
import assert from "node:assert/strict";
import { existsSync, mkdirSync, readFileSync, statSync } from "node:fs";
import { resolve } from "node:path";
import { Control } from "../panel/lib/control.mjs";

const panelUrl = process.env.PANEL_URL ?? "http://127.0.0.1:8789";
const out = resolve(process.argv[2] ?? "out");
mkdirSync(out, { recursive: true });
const vc = new Control({ port: Number(process.env.VC_PORT ?? 7979) });
const ID = "com.cosmicceo.cartographer.build";
const state = async () => (await fetch(`${panelUrl}/state`)).json();
const post = async (action, body = {}) => (await fetch(`${panelUrl}/api/${action}`, { method: "POST", headers: { "content-type": "application/json", origin: panelUrl }, body: JSON.stringify(body) })).json();
const until = async (what, fn, ms = 20_000) => {
  let s;
  for (const t = Date.now(); Date.now() - t < ms; await new Promise((r) => setTimeout(r, 150))) {
    s = await state();
    if (fn(s)) return s;
  }
  throw new Error(`timed out waiting for: ${what} (message: ${s?.message})`);
};
const say = (m) => console.log(`- ${m}`);
const line = async (x1, y1, x2, y2) => (await vc.run("path.create", { anchors: [{ x: x1, y: y1 }, { x: x2, y: y2 }] })).id;

/** Exports the document as SVG and waits for the file to be written. */
async function exportSvg(file) {
  await vc.run("select.all");
  await vc.call("app.export", { path: file, format: "svg" });
  let last = -1;
  for (let i = 0; i < 100; i++) {
    const size = existsSync(file) ? statSync(file).size : 0;
    if (size > 0 && size === last) return readFileSync(file, "utf8");
    last = size;
    await new Promise((r) => setTimeout(r, 100));
  }
  throw new Error(`${file} was not written`);
}
const road = (svg) => [...svg.matchAll(/<[a-z]+\s[^>]*id="gen\.road[^"]*"[^>]*>/g)].map((m) => m[0]);
const colours = (els) => els.flatMap((e) => [...e.matchAll(/(?:fill|stroke)="#([0-9a-f]{6})"/g)].map((m) => m[1]));
const grey = (hex) => hex.slice(0, 2) === hex.slice(2, 4) && hex.slice(2, 4) === hex.slice(4, 6);
const widths = (els) => els.flatMap((e) => [...e.matchAll(/stroke-width="([0-9.]+)"/g)].map((m) => Number(m[1])));

/** Sets the settings through the card, Draw this (which rebuilds), then exports. */
async function setAndExport(name, patch) {
  let s = await post("settings", { patch });
  assert.ok(s.settingsPending, `the card took ${JSON.stringify(patch)}: ${s.message}`);
  await post("drawsettings");
  await until("the settings are drawn", (x) => !x.busy && !x.settingsPending);
  const svg = await exportSvg(`${out}/live-${name}.svg`);
  await vc.call("app.export", { path: `${out}/live-${name}.png`, format: "png", ppi: 144, background: "white" });
  return svg;
}

await vc.run("file.new", {});
const A = await line(50, 300, 550, 300);
const B = await line(300, 100, 300, 500);
for (const id of [A, B]) {
  await vc.run("select.set", { ids: [id] });
  await vc.run(`plugin.${ID}.set_cross_section`, { params: { cross_section: "lane back | lane fwd" } });
}
await vc.run("select.all");
await vc.run("plugin.run", { id: ID });
await post("opensettings");
await until("the settings card", (s) => s.settings);
say("two crossing roads built, settings card open");

const medium = await exportSvg(`${out}/live-medium-contrast.svg`);
await vc.call("app.export", { path: `${out}/live-medium-contrast.png`, format: "png", ppi: 144, background: "white" });
assert.ok(road(medium).length > 3, "the roads were built");
assert.ok(colours(road(medium)).some((c) => !grey(c)), "medium-contrast has colour");

const hand = await setAndExport("hand-drawn", { style: "hand-drawn" });
assert.notDeepEqual(road(hand), road(medium), "hand-drawn moves the road");
const spine = (svg) => svg.match(/<[a-z]+\s[^>]*id="road\.r1"[^>]*>/)?.[0];
assert.ok(spine(medium) && spine(medium) === spine(hand), "the line you drew is not wobbled");
const edge = road(hand).find((e) => e.includes('id="gen.road.r1.1.line.0"')) ?? "";
const ys = [...(edge.match(/points="([^"]*)"/)?.[1] ?? "").matchAll(/[-0-9.]+,([-0-9.]+)/g)].map((m) => Number(m[1]));
assert.ok(ys.length > 10 && Math.max(...ys) - Math.min(...ys) > 0.1, `a straight hand-drawn edge wanders: ${ys.length} points`);
say(`hand-drawn: ${road(hand).length} road objects differ from medium-contrast, the spine is unchanged`);

const bw = await setAndExport("black-white", { style: "black-white" });
assert.ok(colours(road(bw)).every(grey), "black-white is grey only");
say("black-white: every road colour is a grey");

const hc = await setAndExport("high-contrast", { style: "high-contrast" });
assert.notDeepEqual(colours(road(hc)), colours(road(medium)), "high-contrast changes the colours");
say("high-contrast: colours differ from medium-contrast");

const back = await setAndExport("medium-contrast-again", { style: "medium-contrast" });
assert.deepEqual(road(back), road(medium), "going back to medium-contrast gives the first drawing again");
say("back to medium-contrast: the same drawing as at the start");

const scaled = await setAndExport("scale-1-5000", { target_print_scale: "1:5000" });
assert.ok(widths(road(scaled)).every((w) => w >= 0.6 - 1e-6), `strokes are at least 0.6 m at 1:5000: ${widths(road(scaled)).slice(0, 6)}`);
assert.ok(widths(road(medium)).some((w) => w < 0.6), "and they were thinner at true scale");
say("target print scale 1:5000: every road stroke is at least 0.6 m wide");

vc.close();
console.log("style live check passed");
