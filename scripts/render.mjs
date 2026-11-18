// Builds track in a headless VectorCraft session (never your open document) and exports a cropped PNG, so you can look at it.
// Usage: node scripts/render.mjs <vectorcraft-cli> <scenario.json> <out.png> [pixels-per-point (default 56: about 9 pixels per mm of model)]
// Scenario: {"joints": [{"lines": ["a", "b"], "spec": {"kind": "crossing"}}], "lines": [{"id": "a", "points": [[x_mm, y_mm], ... or {"x","y","in":[x,y],"out":[x,y]} for Bezier handles], "spec": {"tracks": 2}, "command": "set_tracks"}]}  (points in mm from the page's top left; command set_tracks or set_yard)
import { spawn } from "node:child_process";
import { mkdirSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";

const [cli, scenarioPath, outArg, scale = "56"] = process.argv.slice(2);
if (!cli || !scenarioPath || !outArg) {
  console.error("usage: render.mjs <vectorcraft-cli> <scenario.json> <out.png> [scale]");
  process.exit(2);
}
const PLUGIN = "com.cosmicceo.cartographer.build";
const PT = 0.16; // document points per millimetre of model (1:160 on a 1 point = 1 m ground document)
const root = resolve(import.meta.dirname, "..");
const out = resolve(outArg);
mkdirSync(dirname(out), { recursive: true });
const scenario = JSON.parse(readFileSync(scenarioPath, "utf8"));

const child = spawn(cli, ["mcp", "--automation-read-root", root, "--automation-write-root", root], { stdio: ["pipe", "pipe", "inherit"] });
let buffer = "";
const waiting = new Map();
child.stdout.on("data", (chunk) => {
  buffer += chunk;
  for (let nl = buffer.indexOf("\n"); nl >= 0; nl = buffer.indexOf("\n")) {
    const msg = JSON.parse(buffer.slice(0, nl));
    buffer = buffer.slice(nl + 1);
    waiting.get(msg.id)?.(msg);
  }
});
let nextId = 1;
const call = (method, params) => new Promise((done) => {
  const id = nextId++;
  waiting.set(id, done);
  child.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", id, method, params })}\n`);
});
const tool = async (name, args) => {
  const r = await call("tools/call", { name, arguments: args });
  if (r.error || r.result?.isError) throw new Error(`${name}: ${JSON.stringify(r.error ?? r.result).slice(0, 400)}`);
  return r.result;
};
const run = (id, params = {}) => tool("command_run", { id, params });

await call("initialize", { protocolVersion: "2025-03-26", capabilities: {}, clientInfo: { name: "cartographer-render", version: "0" } });
child.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" })}\n`);
try {
  await run("plugin.install", { path: resolve(root, "target/wasm32-unknown-unknown/release/cartographer_filter.wasm"), replace: true });
  const ids = {};
  for (const l of scenario.lines) {
    const made = await tool("draw_path", { points: l.points.map((p) => (Array.isArray(p) ? [p[0] * PT, p[1] * PT] : { x: p.x * PT, y: p.y * PT, ...(p.in && { in: p.in.map((v) => v * PT) }), ...(p.out && { out: p.out.map((v) => v * PT) }) })), name: `track.${l.id}`, fill: "none", stroke: "#d91a1a", strokeWidth: 0.75 });
    ids[l.id] = JSON.parse(made.content[0].text).id;
    await run(`plugin.${PLUGIN}.${l.command ?? "set_tracks"}`, { params: { description: JSON.stringify(l.spec ?? { tracks: 1 }) } });
  }
  // Joints: select one line of each pair, then describe how they meet.
  for (const j of scenario.joints ?? []) {
    await run("select.set", { ids: j.lines.map((id) => ids[id]) });
    await run(`plugin.${PLUGIN}.set_joint`, { params: { description: JSON.stringify(j.spec ?? { kind: "crossing" }) } });
  }
  await run("select.all");
  const built = await run("plugin.run", { id: PLUGIN });
  console.log("build:", JSON.stringify(built).slice(0, 300));
  await run("select.all");
  await tool("export", { format: "png", path: out, selection: true, scale: Number(scale) });
  console.log("wrote", out);
  if (process.env.SAVE_DOC) await run("document.export", { format: "vectorcraft", path: resolve(process.env.SAVE_DOC) });
} finally {
  child.kill();
}
