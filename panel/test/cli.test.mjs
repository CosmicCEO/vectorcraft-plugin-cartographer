import assert from "node:assert/strict";
import { test } from "node:test";
import { library, parseCrossSection, parseSettings, runCli } from "../lib/cli.mjs";

test("a failing CLI reply is returned, not thrown", async () => {
  // The CLI exits 1 when ok is false; the panel still gets its plain message.
  const r = await parseSettings(JSON.stringify({ target_print_scale: "1:0" }));
  assert.equal(r.ok, false);
  assert.match(r.error, /1:N/);
});

test("a missing CLI names the fix", async () => {
  const before = process.env.CARTOGRAPHER_CLI;
  process.env.CARTOGRAPHER_CLI = "/nonexistent/cartographer-cli";
  try {
    await assert.rejects(runCli(["version"]), /cartographer-cli not found.*node scripts\/build\.mjs/);
  } finally {
    if (before === undefined) delete process.env.CARTOGRAPHER_CLI;
    else process.env.CARTOGRAPHER_CLI = before;
  }
});

test("the road wrappers reach the CLI", async () => {
  const p = await parseCrossSection("lane back | lane fwd");
  assert.equal(p.ok, true);
  assert.equal(p.total_width, 7.2);
  assert.ok((await library()).elements.length >= 15);
});
