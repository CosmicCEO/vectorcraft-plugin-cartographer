import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { cliBinary, isPackaged, wasmFile } from "../lib/locate.mjs";

const folder = () => mkdtempSync(join(tmpdir(), "locate-"));
const put = (root, rel) => {
  mkdirSync(join(root, rel, ".."), { recursive: true });
  writeFileSync(join(root, rel), "x");
};

test("a source checkout is found by its build folders", () => {
  const root = folder();
  put(root, "Cargo.toml");
  put(root, "target/wasm32-unknown-unknown/release/cartographer_filter.wasm");
  put(root, "target/release/cartographer-cli");
  assert.equal(wasmFile(root), join(root, "target/wasm32-unknown-unknown/release/cartographer_filter.wasm"));
  assert.equal(cliBinary(root), join(root, "target/release/cartographer-cli"));
  assert.equal(isPackaged(root), false);
});

test("a release package is found by its own layout", () => {
  const root = folder();
  put(root, "cartographer_filter.wasm");
  put(root, "bin/cartographer-cli");
  assert.equal(wasmFile(root), join(root, "cartographer_filter.wasm"));
  assert.equal(cliBinary(root), join(root, "bin/cartographer-cli"));
  assert.equal(isPackaged(root), true);
});

test("a fresh build wins over a packaged copy, and nothing found is null", () => {
  const root = folder();
  assert.equal(wasmFile(root), null);
  assert.equal(cliBinary(root), null);
  put(root, "cartographer_filter.wasm");
  put(root, "target/wasm32-unknown-unknown/release/cartographer_filter.wasm");
  assert.equal(wasmFile(root), join(root, "target/wasm32-unknown-unknown/release/cartographer_filter.wasm"));
});
