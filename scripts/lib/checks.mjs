// What the install check looks at, as a pure function of what was probed.

const NODE_MIN = 22;

/** `probe`: {nodeVersion, hasRustup, hasWasmTarget, hasCli, controlReachable}. Returns [{name, ok, fix?, optional?}]. */
export function checks(probe) {
  const major = Number(/^v?(\d+)\./.exec(probe.nodeVersion ?? "")?.[1]);
  return [
    { name: `Node ${NODE_MIN} or newer`, ok: major >= NODE_MIN, ...(major >= NODE_MIN ? {} : { fix: `Install Node ${NODE_MIN} or newer (https://nodejs.org); you have ${probe.nodeVersion}.` }) },
    { name: "Rust toolchain (rustup, stable)", ok: !!probe.hasRustup, ...(probe.hasRustup ? {} : { fix: "Install rustup (https://rustup.rs) and run: rustup toolchain install stable" }) },
    { name: "WebAssembly target", ok: !!probe.hasWasmTarget, ...(probe.hasWasmTarget ? {} : { fix: "Run: rustup target add wasm32-unknown-unknown --toolchain stable" }) },
    { name: "vectorcraft-cli (headless renders, macOS only)", ok: !!probe.hasCli, optional: true, ...(probe.hasCli ? {} : { fix: "Optional. On macOS, download vectorcraft-cli from the VectorCraft release; there is no Linux build." }) },
    { name: "VectorCraft running with its control port", ok: !!probe.controlReachable, optional: true, ...(probe.controlReachable ? {} : { fix: "Optional. Start VectorCraft with its control port (see docs/GETTING-STARTED.md)." }) },
  ];
}
