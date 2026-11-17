import assert from "node:assert/strict";
import { after, test } from "node:test";
import { Control } from "../lib/control.mjs";
import { SET_SETTINGS } from "../lib/doc.mjs";
import { Panel } from "../lib/panel.mjs";
import { fakeVectorCraft } from "./fake-vectorcraft.mjs";

const cleanups = [];
after(() => cleanups.forEach((f) => f()));

const TWO = "lane back | lane fwd";
const agent = { default: "fake", providers: { fake: { kind: "function", label: "Fake", fn: () => "{}" } } };

async function setup(options = { paths: [{ id: 5, name: "" }], selection: [5] }, providers = agent) {
  const vc = await fakeVectorCraft(options);
  const control = new Control({ port: vc.port });
  const panel = new Panel({ control, providers });
  cleanups.push(() => { control.close(); vc.close(); });
  await panel.tick();
  await panel.openSettings();
  return { vc, panel };
}

const steps = (vc) => vc.doc.calls.filter((c) => c.command !== "file.saveCopy" && !c.command.startsWith("layer.")).map((c) => c.command);

test("the settings card is not read until it is opened, and not refreshed after it is closed", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 5, name: "" }], selection: [5] });
  const control = new Control({ port: vc.port });
  const panel = new Panel({ control, providers: agent });
  cleanups.push(() => { control.close(); vc.close(); });
  await panel.tick();
  assert.equal(panel.snapshot().settings, null);
  assert.equal(vc.doc.calls.length, 0, "a plain selection makes no calls");
  await panel.openSettings();
  assert.equal(panel.snapshot().settings.values.style, "medium-contrast");
  await panel.editSettings({ seed: 5 });
  await panel.closeSettings();
  assert.equal(panel.snapshot().settings, null);
  assert.equal(panel.snapshot().settingsPending, null);
  const calls = vc.doc.calls.length;
  vc.doc.revision++;
  await panel.tick();
  assert.equal(vc.doc.calls.length, calls, "a closed card is not read again");
});

test("the settings card shows defaults when nothing is stored", async () => {
  const { panel } = await setup();
  const s = panel.snapshot();
  assert.equal(s.error, null);
  assert.equal(s.settings.card.fields.length, 4);
  assert.equal(s.settings.values.style, "medium-contrast");
  assert.equal(s.settings.values.target_print_scale, "true");
  assert.match(s.settings.summary, /style medium-contrast, target print scale true/);
});

test("a bad scale is refused before it reaches VectorCraft", async () => {
  const { vc, panel } = await setup();
  await panel.editSettings({ target_print_scale: "1:0" });
  assert.match(panel.snapshot().message, /1:N/);
  assert.equal(panel.snapshot().settingsPending, null);
  assert.deepEqual(steps(vc), []);
});

test("a choice is previewed, then Draw stores it", async () => {
  const { vc, panel } = await setup();
  await panel.editSettings({ style: "hand-drawn" });
  assert.equal(panel.snapshot().settingsPending.settings.style, "hand-drawn");
  assert.deepEqual(steps(vc), [], "nothing is applied until Draw");
  await panel.drawSettings();
  assert.deepEqual(steps(vc), ["select.all", SET_SETTINGS, "select.set"], "the selection is put back");
  assert.equal(vc.doc.spec.settings.style, "hand-drawn");
  const s = panel.snapshot();
  assert.equal(s.settingsPending, null);
  assert.equal(s.settings.values.style, "hand-drawn");
  assert.equal(s.message, "Drawn.", "the message survives the re-read");
});

test("Cancel drops a proposal and Draw with nothing proposed does nothing", async () => {
  const { vc, panel } = await setup();
  await panel.editSettings({ seed: 9 });
  await panel.cancelSettings();
  assert.equal(panel.snapshot().settingsPending, null);
  await panel.drawSettings();
  assert.deepEqual(steps(vc), []);
  assert.match(panel.snapshot().message, /Nothing to change/);
});

test("stored settings are shown, and a second change keeps the first", async () => {
  const { panel, vc } = await setup({ paths: [{ id: 5, name: "" }], selection: [5], spec: { schema: 1, settings: { style: "high-contrast", seed: 4 } } });
  assert.equal(panel.snapshot().settings.values.style, "high-contrast");
  await panel.editSettings({ target_print_scale: "1:2000" });
  await panel.editSettings({ north_deg: 15 });
  const p = panel.snapshot().settingsPending.settings;
  assert.deepEqual([p.style, p.target_print_scale, p.north_deg, p.seed], ["high-contrast", "1:2000", 15, 4]);
  assert.deepEqual(steps(vc), []);
});

test("settings that cannot be read show the defaults and say why, and Draw repairs them", async () => {
  const { panel, vc } = await setup({ paths: [{ id: 5, name: "" }], selection: [5], spec: { schema: 1, settings: { style: "neon" } } });
  const s = panel.snapshot();
  assert.equal(s.settings.values.style, "medium-contrast");
  assert.match(s.settings.problem, /style must be one of/);
  await panel.editSettings({ style: "hand-drawn" });
  await panel.drawSettings();
  assert.equal(vc.doc.spec.settings.style, "hand-drawn");
  assert.equal(panel.snapshot().settings.problem ?? null, null);
});

test("no open document is reported plainly", async () => {
  const { panel } = await setup({ hasDocument: false });
  assert.match(panel.snapshot().error, /Open or create a document/);
  assert.equal(panel.snapshot().settings, null);
  await panel.openSettings();
  assert.equal(panel.snapshot().settings, null, "nothing to show without a document");
});

