import assert from "node:assert/strict";
import { test } from "node:test";
import { headless } from "../lib/headless.mjs";

// A stand-in for `vectorcraft-cli mcp`: answers initialize, and every tool call with {ok: 1, id} (or an error result for the tool "bad").
const SERVER = `
const rl = require("node:readline").createInterface({ input: process.stdin });
rl.on("line", (l) => {
  const m = JSON.parse(l);
  if (m.id === undefined) return;
  if (m.method === "initialize") return console.log(JSON.stringify({ jsonrpc: "2.0", id: m.id, result: {} }));
  const bad = m.params.name === "bad";
  console.log(JSON.stringify({ jsonrpc: "2.0", id: m.id, result: { isError: bad, content: [{ text: bad ? "no such thing" : JSON.stringify({ ok: 1, id: m.id, args: m.params.arguments }) }] } }));
});`;

const open = () => headless(process.execPath, "/tmp", ["-e", SERVER]);

test("headless frames requests and matches replies by id", async () => {
  const h = await open();
  const [a, b] = await Promise.all([h.tool("x", { n: 1 }), h.tool("y", { n: 2 })]);
  assert.equal(a.ok, 1);
  assert.deepEqual(a.args, { n: 1 });
  assert.deepEqual(b.args, { n: 2 });
  assert.notEqual(a.id, b.id);
  h.close();
});

test("an error result rejects with the host's text", async () => {
  const h = await open();
  await assert.rejects(h.tool("bad", {}), /no such thing/);
  h.close();
});

test("run sends a command through the command_run tool", async () => {
  const h = await open();
  const r = await h.run("select.all", { a: 1 });
  assert.deepEqual(r.args, { id: "select.all", params: { a: 1 } });
  h.close();
});
