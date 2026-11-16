import assert from "node:assert/strict";
import { test } from "node:test";
import { rustEnv } from "../lib/env.mjs";
import { appLaunch } from "../lib/vc.mjs";

test("rustEnv puts Homebrew's rustup first on macOS, only when it exists", () => {
  const env = { PATH: "/usr/bin:/bin", HOME: "/Users/x" };
  const yes = rustEnv("darwin", env, (p) => p === "/opt/homebrew/opt/rustup/bin");
  assert.equal(yes.PATH, "/opt/homebrew/opt/rustup/bin:/usr/bin:/bin");
  assert.equal(yes.HOME, "/Users/x");
  assert.deepEqual(rustEnv("darwin", env, () => false), env);
  const intel = rustEnv("darwin", env, (p) => p === "/usr/local/opt/rustup/bin");
  assert.equal(intel.PATH, "/usr/local/opt/rustup/bin:/usr/bin:/bin");
});

test("rustEnv leaves Linux alone and does not change the input", () => {
  const env = { PATH: "/usr/bin" };
  assert.deepEqual(rustEnv("linux", env, () => true), env);
  rustEnv("darwin", env, () => true);
  assert.equal(env.PATH, "/usr/bin");
});

test("appLaunch on macOS opens VectorCraft with the control port", () => {
  assert.deepEqual(appLaunch("darwin", {}, () => true, 7979), { cmd: "open", args: ["-a", "VectorCraft", "--args", "--control", "7979"] });
});

test("appLaunch on Linux uses VECTORCRAFT_APP, then PATH, else says what to set", () => {
  const app = "/tmp/Café maps/VectorCraft.AppImage";
  assert.deepEqual(appLaunch("linux", { VECTORCRAFT_APP: app }, () => true, 7979), { cmd: app, args: ["--control", "7979"] });
  assert.deepEqual(appLaunch("linux", { PATH: "/a:/opt/vc" }, (p) => p === "/opt/vc/vectorcraft", 7979), { cmd: "/opt/vc/vectorcraft", args: ["--control", "7979"] });
  const none = appLaunch("linux", {}, () => false, 7979);
  assert.match(none.error, /VECTORCRAFT_APP/);
  assert.match(appLaunch("win32", {}, () => true, 7979).error, /macOS and Linux/);
});
