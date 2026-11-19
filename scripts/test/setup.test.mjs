import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { cpSync, mkdtempSync } from "node:fs";
import net from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { test } from "node:test";
import { waitForPort } from "../lib/vc.mjs";

const repo = resolve(import.meta.dirname, "../..");

test("build.mjs in a release package says there is nothing to build", () => {
  const pkg = mkdtempSync(join(tmpdir(), "pkg-"));
  cpSync(join(repo, "scripts"), join(pkg, "scripts"), { recursive: true, filter: (p) => !p.includes("/test") });
  const r = spawnSync(process.execPath, [join(pkg, "scripts/build.mjs")], { encoding: "utf8" });
  assert.equal(r.status, 0, r.stderr);
  assert.match(r.stdout, /release package/i);
  assert.match(r.stdout, /nothing to build/i);
});

test("the panel says plainly when its port is already in use", async () => {
  const blocker = net.createServer().listen(0, "127.0.0.1");
  await new Promise((ok) => blocker.once("listening", ok));
  const port = blocker.address().port;
  const r = spawnSync(process.execPath, [join(repo, "panel/server.mjs"), "--port", String(port), "--control", "1"], { encoding: "utf8", timeout: 15000 });
  blocker.close();
  assert.equal(r.status, 1, r.stdout + r.stderr);
  assert.match(r.stderr, new RegExp(`port ${port} is already in use`));
  assert.doesNotMatch(r.stderr, /\n\s+at /, "no stack trace");
});

test("waitForPort answers true once something listens and false after the time is up", async () => {
  const free = await new Promise((ok) => { const s = net.createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => ok(p)); }); });
  assert.equal(await waitForPort(free, 400), false);
  const late = net.createServer();
  setTimeout(() => late.listen(free, "127.0.0.1"), 300);
  assert.equal(await waitForPort(free, 5000), true);
  late.close();
});
