// Installs the plug-in in a headless VectorCraft session (macOS), draws a line, runs Set Settings, saves a copy and checks the settings are in it.
//   node scripts/smoke-headless.mjs <vectorcraft-cli>
import { copyFileSync, mkdirSync, readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { gunzipSync } from "node:zlib";
import { headless } from "./lib/headless.mjs";

const [cli] = process.argv.slice(2);
if (!cli) {
  console.error("usage: smoke-headless.mjs <vectorcraft-cli>");
  process.exit(2);
}
const root = resolve(import.meta.dirname, "..");
const out = join(root, "target/smoke");
mkdirSync(out, { recursive: true });
const wasm = join(out, "cartographer_filter.wasm");
copyFileSync(join(root, "target/wasm32-unknown-unknown/release/cartographer_filter.wasm"), wasm);

const h = await headless(cli, root);
await h.run("plugin.install", { path: wasm, replace: true });
await h.run("file.new", { width: 600, height: 400 });
await h.tool("draw_path", { points: [{ x: 50, y: 300 }, { x: 550, y: 100 }] });
await h.run("select.all");
await h.run("plugin.com.cosmicceo.cartographer.build.set_settings", { params: { description: '{"style":"hand-drawn","target_print_scale":"1:5000","seed":7}' } });
const copy = join(out, "smoke.vectorcraft");
await h.run("file.saveCopy", { path: copy });
h.close();

const raw = readFileSync(copy);
let text = raw.toString("latin1");
try {
  text += gunzipSync(raw).toString("latin1");
} catch {}
const squeezed = text.replace(/\s+/g, "");
const ok = ["\"target_print_scale\":\"1:5000\"", "\"style\":\"hand-drawn\"", "\"seed\":7", "\"schema\":1"].every((s) => squeezed.includes(s));
console.log(ok ? "ok: the saved copy holds the settings" : "FAILED: settings not found in the saved copy");
process.exit(ok ? 0 : 1);
