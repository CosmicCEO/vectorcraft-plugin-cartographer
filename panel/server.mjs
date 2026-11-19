#!/usr/bin/env node
// The companion panel: a small local web page beside VectorCraft that follows your selection, shows the road's elements,
// takes a description in words, and draws it. Zero dependencies.
//
//   Start VectorCraft with its control port (see docs/GETTING-STARTED.md; loopback only), then:
//   node panel/server.mjs [--control 7979] [--port 8789] [--agent claude]
//   then open http://127.0.0.1:8789
import "../scripts/lib/require-node.mjs";
import { createServer as httpServer } from "node:http";
import { readFile } from "node:fs/promises";
import { extname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { Control } from "./lib/control.mjs";
import { Panel } from "./lib/panel.mjs";
import { loadProviders } from "./lib/providers.mjs";

const PUBLIC = resolve(import.meta.dirname, "public");
const TYPES = { ".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8", ".css": "text/css; charset=utf-8" };

/** The HTTP side. Only answers requests that name this server (Host) and, for changes, that come from its own page (Origin). */
export function createServer({ panel, port }) {
  const hosts = new Set([`127.0.0.1:${port}`, `localhost:${port}`]);
  const json = (res, code, body) => {
    res.writeHead(code, { "content-type": "application/json", "cache-control": "no-store" });
    res.end(JSON.stringify(body));
  };
  const readBody = (req) =>
    new Promise((done, fail) => {
      let t = "";
      req.on("data", (d) => {
        t += d;
        if (t.length > 100_000) fail(new Error("request too large"));
      });
      req.on("end", () => {
        try {
          done(t ? JSON.parse(t) : {});
        } catch {
          fail(new Error("the request was not valid JSON"));
        }
      });
    });

  const actions = {
    propose: (b) => panel.propose(String(b.description ?? "").slice(0, 2000), b.agent),
    edit: (b) => panel.edit(b.op),
    text: (b) => panel.setText(String(b.text ?? "").slice(0, 1000)),
    draw: (b) => panel.draw({ scope: b.scope }),
    cancel: () => panel.cancel(),
    selectline: () => panel.selectLine(),
    mode: (b) => panel.setMode(String(b.mode)),
    match: () => panel.match(),
    junction: (b) => panel.editJunction(b.patch ?? {}),
    crossing: (b) => panel.editCrossing(b.patch ?? {}),
    hub: (b) => panel.editHub(b.patch ?? {}),
    transition: (b) => panel.editTransition(b.patch ?? {}),
    removetransition: (b) => panel.removeTransition(Number(b.index)),
    track: (b) => panel.editTrack(b.patch ?? {}),
    joint: (b) => panel.editJoint(b.patch ?? {}),
    level: (b) => panel.editLevel(b.patch ?? {}),
    water: (b) => panel.editWater(b.patch ?? {}),
    wet: (b) => panel.editWet(b.patch ?? {}),
    embank: (b) => panel.editEmbank(b.patch ?? {}),
    removeembank: (b) => panel.removeEmbank(Number(b.index)),
    agent: (b) => panel.setAgent(String(b.id)),
    opensettings: () => panel.openSettings(),
    closesettings: () => panel.closeSettings(),
    settings: (b) => panel.editSettings(b.patch ?? {}),
    drawsettings: () => panel.drawSettings(),
    cancelsettings: () => panel.cancelSettings(),
  };

  return httpServer(async (req, res) => {
    if (!hosts.has(req.headers.host ?? "")) return json(res, 403, { error: "unexpected Host header" });
    try {
      const url = new URL(req.url, `http://${req.headers.host}`);
      if (req.method === "GET" && url.pathname === "/events") {
        res.writeHead(200, { "content-type": "text/event-stream", "cache-control": "no-store", connection: "keep-alive" });
        const send = (s) => res.write(`data: ${JSON.stringify(s)}\n\n`);
        send(panel.snapshot());
        panel.on("change", send);
        req.on("close", () => panel.off("change", send));
        return;
      }
      if (req.method === "GET" && url.pathname === "/state") return json(res, 200, panel.snapshot());
      if (req.method === "POST" && url.pathname.startsWith("/api/")) {
        const origin = req.headers.origin;
        if ((origin && !hosts.has(new URL(origin).host)) || !String(req.headers["content-type"] ?? "").startsWith("application/json")) return json(res, 403, { error: "requests must come from the panel page" });
        const name = url.pathname.slice(5);
        const action = Object.hasOwn(actions, name) ? actions[name] : null;
        if (!action) return json(res, 404, { error: "unknown action" });
        await action(await readBody(req));
        return json(res, 200, panel.snapshot());
      }
      if (req.method === "GET") {
        const file = url.pathname === "/" ? "index.html" : url.pathname.slice(1);
        if (!/^[\w.-]+$/.test(file) || !TYPES[extname(file)]) return json(res, 404, { error: "not found" });
        const body = await readFile(resolve(PUBLIC, file));
        // Only forbids inline script and plug-ins; styles are left alone because the page sets them from script.
        res.writeHead(200, { "content-type": TYPES[extname(file)], "cache-control": "no-store", "content-security-policy": "script-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'" });
        return res.end(body);
      }
      json(res, 405, { error: "method not allowed" });
    } catch (e) {
      json(res, e.code === "ENOENT" ? 404 : 400, { error: e.message });
    }
  });
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const arg = (name, fallback) => {
    const i = process.argv.indexOf(`--${name}`);
    return i > 0 ? process.argv[i + 1] : fallback;
  };
  const port = Number(arg("port", 8789));
  const providers = loadProviders();
  for (const w of providers.warnings ?? []) console.warn(`warning: ${w}`);
  if (arg("agent")) providers.default = arg("agent");
  const panel = new Panel({ control: new Control({ port: Number(arg("control", 7979)) }), providers });
  const server = createServer({ panel, port });
  server.on("error", (e) => {
    console.error(e.code === "EADDRINUSE" ? `fix: port ${port} is already in use (another panel is probably running). Stop it, or start this one with --port <another port>.` : `fix: the panel could not start: ${e.message}`);
    process.exit(1);
  });
  server.listen(port, "127.0.0.1", () => {
    console.log(`Cartographer panel: http://127.0.0.1:${port}   (VectorCraft control port ${arg("control", 7979)}, agent ${providers.default})`);
    panel.start();
  });
}
