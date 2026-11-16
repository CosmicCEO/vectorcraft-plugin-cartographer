import assert from "node:assert/strict";
import net from "node:net";
import { test } from "node:test";
import { call } from "../lib/control.mjs";

const serve = (reply) =>
  new Promise((ok) => {
    const server = net.createServer((s) => s.on("data", (d) => s.write(`${JSON.stringify({ id: 1, ...reply(JSON.parse(d.toString().split("\n")[0])) })}\n`)));
    server.listen(0, "127.0.0.1", () => ok(server));
  });

test("call sends the method and params and returns the result", async () => {
  const server = await serve((req) => ({ ok: true, result: { got: req.method, params: req.params } }));
  const r = await call("engine.execute", { command: "select.all" }, server.address().port);
  assert.deepEqual(r, { got: "engine.execute", params: { command: "select.all" } });
  server.close();
});

test("an error reply rejects with its text", async () => {
  const server = await serve(() => ({ ok: false, error: "no active document" }));
  await assert.rejects(call("x", {}, server.address().port), /no active document/);
  server.close();
});

test("a closed port is a plain message that names the port", async () => {
  await assert.rejects(call("x", {}, 1), /cannot reach VectorCraft on port 1/);
});
