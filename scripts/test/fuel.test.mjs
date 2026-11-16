import assert from "node:assert/strict";
import { test } from "node:test";
import { parseFuel, withinParity } from "../lib/fuel.mjs";

test("parseFuel reads the fuel number from the tool's line", () => {
  assert.equal(parseFuel('fuel 984433 output_bytes 1603 starts {"objects":[{"name":"gen.road'), 984433);
  assert.throws(() => parseFuel("panic: nothing"), /fuel/);
});

test("withinParity allows up to the limit and no more", () => {
  assert.equal(withinParity(1000, 1050, 1.05).ok, true);
  assert.equal(withinParity(1000, 1051, 1.05).ok, false);
  assert.equal(withinParity(1000, 900, 1.05).ok, true);
  assert.equal(withinParity(1000, 1049, 1.05).ratio.toFixed(3), "1.049");
});
