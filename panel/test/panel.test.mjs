import assert from "node:assert/strict";
import { after, test } from "node:test";
import { Control } from "../lib/control.mjs";
import { SET_SETTINGS } from "../lib/doc.mjs";
import { Panel } from "../lib/panel.mjs";
import { fakeVectorCraft } from "./fake-vectorcraft.mjs";

const cleanups = [];
after(() => cleanups.forEach((f) => f()));

async function setup(options = { paths: [{ id: 5, name: "" }], selection: [5] }) {
  const vc = await fakeVectorCraft(options);
  const control = new Control({ port: vc.port });
  const panel = new Panel({ control });
  cleanups.push(() => { control.close(); vc.close(); });
  await panel.tick();
  return { vc, panel };
}

// The engine commands the user's steps made, without the saved copies the panel reads.
const steps = (vc) => vc.doc.calls.filter((c) => c.command !== "file.saveCopy").map((c) => c.command);

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
  assert.equal(panel.snapshot().pending, null);
  assert.deepEqual(steps(vc), []);
});

test("a choice is previewed, then Draw stores it", async () => {
  const { vc, panel } = await setup();
  await panel.editSettings({ style: "hand-drawn" });
  assert.equal(panel.snapshot().pending.settings.style, "hand-drawn");
  assert.deepEqual(steps(vc), [], "nothing is applied until Draw");
  await panel.draw();
  assert.deepEqual(steps(vc), ["select.all", SET_SETTINGS]);
  assert.equal(vc.doc.spec.settings.style, "hand-drawn");
  const s = panel.snapshot();
  assert.equal(s.pending, null);
  assert.equal(s.settings.values.style, "hand-drawn");
  assert.equal(s.message, "Drawn.");
});

test("Cancel drops a proposal and Draw with nothing proposed does nothing", async () => {
  const { vc, panel } = await setup();
  await panel.editSettings({ seed: 9 });
  await panel.cancel();
  assert.equal(panel.snapshot().pending, null);
  await panel.draw();
  assert.deepEqual(steps(vc), []);
  assert.match(panel.snapshot().message, /Nothing to change/);
});

test("stored settings are shown, and a second change keeps the first", async () => {
  const { panel, vc } = await setup({ paths: [{ id: 5, name: "" }], selection: [5], spec: { schema: 1, settings: { style: "high-contrast", seed: 4 } } });
  assert.equal(panel.snapshot().settings.values.style, "high-contrast");
  await panel.editSettings({ target_print_scale: "1:2000" });
  await panel.editSettings({ north_deg: 15 });
  const p = panel.snapshot().pending.settings;
  assert.deepEqual([p.style, p.target_print_scale, p.north_deg, p.seed], ["high-contrast", "1:2000", 15, 4]);
  assert.deepEqual(steps(vc), []);
});

test("no open document is reported plainly", async () => {
  const { panel } = await setup({ hasDocument: false });
  assert.match(panel.snapshot().error, /Open or create a document/);
});

test("VectorCraft not running is reported plainly", async () => {
  const panel = new Panel({ control: new Control({ port: 1 }) });
  await panel.tick();
  assert.match(panel.snapshot().error, /VectorCraft is not answering on port 1/);
});
