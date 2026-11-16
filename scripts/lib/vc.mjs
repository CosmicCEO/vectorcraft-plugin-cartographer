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
