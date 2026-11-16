import assert from "node:assert/strict";
import { test } from "node:test";
import { archiveName, sumsLine } from "../lib/package.mjs";

test("sumsLine is the SHA-256 and the exact name, with spaces and non-ASCII kept", () => {
  assert.equal(sumsLine("abc.tar.gz", new TextEncoder().encode("abc")), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  abc.tar.gz");
  assert.ok(sumsLine("Café map.tar.gz", new Uint8Array([1, 2, 3])).endsWith("  Café map.tar.gz"));
});

test("archiveName says the version, system and chip", () => {
  assert.equal(archiveName("0.1.0", "darwin", "arm64"), "cartographer-0.1.0-macos-arm64.tar.gz");
  assert.equal(archiveName("0.1.0", "linux", "x64"), "cartographer-0.1.0-linux-x86_64.tar.gz");
  assert.equal(archiveName("0.1.0", "linux", "arm64"), "cartographer-0.1.0-linux-arm64.tar.gz");
  assert.throws(() => archiveName("0.1.0", "win32", "x64"), /macOS and Linux/);
});
