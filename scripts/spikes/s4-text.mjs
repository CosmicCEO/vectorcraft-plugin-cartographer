// Spike S4: can a filter create a text object? Builds spikes/text-filter, installs it in a headless session and tries each text kind.
//   node scripts/spikes/s4-text.mjs <vectorcraft-cli>
import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { headless } from "../lib/headless.mjs";

const [cli] = process.argv.slice(2);
if (!cli) {
  console.error("usage: s4-text.mjs <vectorcraft-cli>");
  process.exit(2);
}
const root = resolve(import.meta.dirname, "../..");
const build = spawnSync("cargo", ["+stable", "build", "--release", "--target", "wasm32-unknown-unknown", "--manifest-path", "spikes/text-filter/Cargo.toml"], { cwd: root, stdio: "inherit" });
if (build.status !== 0) process.exit(1);
const out = join(root, "target/spikes/s4");
mkdirSync(out, { recursive: true });
const wasm = join(out, "text_filter.wasm");
copyFileSync(join(root, "spikes/text-filter/target/wasm32-unknown-unknown/release/text_filter.wasm"), wasm);

const h = await headless(cli, root);
await h.run("plugin.install", { path: wasm, replace: true });
for (const kind of ["point", "area", "path"]) {
  await h.run("file.new", { width: 600, height: 400 });
  const drawn = await h.tool("draw_path", { points: [{ x: 50, y: 300 }, { x: 300, y: 200 }, { x: 550, y: 300 }] });
  await h.run("select.set", { ids: [drawn.id ?? drawn.ids?.[0]] });
  let result;
  try {
    await h.run("plugin.run", { id: "com.cosmicceo.spike.text", params: { kind } });
    const all = await h.tool("doc_inspect", { limit: 1000 });
    const found = JSON.stringify(all).includes("gen.label.spike");
    result = found ? "CREATED (object present afterwards)" : "ran without error but no text object found";
  } catch (e) {
    result = `REFUSED: ${String(e.message).slice(0, 300)}`;
  }
  console.log(`- text kind "${kind}": ${result}`);
}
h.close();
