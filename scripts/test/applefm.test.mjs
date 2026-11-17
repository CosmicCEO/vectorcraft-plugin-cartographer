import assert from "node:assert/strict";
import { test } from "node:test";
import { appleHelperPlan } from "../lib/applefm.mjs";
import { checks } from "../lib/checks.mjs";

test("the helper builds only on macOS with swiftc", () => {
  assert.deepEqual(appleHelperPlan("darwin", true).build, ["swiftc", "-parse-as-library", "-O", "-o", "target/apple-fm-agent", "panel/apple-fm/main.swift"]);
  assert.equal(appleHelperPlan("darwin", false), null);
  assert.equal(appleHelperPlan("linux", true), null);
  assert.equal(appleHelperPlan("win32", true), null);
});

const good = { nodeVersion: "v22.1.0", hasRustup: true, hasWasmTarget: true, hasCli: true, controlReachable: true };

test("the helper check is optional, absent where it does not apply, and never a failure", () => {
  assert.equal(checks(good).some((c) => /Apple/.test(c.name)), false, "no entry when the probe did not look");
  const missing = checks({ ...good, appleHelper: false }).find((c) => /Apple/.test(c.name));
  assert.equal(missing.ok, false);
  assert.equal(missing.optional, true);
  assert.match(missing.fix, /node scripts\/build\.mjs/);
  assert.equal(checks({ ...good, appleHelper: true }).find((c) => /Apple/.test(c.name)).ok, true);
});
