// A client for a running VectorCraft's control port (start the app with its control port; see docs/GETTING-STARTED.md).
import net from "node:net";

/** Calls `method` and resolves with the reply's result; rejects with the reply's error or a plain "cannot reach" message. */
export function call(method, params = {}, port = Number(process.env.VECTORCRAFT_CONTROL_PORT ?? 7979)) {
  return new Promise((done, fail) => {
    const socket = net.connect(port, "127.0.0.1");
    let buffer = "";
    socket.on("error", (e) => fail(new Error(`cannot reach VectorCraft on port ${port}: ${e.message}`)));
    socket.on("connect", () => socket.write(`${JSON.stringify({ id: 1, method, params })}\n`));
    socket.on("data", (chunk) => {
      buffer += chunk;
      if (!buffer.includes("\n")) return;
      const reply = JSON.parse(buffer.split("\n")[0]);
      socket.end();
      if (reply.ok) done(reply.result);
      else fail(new Error(reply.error));
    });
  });
}
