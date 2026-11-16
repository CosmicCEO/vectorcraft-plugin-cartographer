// The environment for running cargo. On macOS, Homebrew's rustc shadows rustup's (and has no WebAssembly target), so rustup's bin comes first.

/** Returns a copy of `env`; on macOS with Homebrew's rustup present, its bin directory is first on PATH. Linux is left as it is. */
export function rustEnv(platform, env, exists) {
  const out = { ...env };
  if (platform !== "darwin") return out;
  const dir = ["/opt/homebrew/opt/rustup/bin", "/usr/local/opt/rustup/bin"].find((d) => exists(d));
  if (dir) out.PATH = env.PATH ? `${dir}:${env.PATH}` : dir;
  return out;
}
