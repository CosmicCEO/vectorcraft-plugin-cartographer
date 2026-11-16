import assert from "node:assert/strict";
import { test } from "node:test";
import { nodeProblem } from "../lib/node-check.mjs";
import { checks, problems } from "../lib/checks.mjs";

test("an old Node gets one plain line naming Node 22", () => {
  for (const v of ["v18.19.0", "v20.10.0", "v21.7.3"]) assert.match(nodeProblem(v), /Node 22/, v);
  for (const v of ["v22.0.0", "v26.13.1"]) assert.equal(nodeProblem(v), null, v);
});

test("problems lists the fix for each required check that fails, and skips the optional ones", () => {
  const good = { nodeVersion: "v22.1.0", hasRustup: true, hasWasmTarget: true, hasCli: false, controlReachable: false };
  assert.deepEqual(problems(good), []);
  const list = problems({ ...good, hasRustup: false, hasWasmTarget: false });
  assert.equal(list.length, 2);
  assert.ok(list.some((l) => /wasm32-unknown-unknown/.test(l)) && list.some((l) => /rustup/.test(l)));
  assert.equal(checks(good).length, 5);
});
