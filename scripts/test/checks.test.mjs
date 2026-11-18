import assert from "node:assert/strict";
import { test } from "node:test";
import { checks, problems } from "../lib/checks.mjs";

const good = { nodeVersion: "v22.1.0", hasRustup: true, hasWasmTarget: true, hasCli: true, controlReachable: true };

test("with everything present every check is ok", () => {
  const list = checks(good);
  assert.ok(list.length >= 5);
  assert.ok(list.every((c) => c.ok), JSON.stringify(list));
});

test("an old Node says what to install", () => {
  const bad = checks({ ...good, nodeVersion: "v20.1.0" }).find((c) => !c.ok);
  assert.match(bad.fix, /Node 22/);
});

test("Node 22 and newer pass, including a double-digit major", () => {
  for (const v of ["v22.0.0", "v24.2.1", "v26.13.1"]) assert.ok(checks({ ...good, nodeVersion: v })[0].ok, v);
  assert.equal(checks({ ...good, nodeVersion: "garbage" })[0].ok, false);
});

test("a missing Rust toolchain or wasm target gets one fix each", () => {
  const rust = checks({ ...good, hasRustup: false }).find((c) => !c.ok);
  assert.match(rust.fix, /rustup/);
  const target = checks({ ...good, hasWasmTarget: false }).find((c) => !c.ok);
  assert.match(target.fix, /wasm32-unknown-unknown/);
});

test("the VectorCraft checks are optional: a missing CLI or app does not fail the install", () => {
  const list = checks({ ...good, hasCli: false, controlReachable: false });
  const missing = list.filter((c) => !c.ok);
  assert.equal(missing.length, 2);
  assert.ok(missing.every((c) => c.optional && c.fix));
});

test("a release package needs no Rust, so the Rust checks are not asked for", () => {
  const list = checks({ nodeVersion: "v22.1.0", hasRustup: false, hasWasmTarget: false, packaged: true });
  assert.ok(!list.some((c) => /Rust|WebAssembly/.test(c.name)), list.map((c) => c.name).join(", "));
  assert.deepEqual(problems({ nodeVersion: "v22.1.0", packaged: true }), []);
  assert.ok(problems({ nodeVersion: "v22.1.0", hasRustup: false, hasWasmTarget: false }).length >= 2, "a source checkout still needs them");
});

