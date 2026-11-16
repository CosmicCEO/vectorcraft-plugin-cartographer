import assert from "node:assert/strict";
import { chmodSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { after, test } from "node:test";
import { Control } from "../lib/control.mjs";
import { runCli } from "../lib/cli.mjs";
import { Panel } from "../lib/panel.mjs";
import { fakeVectorCraft } from "./fake-vectorcraft.mjs";

const cleanups = [];
after(() => cleanups.forEach((f) => f()));

const TWO = "lane back | lane fwd";
const SEG = { segments: { r1: { cross_section: TWO, anchor: "auto" } } };

async function setup(options) {
  const vc = await fakeVectorCraft(options);
  const control = new Control({ port: vc.port });
  const panel = new Panel({ control });
  cleanups.push(() => { control.close(); vc.close(); });
  await panel.tick();
  return { vc, panel };
}

/** Runs `fn` with CARTOGRAPHER_CLI pointing at a program that fails the way a crashed helper does (exit 101, nothing on stdout). */
async function withBrokenCli(fn) {
  const dir = mkdtempSync(join(tmpdir(), "Broken cli "));
  const script = join(dir, "cartographer-cli");
  writeFileSync(script, "#!/bin/sh\necho 'thread panicked' >&2\nexit 101\n");
  chmodSync(script, 0o755);
  const before = process.env.CARTOGRAPHER_CLI;
  process.env.CARTOGRAPHER_CLI = script;
  try {
    return await fn();
  } finally {
    if (before === undefined) delete process.env.CARTOGRAPHER_CLI;
    else process.env.CARTOGRAPHER_CLI = before;
  }
}

test("drawing settings keeps the user's selection and a road proposal", async () => {
  const { vc, panel } = await setup({ paths: [{ id: 2, name: "road.r1" }, { id: 3, name: "road.r2" }], selection: [2], spec: SEG });
  await panel.setText("lane back | lane fwd x2");
  assert.ok(panel.snapshot().pending);
  await panel.openSettings();
  await panel.editSettings({ style: "hand-drawn" });
  await panel.drawSettings();
  assert.deepEqual(vc.doc.selection, [2], "the selection comes back");
  assert.ok(panel.snapshot().pending, "the road proposal is still there");
  assert.equal(panel.snapshot().target.kind, "road");
  assert.equal(vc.doc.spec.settings.style, "hand-drawn");
});

test("a helper that crashes is reported in plain words, not as a raw process error", async () => {
  await withBrokenCli(async () => {
    await assert.rejects(runCli(["version"]), (e) => /cartographer-cli could not run \(exit code 101\)/.test(e.message) && !/Command failed/.test(e.message));
  });
});

test("a crashed helper during the poll shows an error on the page and recovers when it is fixed", async () => {
  const { panel } = await setup({ paths: [{ id: 2, name: "road.r1" }], selection: [2], spec: SEG });
  assert.equal(panel.snapshot().target.kind, "road");
  await withBrokenCli(async () => {
    await panel.tick(true);
    assert.match(panel.snapshot().error ?? "", /cartographer-cli could not run/);
  });
  await panel.tick(true);
  assert.equal(panel.snapshot().error, null);
  assert.equal(panel.snapshot().target.kind, "road");
});

test("settings that cannot be read because the helper crashed say so on the card", async () => {
  const { panel } = await setup({ paths: [{ id: 5, name: "" }], selection: [5] });
  await withBrokenCli(async () => {
    await panel.openSettings();
    assert.match(panel.snapshot().settings?.problem ?? "", /could not be read/);
  });
});

test("a document switch clears the old document's build notes", async () => {
  const { vc, panel } = await setup({ paths: [{ id: 2, name: "road.r1" }], selection: [2], spec: { ...SEG, notes: ["road r1: something from the last build"] } });
  assert.deepEqual(panel.snapshot().notes, ["road r1: something from the last build"]);
  vc.doc.title = "Other.vectorcraft";
  vc.doc.selection = [];
  vc.doc.spec = {};
  await panel.tick();
  assert.deepEqual(panel.snapshot().notes, []);
});
