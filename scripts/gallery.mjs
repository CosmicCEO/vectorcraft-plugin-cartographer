// Re-renders the rail layouts in examples/rail/ into docs/img/gallery-<name>.png using a headless VectorCraft session (your open document is never touched).
//   node scripts/gallery.mjs <vectorcraft-cli>        (build the plug-in first: node scripts/build.mjs)
import "./lib/require-node.mjs";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";

const [cli] = process.argv.slice(2);
if (!cli) {
  console.error("usage: gallery.mjs <vectorcraft-cli>");
  process.exit(2);
}
const root = resolve(import.meta.dirname, "..");
// Pixels per document point: 0.16 points per mm, so 9 is about 1.4 pixels per mm of model (the size Track Layer's gallery used).
for (const [name, scale] of [["oval", 9], ["terminal", 9], ["turnouts", 10.6], ["station", 9]]) {
  const r = spawnSync(process.execPath, [resolve(root, "scripts/render.mjs"), cli, resolve(root, `examples/rail/${name}.json`), resolve(root, `docs/img/gallery-${name}.png`), String(scale)], { stdio: "inherit" });
  if (r.status !== 0) process.exit(r.status ?? 1);
}
