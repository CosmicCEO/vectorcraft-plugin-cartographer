// Reading what the panel needs from the live document: whether there is one, and the data the plug-in keeps with it.
import { existsSync, mkdtempSync, readFileSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { gunzipSync } from "node:zlib";

export const PLUGIN_ID = "com.cosmicceo.cartographer.build";
export const SET_SETTINGS = `plugin.${PLUGIN_ID}.set_settings`;

export async function inspectState(control) {
  const r = await control.call("document.inspect", { depth: 0 });
  return { revision: r.revision, title: r.fileInfoTitle ?? r.title ?? "", selection: r.selection ?? [] };
}

/** The JSON part of a saved .vectorcraft file (a binary preview may follow it, after a NUL and VCBLOBS). */
export function parseNativeFile(bytes) {
  const buf = bytes[0] === 0x1f && bytes[1] === 0x8b ? gunzipSync(bytes) : bytes;
  const text = buf.toString("latin1");
  const marker = text.indexOf("\u0000VCBLOBS");
  const end = (marker >= 0 ? text.lastIndexOf("}", marker) : text.lastIndexOf("}")) + 1;
  return JSON.parse(text.slice(0, end));
}

/** The plug-in's data stored in the document ({} when none), read from a saved copy. */
export async function readData(control) {
  const dir = mkdtempSync(join(tmpdir(), "cartographer-panel-"));
  const path = join(dir, "copy.vectorcraft");
  try {
    await control.run("file.saveCopy", { path });
    // The app may write the copy from a background thread: wait until it is there and has stopped growing.
    let last = -1;
    for (let i = 0; i < 60; i++) {
      const size = existsSync(path) ? statSync(path).size : 0;
      if (size > 0 && size === last) break;
      last = size;
      await new Promise((r) => setTimeout(r, 50));
    }
    const doc = parseNativeFile(readFileSync(path)).document ?? {};
    return doc.unknown?.pluginData?.[PLUGIN_ID] ?? {};
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}
