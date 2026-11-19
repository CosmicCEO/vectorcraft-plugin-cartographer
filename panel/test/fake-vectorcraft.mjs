// A stand-in for VectorCraft's control port: just enough of the protocol for the panel, and it records every command it is sent.
import { writeFileSync } from "node:fs";
import net from "node:net";
import { PLUGIN_ID, SET_SETTINGS, SET_CROSSING, SET_CROSS_SECTION, SET_EMBANKMENT, SET_HUB, SET_JUNCTION, SET_TRANSITION, SET_WATER, SET_WET, SET_JOINT, SET_LEVEL_CROSSING, SET_TRACKS, SET_YARD } from "../lib/doc.mjs";

export async function fakeVectorCraft({ paths = [], selection = [], spec = { segments: {} }, hasDocument = true, rejectYard = null, rejectJoint = null, rejectBuild = null } = {}) {
  const doc = { paths: paths.map((p) => ({ kind: "Path", ...p })), selection: [...selection], spec: structuredClone(spec), revision: 1, title: "Untitled", calls: [], layers: [{ id: 1000, name: "Layer 1", visible: true, locked: false, parent: null }], nextLayer: 1001 };
  const tree = (parent = null) => doc.layers.filter((l) => l.parent === parent).map((l) => ({ id: l.id, kind: "Layer", name: l.name, visible: l.visible, locked: l.locked, children: [...tree(l.id), ...doc.paths.filter((p) => (p.path ?? [1000]).at(-1) === l.id).map((p) => ({ id: p.id, kind: p.kind, name: p.name }))] }));
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
          if (params.kind) return reply(socket, id, { nodes: doc.paths.filter((p) => p.kind === params.kind).map((p) => ({ ...p, path: p.path ?? [1000] })) });
          return reply(socket, id, { revision: doc.revision, fileInfoTitle: doc.title, title: doc.fileName ?? doc.title, selection: doc.selection, layers: tree() });
        }
        if (method !== "engine.execute") return fail(socket, id, `unknown method ${method}`);
        const { command, params: p = {} } = params;
        doc.calls.push({ command, params: p });
        if (command === "select.set") doc.selection = [...p.ids];
        else if (command === "layer.new" || command === "layer.newSublayer") {
          const layer = { id: doc.nextLayer++, name: p.name, visible: true, locked: false, parent: command === "layer.newSublayer" ? p.parent : null };
          if (p.top) doc.layers.unshift(layer);
          else doc.layers.push(layer);
          return reply(socket, id, { id: layer.id });
        } else if (command === "layer.move") {
          for (const x of doc.paths.filter((q) => p.ids.includes(q.id))) x.path = [p.target];
          for (const l of doc.layers.filter((q) => p.ids.includes(q.id))) {
            doc.layers.splice(doc.layers.indexOf(l), 1);
            doc.layers.splice(doc.layers.findIndex((q) => q.id === p.target) + (p.place === "below" ? 1 : 0), 0, l);
          }
        } else if (command === "layer.setProps") {
          const l = doc.layers.find((q) => q.id === p.id);
          if (p.visible !== undefined) l.visible = p.visible;
          if (p.locked !== undefined) l.locked = p.locked;
        } else if (command === "select.all") doc.selection = doc.paths.map((x) => x.id);
        else if (command === SET_CROSS_SECTION) {
          const args = p.params;
          const sel = doc.paths.filter((x) => doc.selection.includes(x.id));
          for (const x of sel) {
            let seg = x.name.startsWith("road.") ? x.name.slice(5) : null;
            if (!seg || args.scope === "selected") {
              seg = `r${Object.keys(doc.spec.segments).length + 1}`;
              x.name = `road.${seg}`;
            }
            doc.spec.segments[seg] = { cross_section: args.cross_section, anchor: "auto" };
          }
          doc.revision++;
        } else if (command === SET_JUNCTION) {
          doc.spec.junctions ??= {};
          for (const x of doc.paths.filter((q) => doc.selection.includes(q.id))) {
            let id = x.name.startsWith("junction.") ? x.name.slice(9) : null;
            if (!id) {
              id = `j${Object.keys(doc.spec.junctions).length + 1}`;
              x.name = `junction.${id}`;
            }
            doc.spec.junctions[id] = JSON.parse(p.params.description);
          }
          doc.revision++;
        } else if (command === SET_CROSSING) {
          const roads = [...new Set(doc.paths.filter((q) => doc.selection.includes(q.id)).map((q) => q.name.slice(5)))];
          doc.spec.crossings ??= {};
          const known = Object.entries(doc.spec.crossings).find(([, c]) => roads.every((r) => c.roads.includes(r)))?.[0];
          doc.spec.crossings[known ?? `k${Object.keys(doc.spec.crossings).length + 1}`] = { ...JSON.parse(p.params.description), roads };
          doc.revision++;
        } else if (command === SET_TRANSITION) {
          const road = doc.paths.find((q) => doc.selection.includes(q.id))?.name.slice(5);
          const d = JSON.parse(p.params.description);
          doc.spec.transitions ??= {};
          const list = (doc.spec.transitions[road] ??= []).filter((e) => !((e.piece ?? 1) === (d.piece ?? 1) && Math.abs((e.at ?? -1e9) - (d.at ?? -1e9)) <= 5));
          if (!d.remove) list.push(d);
          doc.spec.transitions[road] = list;
          doc.revision++;
        } else if (command === SET_HUB) {
          const roads = [...new Set(doc.paths.filter((q) => doc.selection.includes(q.id)).map((q) => q.name.slice(5)))].sort();
          doc.spec.hubs ??= {};
          const known = Object.entries(doc.spec.hubs).find(([, h]) => roads.length === h.roads.length && roads.every((r) => h.roads.includes(r)))?.[0];
          doc.spec.hubs[known ?? `h${Object.keys(doc.spec.hubs).length + 1}`] = { ...JSON.parse(p.params.description), roads };
          doc.revision++;
        } else if (command === SET_WATER) {
          doc.spec.waters ??= {};
          for (const x of doc.paths.filter((q) => doc.selection.includes(q.id))) {
            let wid = x.name.startsWith("water.") ? x.name.slice(6) : null;
            if (!wid) {
              wid = `w${Object.keys(doc.spec.waters).length + 1}`;
              x.name = `water.${wid}`;
            }
            doc.spec.waters[wid] = JSON.parse(p.params.description);
          }
          doc.revision++;
        } else if (command === SET_WET) {
          const sel = doc.paths.filter((q) => doc.selection.includes(q.id));
          const road = sel.find((q) => q.name.startsWith("road."))?.name.slice(5);
          const water = sel.find((q) => q.name.startsWith("water."))?.name.slice(6);
          doc.spec.wets ??= {};
          const known = Object.entries(doc.spec.wets).find(([, w]) => w.road === road && w.water === water)?.[0];
          doc.spec.wets[known ?? `x${Object.keys(doc.spec.wets).length + 1}`] = { ...JSON.parse(p.params.description), road, water };
          doc.revision++;
        } else if (command === SET_EMBANKMENT) {
          const road = doc.paths.find((q) => doc.selection.includes(q.id))?.name.slice(5);
          const d = JSON.parse(p.params.description);
          doc.spec.embankments ??= {};
          const list = (doc.spec.embankments[road] ??= []).filter((e) => !((e.piece ?? 1) === (d.piece ?? 1) && Math.abs((e.from ?? -1e9) - (d.from ?? -1e9)) <= 5));
          if (!d.remove) list.push(d);
          doc.spec.embankments[road] = list;
          doc.revision++;
        } else if (command === SET_SETTINGS) {
          doc.spec = { ...doc.spec, schema: 1, settings: { ...(doc.spec.settings ?? {}), ...JSON.parse(p.params.description) } };
          doc.revision++;
        } else if (command === SET_TRACKS || command === SET_YARD) {
          if (command === SET_YARD && rejectYard) return fail(socket, id, rejectYard);
          const [kind, other] = command === SET_YARD ? ["yards", "tracks"] : ["tracks", "yards"];
          doc.spec[kind] ??= {};
          doc.spec[other] ??= {};
          for (const x of doc.paths.filter((q) => doc.selection.includes(q.id))) {
            let tid = x.name.startsWith("track.") ? x.name.slice(6) : null;
            if (!tid) {
              tid = `t${Object.keys({ ...doc.spec.tracks, ...doc.spec.yards }).length + 1}`;
              x.name = `track.${tid}`;
            }
            doc.spec[kind][tid] = JSON.parse(p.params.description);
            delete doc.spec[other][tid];
          }
          doc.revision++;
        } else if (command === SET_JOINT) {
          if (rejectJoint) return fail(socket, id, rejectJoint);
          const lines = doc.paths.filter((q) => doc.selection.includes(q.id) && q.name.startsWith("track.")).map((q) => q.name.slice(6));
          doc.spec.joints ??= {};
          const known = Object.entries(doc.spec.joints).find(([, j]) => lines.every((l) => j.lines.includes(l)))?.[0];
          doc.spec.joints[known ?? `j${Object.keys(doc.spec.joints).length + 1}`] = { ...JSON.parse(p.params.description), lines };
          doc.revision++;
        } else if (command === SET_LEVEL_CROSSING) {
          const sel = doc.paths.filter((q) => doc.selection.includes(q.id));
          const road = sel.find((q) => q.name.startsWith("road."))?.name.slice(5);
          const track = sel.find((q) => q.name.startsWith("track."))?.name.slice(6);
          doc.spec.level_crossings ??= {};
          const known = Object.entries(doc.spec.level_crossings).find(([, v]) => v.road === road && v.track === track)?.[0];
          doc.spec.level_crossings[known ?? `lc${Object.keys(doc.spec.level_crossings).length + 1}`] = { road, track, treatment: JSON.parse(p.params.description).treatment ?? "level" };
          doc.revision++;
        } else if (command === "file.saveCopy") {
          // The saved copy holds every path that has a geometry (the panel reads line geometry from it) and the plug-in's data.
          const nodes = doc.paths.filter((q) => q.geom).map((q) => ({ id: q.id, name: q.name, kind: { type: "path", path: q.geom } }));
          const body = JSON.stringify({ document: { layers: [{ kind: { type: "layer", children: nodes } }], unknown: { pluginData: { [PLUGIN_ID]: doc.spec } } } });
          writeFileSync(p.path, `${body}\n\u0000VCBLOBS\u0000%PDF-1.7 binary preview`, "latin1");
          return reply(socket, id, { path: p.path });
        } else if (command === "plugin.run") {
          if (rejectBuild) return fail(socket, id, rejectBuild);
          doc.revision++;
        }
        reply(socket, id, { ok: true });
      }
    });
  });
  await new Promise((ok) => server.listen(0, "127.0.0.1", ok));
  return { port: server.address().port, doc, close: () => server.close() };
}
