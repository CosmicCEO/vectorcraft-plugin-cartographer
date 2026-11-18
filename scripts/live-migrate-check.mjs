// Proves the migration against the real old plug-ins: the same document built by Road Builder (or Track Layer) and, after `migrate`, by Cartographer
// must give the same drawing (roads: identical SVG; tracks: identical SVG when scaled). Needs a running VectorCraft (started with --control).
//   node scripts/live-migrate-check.mjs <road_builder_filter.wasm> <track_layer_filter.wasm> [out-folder]
// It works in NEW documents and installs the three plug-ins (replacing older copies of the same ids).
import "./lib/require-node.mjs";
import assert from "node:assert/strict";
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { Control } from "../panel/lib/control.mjs";
import { IDS, TRACK_SCALE, migrateBytes } from "./lib/migrate.mjs";
import { compareSvg, compareSvgScaled, skipCollapsedRings } from "./lib/svgcompare.mjs";

const [roadWasm, trackWasm, outArg = "out"] = process.argv.slice(2);
if (!roadWasm || !trackWasm) {
  console.error("usage: live-migrate-check.mjs <road_builder_filter.wasm> <track_layer_filter.wasm> [out-folder]");
  process.exit(2);
}
const root = resolve(import.meta.dirname, "..");
const out = resolve(outArg);
mkdirSync(out, { recursive: true });
const vc = new Control({ port: Number(process.env.VC_PORT ?? 7979) });
const say = (m) => console.log(`- ${m}`);

const settled = async (file) => {
  let last = -1;
  for (let i = 0; i < 100; i++) {
    const size = existsSync(file) ? statSync(file).size : 0;
    if (size > 0 && size === last) return;
    last = size;
    await new Promise((r) => setTimeout(r, 100));
  }
  throw new Error(`${file} was not written`);
};
async function exportSvg(file) {
  await vc.run("select.all");
  await vc.call("app.export", { path: file, format: "svg" });
  await settled(file);
  return readFileSync(file, "utf8");
}
async function saveCopy(file) {
  await vc.run("file.saveCopy", { path: file });
  await settled(file);
}
const line = async (x1, y1, x2, y2) => (await vc.run("path.create", { anchors: [{ x: x1, y: y1 }, { x: x2, y: y2 }] })).id;
const select = (...ids) => vc.run("select.set", { ids });
/** Migrates a saved copy and opens the result as the active document. */
async function migrateAndOpen(name) {
  const { bytes, report } = migrateBytes(readFileSync(`${out}/${name}.old.vectorcraft`));
  writeFileSync(`${out}/${name}.migrated.vectorcraft`, bytes);
  for (const l of report) say(`  ${l}`);
  await vc.call("app.open", { path: `${out}/${name}.migrated.vectorcraft` });
}

for (const [wasm, label] of [[roadWasm, "Road Builder"], [trackWasm, "Track Layer"], [`${root}/target/wasm32-unknown-unknown/release/cartographer_filter.wasm`, "Cartographer"]]) {
  await vc.run("plugin.install", { path: resolve(wasm), replace: true });
  say(`installed ${label}`);
}

// ---- roads: two crossing roads and a signal, built by Road Builder
await vc.run("file.new", {});
const [A, B] = [await line(50, 300, 550, 300), await line(300, 100, 300, 500)];
for (const id of [A, B]) {
  await select(id);
  await vc.run(`plugin.${IDS.roadbuilder}.set_cross_section`, { params: { cross_section: "lane back | lane fwd" } });
}
await select(A, B);
await vc.run(`plugin.${IDS.roadbuilder}.set_crossing`, { params: { description: JSON.stringify({ treatment: "signal", crosswalks: true }) } });
await vc.run("select.all");
await vc.run("plugin.run", { id: IDS.roadbuilder });
const oldRoad = await exportSvg(`${out}/mig-road.old.svg`);
await saveCopy(`${out}/mig-road.old.vectorcraft`);
await migrateAndOpen("mig-road");
await vc.run("select.all");
await vc.run("plugin.run", { id: IDS.cartographer });
const newRoad = await exportSvg(`${out}/mig-road.new.svg`);
const roads = compareSvg(oldRoad, newRoad);
assert.ok(roads.same, `roads: ${roads.summary}`);
assert.ok((oldRoad.match(/id="gen\./g) ?? []).length > 10, "the roads were really built");
say("roads: the migrated document builds with Cartographer to the same SVG as Road Builder");

// ---- track: a yard and a crossing track, in model millimetres at 72/25.4 points per mm, built by Track Layer
await vc.run("file.new", {});
const K = 72 / 25.4;
const [L1, L2] = [await line(100 * K, 400 * K, 1700 * K, 400 * K), await line(900 * K, 200 * K, 900 * K, 600 * K)];
await select(L1);
await vc.run(`plugin.${IDS.tracklayer}.set_yard`, { params: { description: JSON.stringify({ mains: 1, left: 2, ends: "stub" }) } });
await select(L2);
await vc.run(`plugin.${IDS.tracklayer}.set_tracks`, { params: { description: JSON.stringify({ tracks: 1 }) } });
await select(L1, L2);
await vc.run(`plugin.${IDS.tracklayer}.set_joint`, { params: { description: JSON.stringify({ kind: "crossing", main: "t1" }) } });
await vc.run("select.all");
await vc.run("plugin.run", { id: IDS.tracklayer });
const oldTrack = await exportSvg(`${out}/mig-track.old.svg`);
await saveCopy(`${out}/mig-track.old.vectorcraft`);
await migrateAndOpen("mig-track");
await vc.run("select.all");
await vc.run("plugin.run", { id: IDS.cartographer });
const newTrack = await exportSvg(`${out}/mig-track.new.svg`);
const skipped = skipCollapsedRings(oldTrack, newTrack, TRACK_SCALE);
const tracks = compareSvgScaled(skipped.old, skipped.new, { factor: TRACK_SCALE, tolerance: 0.001 });
assert.ok(tracks.same, `tracks: ${tracks.summary}`);
assert.ok((oldTrack.match(/id="gen\.track/g) ?? []).length > 5, "the track was really built");
say(`tracks: the migrated document builds with Cartographer to the same SVG as Track Layer, scaled by ${TRACK_SCALE.toFixed(7)} (${skipped.skipped} collapsed ring(s) of Track Layer skipped)`);

await vc.call("app.export", { path: `${out}/mig-track.new.png`, format: "png", ppi: 144, background: "white" });
vc.close();
console.log("migration live check passed");
