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

test("the page, script and style are served; nothing else is", async () => {
  const { base } = await start();
  const page = await fetch(`${base}/`);
  assert.equal(page.status, 200);
  assert.match(await page.text(), /<title>Cartographer<\/title>/);
  assert.match((await fetch(`${base}/app.js`)).headers.get("content-type"), /javascript/);
  assert.equal((await fetch(`${base}/style.css`)).status, 200);
  assert.equal((await fetch(`${base}/..%2Fserver.mjs`)).status, 404, "no path traversal");
  assert.equal((await fetch(`${base}/package.json`)).status, 404);
});

test("a request that does not name this server is refused with 403", async () => {
  const { port } = await start();
  const status = (host) => new Promise((ok, fail) => {
    const r = request({ host: "127.0.0.1", port, path: "/state", headers: { host } }, (res) => { res.resume(); ok(res.statusCode); });
    r.on("error", fail);
    r.end();
  });
  assert.equal(await status(`evil.example:${port}`), 403, "a foreign Host header (DNS rebinding) is refused");
  assert.equal(await status(`127.0.0.1:${port}`), 200);
  assert.equal(await status(`localhost:${port}`), 200);
});

test("changes need JSON from the panel's own page", async () => {
  const { base } = await start();
  const noType = await fetch(`${base}/api/cancel`, { method: "POST", headers: { "content-type": "text/plain" }, body: "{}" });
  assert.equal(noType.status, 403);
  assert.equal((await post(base, "/api/cancel", {}, { origin: "https://evil.example" })).status, 403);
  assert.equal((await post(base, "/api/cancel", {}, { origin: base })).status, 200);
  assert.equal((await post(base, "/api/nothing", {}, { origin: base })).status, 404);
});

test("propose and draw settings over HTTP", async () => {
  const { base, vc } = await start();
  const r = await post(base, "/api/settings", { patch: { style: "black-white" } }, { origin: base });
  assert.equal(r.status, 200);
  assert.equal((await r.json()).pending.settings.style, "black-white");
  await post(base, "/api/draw", {}, { origin: base });
  assert.equal(vc.doc.spec.settings.style, "black-white");
});

test("the live feed starts with the current state", async () => {
  const { base } = await start();
  const res = await fetch(`${base}/events`);
  const chunk = new TextDecoder().decode((await res.body.getReader().read()).value);
  assert.match(chunk, /^data: \{/);
  assert.match(chunk, /"settings"/);
});
