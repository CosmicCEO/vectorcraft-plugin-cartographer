import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { cpSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

test("a script runs from a folder with spaces and non-ASCII letters in its path", () => {
  const dir = mkdtempSync(join(tmpdir(), "Café maps "));
  try {
    cpSync(join(import.meta.dirname, ".."), join(dir, "my scripts"), { recursive: true, filter: (p) => !p.includes("/test") });
    const r = spawnSync(process.execPath, [join(dir, "my scripts", "vc.mjs")], { encoding: "utf8" });
    assert.equal(r.status, 2, r.stderr);
    assert.match(r.stderr, /usage: vc\.mjs/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
