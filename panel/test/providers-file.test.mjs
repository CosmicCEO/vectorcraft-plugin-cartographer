import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { loadProviders } from "../lib/providers.mjs";

const file = (text) => {
  const p = join(mkdtempSync(join(tmpdir(), "prov-")), "providers.json");
  writeFileSync(p, text);
  return p;
};

test("a malformed or odd user providers file is ignored with a warning, and the built-in agents stay", () => {
  for (const text of ["{", "null", "[]", '{"providers": 5}', '"text"']) {
    const cfg = loadProviders(file(text));
    assert.ok(cfg.providers.claude, `built-in claude survives ${text}`);
    assert.equal(cfg.warnings.length, 1, `${text} gives one warning`);
    assert.match(cfg.warnings[0], /providers\.json.*ignored/);
  }
});

test("an agent that cannot work is left out with a warning, the others are kept", () => {
  const cfg = loadProviders(file(JSON.stringify({ providers: { nocmd: { kind: "cli" }, nourl: { kind: "http" }, notobject: 5, good: { kind: "cli", command: "my-agent" } } })));
  assert.ok(cfg.providers.good && cfg.providers.claude);
  for (const bad of ["nocmd", "nourl", "notobject"]) assert.equal(cfg.providers[bad], undefined, bad);
  assert.equal(cfg.warnings.length, 3);
  assert.ok(cfg.warnings.some((w) => /nocmd.*command/.test(w)) && cfg.warnings.some((w) => /nourl.*base_url/.test(w)));
});

test("a good file and a missing file give no warnings", () => {
  assert.deepEqual(loadProviders(file('{"default": "local", "providers": {"local": {"kind": "http", "base_url": "http://localhost:11434/v1", "model": "m"}}}')).warnings, []);
  assert.deepEqual(loadProviders(join(tmpdir(), "no-such-providers.json")).warnings, []);
});
