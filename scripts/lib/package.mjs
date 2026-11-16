// Release package naming and checksums.
import { createHash } from "node:crypto";

/** One line of SHA256SUMS: `<hex>  <name>`. */
export function sumsLine(name, bytes) {
  return `${createHash("sha256").update(bytes).digest("hex")}  ${name}`;
}

/** `cartographer-<version>-<macos|linux>-<arm64|x86_64>.tar.gz`. */
export function archiveName(version, platform, arch) {
  const os = { darwin: "macos", linux: "linux" }[platform];
  if (!os) throw new Error("Cartographer packages are built on macOS and Linux.");
  return `cartographer-${version}-${os}-${arch === "x64" ? "x86_64" : arch}.tar.gz`;
}
