// Drives the rail parts of the panel against a live VectorCraft (started with --control): a track, a yard, a joint, a road next to a track.
//   node scripts/start-vectorcraft.mjs ; node scripts/install.mjs ; node panel/server.mjs &   then:   node scripts/live-rail-check.mjs [out-folder]
// It works in a NEW document (File > New), never in one you have open to edit. It needs no agent: cards and plug-in commands only.
import "./lib/require-node.mjs";
import assert from "node:assert/strict";
import { mkdirSync } from "node:fs";
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
  throw new Error(`timed out waiting for: ${what} (message: ${s?.message}, target: ${JSON.stringify(s?.target)})`);
};
const say = (m) => console.log(`- ${m}`);
const line = async (x1, y1, x2, y2) => (await vc.run("path.create", { anchors: [{ x: x1, y: y1 }, { x: x2, y: y2 }] })).id;
const select = (...ids) => vc.run("select.set", { ids });
const names = async () => ((await vc.call("document.inspect", { kind: "Path", limit: 100000 })).nodes ?? []).map((n) => n.name ?? "");

await vc.run("file.new", {});
say("a new document");

// 1. A plain line becomes a double track through the card.
const A = await line(50, 300, 650, 300);
await select(A);
await until("a plain line", (s) => s.target?.kind === "line");
await post("mode", { mode: "track" });
let s = await until("the track card for a plain line", (s) => s.track && !s.track.described);
assert.deepEqual(s.track.card.fields.map((f) => f.key), ["tracks", "spacing", "ballast", "ties"]);
s = await post("track", { patch: { tracks: 2 } });
assert.equal(s.pending?.kind, "tracks", `a proposal: ${s.message}`);
await post("draw");
s = await until("a described track", (s) => s.target?.kind === "track" && s.track?.described);
assert.equal(s.track.spec.tracks, 2);
const built = (await names()).filter((n) => n.startsWith("gen.track.t1."));
assert.ok(built.length >= 3, `rails, ties and roadbed were built: ${built}`);
say(`double track drawn: ${built.length} objects, e.g. ${built[0]}`);
const layers = JSON.stringify((await vc.call("document.inspect", { depth: 8 })).layers);
assert.ok(layers.includes("track t1 spine") && layers.includes("track t1 drawing"), "the track has its layers");
say("layers: track t1 with a spine and a drawing part");

// 2. A yard through the plug-in's own command, read back by the panel.
await select(A);
await vc.run(`plugin.${ID}.set_yard`, { params: { description: JSON.stringify({ mains: 1, left: 2, right: 1, ends: "stub" }) } });
await post("draw");
s = await until("a yard", (s) => s.track?.kind === "yard" && s.track.described);
assert.ok(s.track.min_line_m > 0 && s.track.card.fields.length === 10);
say(`yard read back: ${s.track.summary}`);

// 3. A second line, made track, crossing the first: two track lines are a joint.
const B = await line(300, 100, 300, 500);
await select(B);
await until("a plain line again", (s) => s.target?.kind === "line");
await post("draw");
await until("the second track", (s) => s.target?.kind === "track" && s.target.id !== "t1");
await select(A, B);
s = await until("a joint", (s) => s.target?.kind === "joint");
assert.equal(s.joint.found.ok, true, JSON.stringify(s.joint.found));
assert.equal(s.joint.found.kind, "crossing");
say(`the panel read the meeting: ${s.joint.found.summary}`);
await post("draw");
s = await until("a stored joint", (s) => s.joint?.described);
say(`joint stored: ${s.joint.summary}`);
s = await post("joint", { patch: { kind: "crossing", main: "t1" } });
assert.equal(s.pending?.kind, "joint", s.message);
await post("draw");
s = await until("the joint with its main", (s) => s.joint?.described && s.joint.spec.main === "t1");
say(`joint changed with the card: ${s.joint.summary}`);

// 4. A road line crossing a track line is a level crossing: the card makes it gated, Draw builds the panel, crossbucks and gates.
const R = await line(450, 100, 450, 550);
await select(R);
await vc.run(`plugin.${ID}.set_cross_section`, { params: { cross_section: "lane back | lane fwd" } });
await select(R, A);
s = await until("a level crossing", (s) => s.target?.kind === "level");
s = await post("level", { patch: { treatment: "gated" } });
assert.equal(s.pending?.kind, "level", s.message);
await post("draw");
s = await until("the stored level crossing", (s) => s.level?.described && s.level.spec.treatment === "gated");
const lc = (await names()).filter((n) => n.includes(".level."));
assert.ok(lc.some((n) => n.endsWith(".panel")) && lc.filter((n) => n.includes(".gate.")).length === 2, `level crossing objects: ${lc}`);
say(`a road across a track: a gated level crossing (${lc.length} objects)`);

// 5. One rebuild from the panel draws the road and the tracks together.
await select(A);
await until("the track is selected again", (s) => s.target?.kind === "track" && s.target.id === "t1");
await post("draw");
const after = await names();
assert.ok(after.some((n) => n.startsWith("gen.track.t1.")), "the track is still there after a rebuild");
assert.ok(after.some((n) => n.startsWith("gen.road.r1.")), "the road is built too, in the same build");
say("one rebuild draws the road and the tracks");

// 6. A spine layer the user hid is shown for the build and hidden again afterwards, and the track survives.
const tree = (await vc.call("document.inspect", { depth: 8 })).layers;
const flat = (list) => list.flatMap((l) => (l.kind === "Layer" ? [l, ...flat(l.children ?? [])] : []));
const find = (list) => flat(list).find((l) => l.name === "track t1 spine");
const spine = find(tree);
await vc.run("layer.setProps", { id: spine.id, visible: false });
await select(A);
await until("the track is selected again", (s) => s.target?.kind === "track" && s.target.id === "t1");
await post("draw");
assert.equal(find((await vc.call("document.inspect", { depth: 8 })).layers).visible, false, "the spine is hidden again");
assert.ok((await names()).some((n) => n.startsWith("gen.track.t1.")), "the track survived a build with its spine hidden");
await vc.run("layer.setProps", { id: spine.id, visible: true });
say("a hidden spine layer: the track is rebuilt and the spine is hidden again");

await vc.run("select.all");
await vc.call("app.export", { path: `${out}/live-rail.png`, format: "png", ppi: 144, background: "white" });
say(`exported ${out}/live-rail.png`);
vc.close();
console.log("rail live check passed");
