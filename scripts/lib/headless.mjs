// A client for `vectorcraft-cli mcp` over stdio (a headless VectorCraft session; macOS only today). Never touches the user's open document.
import { spawn } from "node:child_process";

/** Starts a session confined to `root` for reading and writing files. `argv` replaces the command line (for tests). */
export async function headless(cli, root, argv = ["mcp", "--automation-read-root", root, "--automation-write-root", root]) {
  const child = spawn(cli, argv, { stdio: ["pipe", "pipe", "inherit"] });
  let buffer = "";
  let next = 1;
  const waiting = new Map();
  child.stdout.on("data", (chunk) => {
    buffer += chunk;
    for (let nl = buffer.indexOf("\n"); nl >= 0; nl = buffer.indexOf("\n")) {
      const line = buffer.slice(0, nl);
      buffer = buffer.slice(nl + 1);
      if (!line.trim()) continue;
      const msg = JSON.parse(line);
      waiting.get(msg.id)?.(msg);
      waiting.delete(msg.id);
    }
  });
  const call = (method, params) =>
    new Promise((done) => {
      const id = next++;
      waiting.set(id, done);
      child.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", id, method, params })}\n`);
    });
  const tool = async (name, args) => {
    const r = await call("tools/call", { name, arguments: args });
    if (r.error) throw new Error(JSON.stringify(r.error));
    const text = r.result.content?.[0]?.text;
    if (r.result.isError) throw new Error(text);
    try {
      return JSON.parse(text);
    } catch {
      return text;
    }
  };
  await call("initialize", { protocolVersion: "2025-03-26", capabilities: {}, clientInfo: { name: "cartographer", version: "0" } });
  child.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" })}\n`);
  return { tool, run: (command, params = {}) => tool("command_run", { id: command, params }), close: () => child.kill() };
}
