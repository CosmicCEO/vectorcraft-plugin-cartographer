// A stand-in for VectorCraft's control port: just enough of the protocol for the panel, and it records every engine command it is sent.
import { writeFileSync } from "node:fs";
import net from "node:net";
import { PLUGIN_ID, SET_SETTINGS } from "../lib/doc.mjs";

export async function fakeVectorCraft({ paths = [], selection = [], spec = {}, hasDocument = true } = {}) {
  const doc = { paths: paths.map((p) => ({ kind: "Path", ...p })), selection: [...selection], spec: structuredClone(spec), revision: 1, title: "Untitled", calls: [] };
  const reply = (socket, id, result) => socket.write(`${JSON.stringify({ id, ok: true, result })}\n`);
  const fail = (socket, id, error) => socket.write(`${JSON.stringify({ id, ok: false, error })}\n`);

  const server = net.createServer((socket) => {
    let buf = "";
    socket.setEncoding("utf8");
    socket.on("data", (chunk) => {
      buf += chunk;
      for (let i = buf.indexOf("\n"); i >= 0; i = buf.indexOf("\n")) {
        const { id, method, params } = JSON.parse(buf.slice(0, i));
        buf = buf.slice(i + 1);
        if (!hasDocument && method === "document.inspect") return fail(socket, id, "no active document");
        if (method === "document.inspect") {
          if (params.ids) return reply(socket, id, { nodes: doc.paths.filter((p) => params.ids.includes(p.id)) });
          if (params.kind) return reply(socket, id, { nodes: doc.paths });
          return reply(socket, id, { revision: doc.revision, fileInfoTitle: doc.title, selection: doc.selection, layers: [] });
        }
        if (method !== "engine.execute") return fail(socket, id, `unknown method ${method}`);
        const { command, params: p = {} } = params;
        doc.calls.push({ command, params: p });
        if (command === "select.set") doc.selection = [...p.ids];
        else if (command === "select.all") doc.selection = doc.paths.map((x) => x.id);
        else if (command === SET_SETTINGS) {
          doc.spec = { ...doc.spec, schema: 1, settings: { ...(doc.spec.settings ?? {}), ...JSON.parse(p.params.description) } };
          doc.revision++;
        } else if (command === "file.saveCopy") {
          const body = JSON.stringify({ document: { unknown: { pluginData: { [PLUGIN_ID]: doc.spec } } } });
          writeFileSync(p.path, `${body}\n\u0000VCBLOBS\u0000%PDF-1.7 binary preview`, "latin1");
          return reply(socket, id, { path: p.path });
        }
        reply(socket, id, { ok: true });
      }
    });
  });
  await new Promise((ok) => server.listen(0, "127.0.0.1", ok));
  return { port: server.address().port, doc, close: () => server.close() };
}
