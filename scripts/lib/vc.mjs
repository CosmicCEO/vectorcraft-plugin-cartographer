// How to start VectorCraft with its control port, on each system.

/** `{cmd, args}` to start the application, or `{error}` saying what to set. */
export function appLaunch(platform, env, exists, port) {
  if (platform === "darwin") return { cmd: "open", args: ["-a", "VectorCraft", "--args", "--control", String(port)] };
  if (platform !== "linux") return { error: "Cartographer's scripts support macOS and Linux." };
  const args = ["--control", String(port)];
  if (env.VECTORCRAFT_APP && exists(env.VECTORCRAFT_APP)) return { cmd: env.VECTORCRAFT_APP, args };
  for (const dir of (env.PATH ?? "").split(":").filter(Boolean)) {
    if (exists(`${dir}/vectorcraft`)) return { cmd: `${dir}/vectorcraft`, args };
  }
  return { error: "Could not find VectorCraft. Set VECTORCRAFT_APP to the AppImage (or program) path, or put `vectorcraft` on your PATH." };
}

/** Waits until something answers on 127.0.0.1:`port`, trying every 250 ms for up to `ms` milliseconds. Resolves true or false. */
export async function waitForPort(port, ms) {
  const { connect } = await import("node:net");
  const once = () => new Promise((done) => {
    const s = connect(port, "127.0.0.1");
    s.setTimeout(500, () => (s.destroy(), done(false)));
    s.on("connect", () => (s.destroy(), done(true)));
    s.on("error", () => done(false));
  });
  for (const end = Date.now() + ms; ; ) {
    if (await once()) return true;
    if (Date.now() >= end) return false;
    await new Promise((r) => setTimeout(r, 250));
  }
}

/** Is VectorCraft running already? Checked on macOS only (`pgrep -x VectorCraft`); `run` is spawnSync, replaceable in tests. */
export function isRunning(platform, run) {
  if (platform !== "darwin") return false;
  return run("pgrep", ["-x", "VectorCraft"]).status === 0;
}

