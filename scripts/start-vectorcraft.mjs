// Starts VectorCraft with its control port, on macOS or Linux.   node scripts/start-vectorcraft.mjs [port]
// The port is set only when VectorCraft starts: if it is already running, quit it first.
import "./lib/require-node.mjs";
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { appLaunch } from "./lib/vc.mjs";

const port = Number(process.argv[2] ?? process.env.VECTORCRAFT_CONTROL_PORT ?? 7979);
const launch = appLaunch(process.platform, process.env, existsSync, port);
if (launch.error) {
  console.error(`fix: ${launch.error}`);
  process.exit(1);
}
const child = spawn(launch.cmd, launch.args, { detached: true, stdio: "ignore" });
child.on("error", (e) => {
  console.error(`fix: could not start ${launch.cmd}: ${e.message}`);
  process.exit(1);
});
child.unref();
console.log(`started VectorCraft with control port ${port} (if it was already running, quit it and run this again)`);
