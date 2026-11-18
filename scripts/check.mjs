// Checks this machine can build and run Cartographer; prints `ok:` or `fix:` lines. Exit 1 if a required check fails.
//   node scripts/check.mjs [--ci]      (--ci skips the VectorCraft checks)
import "./lib/require-node.mjs";
import { existsSync } from "node:fs";
import net from "node:net";
import { homedir } from "node:os";
import { join } from "node:path";
import { checks } from "./lib/checks.mjs";
import { rustEnv } from "./lib/env.mjs";
import { isPackaged } from "./lib/locate.mjs";
import { probeRust } from "./lib/probe.mjs";

const ci = process.argv.includes("--ci");
const env = rustEnv(process.platform, process.env, existsSync);
const cliPath = process.env.VECTORCRAFT_CLI ?? join(homedir(), ".local/share/vectorcraft-cli/vectorcraft-cli-0.24.1-macos-universal/vectorcraft-cli");
const reachable = (port) =>
  new Promise((done) => {
    const s = net.connect(port, "127.0.0.1");
    s.setTimeout(500, () => (s.destroy(), done(false)));
    s.on("connect", () => (s.destroy(), done(true)));
    s.on("error", () => done(false));
  });

const probe = {
  nodeVersion: process.version,
  packaged: isPackaged(join(import.meta.dirname, "..")),
  ...probeRust(env),
  hasCli: existsSync(cliPath),
  appleHelper: process.platform === "darwin" ? existsSync(join(import.meta.dirname, "../target/apple-fm-agent")) : undefined,
  controlReachable: ci ? false : await reachable(Number(process.env.VECTORCRAFT_CONTROL_PORT ?? 7979)),
};
let failed = false;
for (const c of checks(probe)) {
  if (ci && c.optional) continue;
  console.log(`${c.ok ? "ok" : c.optional ? "note" : "fix"}: ${c.name}${c.ok ? "" : ` - ${c.fix}`}`);
  if (!c.ok && !c.optional) failed = true;
}
process.exit(failed ? 1 : 0);
