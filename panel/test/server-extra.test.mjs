import assert from "node:assert/strict";
import { request } from "node:http";
import net from "node:net";
import { after, test } from "node:test";
import { Control } from "../lib/control.mjs";
import { Panel } from "../lib/panel.mjs";
import { createServer } from "../server.mjs";
import { fakeVectorCraft } from "./fake-vectorcraft.mjs";

const cleanups = [];
after(() => cleanups.forEach((f) => f()));

const freePort = () => new Promise((ok) => { const s = net.createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => ok(p)); }); });

async function start() {
  const vc = await fakeVectorCraft({ paths: [{ id: 5, name: "" }], selection: [5] });
  const control = new Control({ port: vc.port });
  const panel = new Panel({ control });
  await panel.tick();
  const port = await freePort();
  const server = createServer({ panel, port });
  await new Promise((ok) => server.listen(port, "127.0.0.1", ok));
  cleanups.push(() => { server.closeAllConnections(); server.close(); control.close(); vc.close(); });
  return { base: `http://127.0.0.1:${port}`, port, vc };
}

const post = (base, path, body, headers = {}) => fetch(`${base}${path}`, { method: "POST", headers: { "content-type": "application/json", ...headers }, body: JSON.stringify(body) });

test("the settings card over HTTP: open it, preview a change, draw it", async () => {
  const { base, vc } = await start();
  const own = { origin: base };
  assert.equal((await post(base, "/api/opensettings", {}, own)).status, 200);
  const r = await post(base, "/api/settings", { patch: { style: "black-white" } }, own);
  assert.equal(r.status, 200);
  assert.equal((await r.json()).settingsPending.settings.style, "black-white");
  await post(base, "/api/drawsettings", {}, own);
  assert.equal(vc.doc.spec.settings.style, "black-white");
  const closed = await (await post(base, "/api/closesettings", {}, own)).json();
  assert.equal(closed.settings, null);
});

test("a malformed address cannot crash the server", async () => {
  const { port } = await start();
  const status = (path) => new Promise((ok, fail) => {
    const r = request({ host: "127.0.0.1", port, path, headers: { host: `127.0.0.1:${port}` } }, (res) => { res.resume(); ok(res.statusCode); });
    r.on("error", fail);
    r.end();
  });
  const bad = await status("//[");
  assert.ok(bad >= 400 && bad < 500, `got ${bad}`);
  assert.equal(await status("/state"), 200, "the server is still alive");
});