test("VectorCraft not running is reported plainly", async () => {
  const panel = new Panel({ control: new Control({ port: 1 }), providers: agent });
  await panel.tick();
  assert.match(panel.snapshot().error, /VectorCraft is not answering on port 1/);
});

test("a Panel without agents still works, and an agent request says so plainly", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 5, name: "" }], selection: [5] });
  const control = new Control({ port: vc.port });
  const panel = new Panel({ control });
  cleanups.push(() => { control.close(); vc.close(); });
  await panel.tick();
  await panel.openSettings();
  await panel.editSettings({ seed: 3 });
  assert.equal(panel.snapshot().settingsPending.settings.seed, 3);
  await panel.propose("a two lane road", undefined);
  assert.match(panel.snapshot().message, /agent/i);
});

// Each way of making a proposal from a selection, in a document of its own; changing the document's title must drop it.
const KINDS = [
  ["road text", { paths: [{ id: 2, name: "road.r1" }], selection: [2], spec: { segments: { r1: { cross_section: TWO, anchor: "auto" } } } }, (p) => p.setText("lane back | lane fwd x2")],
  ["junction", { paths: [{ id: 2, name: "" }], selection: [2] }, async (p) => { p.setMode("connection"); await p.tick(true); await p.propose("a signalised crossing street", "fake"); }, { default: "fake", providers: { fake: { kind: "function", label: "Fake", fn: () => '{"legs":[{"side":"left","cross_section":"lane back | lane fwd"}],"control":"signal","crosswalks":true}' } } }],
  ["crossing", { paths: [{ id: 2, name: "road.a" }, { id: 3, name: "road.b" }], selection: [2, 3], spec: { segments: { a: { cross_section: TWO, anchor: "auto" }, b: { cross_section: TWO, anchor: "auto" } } } }, (p) => p.editCrossing({ treatment: "roundabout" })],
  ["hub", { paths: [{ id: 2, name: "road.a" }, { id: 3, name: "road.b" }, { id: 4, name: "road.c" }], selection: [2, 3, 4], spec: { segments: { a: { cross_section: TWO, anchor: "auto" }, b: { cross_section: TWO, anchor: "auto" }, c: { cross_section: TWO, anchor: "auto" } } } }, (p) => p.editHub({ treatment: "all way stop" })],
  ["lane change", { paths: [{ id: 2, name: "road.r1" }], selection: [2], spec: { segments: { r1: { cross_section: TWO, anchor: "auto" } } } }, async (p) => { p.setMode("transition"); await p.tick(true); await p.editTransition({ change: "add" }); }],
  ["water", { paths: [{ id: 2, name: "" }], selection: [2] }, async (p) => { p.setMode("water"); await p.tick(true); await p.editWater({ kind: "flowing", width: 10 }); }],
  ["road meets water", { paths: [{ id: 2, name: "road.a" }, { id: 3, name: "water.c" }], selection: [2, 3], spec: { segments: { a: { cross_section: TWO, anchor: "auto" } }, waters: { c: { kind: "flowing", width: 10 } } } }, (p) => p.editWet({ treatment: "culvert" })],
  ["embankment", { paths: [{ id: 2, name: "road.r1" }], selection: [2], spec: { segments: { r1: { cross_section: TWO, anchor: "auto" } } } }, async (p) => { p.setMode("fill"); await p.tick(true); await p.editEmbank({ height: 3 }); }],
];
for (const [name, options, prepare, providers] of KINDS) {
  test(`switching documents drops a ${name} proposal`, async () => {
    const { vc, panel } = await setup(options, providers);
    await prepare(panel);
    assert.ok(panel.snapshot().pending, `a ${name} proposal exists`);
    vc.doc.title = "Other.vectorcraft";
    await panel.tick();
    assert.equal(panel.snapshot().pending, null);
    assert.equal(panel.snapshot().doc.title, "Other.vectorcraft");
  });
}

test("switching documents drops a settings proposal and reads the new document's settings", async () => {
  const { vc, panel } = await setup();
  await panel.editSettings({ seed: 9 });
  assert.equal(panel.snapshot().settingsPending.settings.seed, 9);
  vc.doc.title = "Other.vectorcraft";
  vc.doc.spec = { schema: 1, settings: { seed: 2 } };
  await panel.tick();
  const s = panel.snapshot();
  assert.equal(s.settingsPending, null);
  assert.equal(s.settings.values.seed, 2);
});

test("Draw sends only the keys that were changed, so another document's other settings are never overwritten", async () => {
  const { vc, panel } = await setup({ paths: [{ id: 5, name: "" }], selection: [5], spec: { schema: 1, settings: { style: "high-contrast", seed: 4 } } });
  await panel.editSettings({ target_print_scale: "1:2000" });
  await panel.drawSettings();
  const call = vc.doc.calls.find((c) => c.command === SET_SETTINGS);
  assert.deepEqual(JSON.parse(call.params.params.description), { target_print_scale: "1:2000" });
  assert.equal(vc.doc.spec.settings.style, "high-contrast");
  assert.equal(vc.doc.spec.settings.seed, 4);
});

test("a settings proposal is dropped when the stored settings changed under it (another document with the same title, or an undo)", async () => {
  const { vc, panel } = await setup({ paths: [{ id: 5, name: "" }], selection: [5], spec: { schema: 1, settings: { style: "high-contrast", seed: 4 } } });
  await panel.editSettings({ seed: 9 });
  assert.ok(panel.snapshot().settingsPending);
  vc.doc.spec = { schema: 1, settings: { style: "black-white", seed: 4 } };
  vc.doc.revision++;
  await panel.tick();
  assert.equal(panel.snapshot().settingsPending, null);
  assert.equal(panel.snapshot().settings.values.style, "black-white");
});
