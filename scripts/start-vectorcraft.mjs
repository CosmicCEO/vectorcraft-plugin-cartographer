// Starts VectorCraft with its control port, on macOS or Linux.   node scripts/start-vectorcraft.mjs [port]
// The port is set only when VectorCraft starts: if it is already running, quit it first.
import "./lib/require-node.mjs";
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { appLaunch, waitForPort } from "./lib/vc.mjs";

const port = Number(process.argv[2] ?? process.env.VECTORCRAFT_CONTROL_PORT ?? 7979);
if (await waitForPort(port, 300)) {
  console.log(`VectorCraft is already running with its control port ${port}: nothing to do.`);
  process.exit(0);
}
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
// The port answers only once VectorCraft has finished starting; install and the panel need it, so wait here.
if (await waitForPort(port, 30_000)) {
  console.log(`VectorCraft is ready on control port ${port}.`);
} else {
  console.error(`fix: VectorCraft did not answer on control port ${port} within 30 seconds. If it was already running, quit it (Cmd-Q on a Mac) and run this again: the port is set only when VectorCraft starts.`);
  process.exit(1);
}
