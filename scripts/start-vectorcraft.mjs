// Starts VectorCraft with its control port, on macOS or Linux.   node scripts/start-vectorcraft.mjs [port]
// The port is set only when VectorCraft starts: if it is already running, quit it first.
import "./lib/require-node.mjs";
import { spawn, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { appLaunch, isRunning, waitForPort } from "./lib/vc.mjs";

const port = Number(process.argv[2] ?? process.env.VECTORCRAFT_CONTROL_PORT ?? 7979);
if (await waitForPort(port, 300)) {
  console.log(`VectorCraft is already running with its control port ${port}: nothing to do.`);
  process.exit(0);
}
// A running VectorCraft may just be busy: give its port a while before deciding it was started without one.
if (isRunning(process.platform, spawnSync) && (await waitForPort(port, 10_000))) {
  console.log(`VectorCraft is already running with its control port ${port}: nothing to do.`);
  process.exit(0);
}
if (isRunning(process.platform, spawnSync)) {
  console.error(`fix: VectorCraft is running without its control port ${port}. Quit it (Cmd-Q) and run this again: the port is set only when VectorCraft starts.`);
  process.exit(1);
}
const launch = appLaunch(process.platform, process.env, existsSync, port);
if (launch.error) {
  console.error(`fix: ${launch.error}`);
  process.exit(1);
}
const child = spawn(launch.cmd, launch.args, { detached: true, stdio: ["ignore", "ignore", "pipe"] });
let stderr = "";
let failed = null;
child.stderr.on("data", (d) => (stderr += d));
child.on("error", (e) => (failed = e.message));
// `open -a` (macOS) exits at once: non-zero if VectorCraft is not installed. On Linux the app itself runs here: an exit is a crash.
child.on("exit", (code) => {
  if (code) failed = `${launch.cmd} exited with code ${code}${stderr.trim() ? `: ${stderr.trim().slice(0, 200)}` : ""}`;
});
child.unref();
// The port answers only once VectorCraft has finished starting; install and the panel need it, so wait here.
let ready = false;
for (const end = Date.now() + 30_000; Date.now() < end && !failed && !(ready = await waitForPort(port, 500)); );
if (failed) {
  console.error(`fix: VectorCraft could not be started (${failed}). Is it installed? On Linux set VECTORCRAFT_APP to the AppImage.`);
  process.exit(1);
}
if (ready) {
  console.log(`VectorCraft is ready on control port ${port}.`);
  process.exit(0);
} else {
  console.error(`fix: VectorCraft did not answer on control port ${port} within 30 seconds. If it was already running, quit it (Cmd-Q on a Mac) and run this again: the port is set only when VectorCraft starts.`);
  process.exit(1);
}
