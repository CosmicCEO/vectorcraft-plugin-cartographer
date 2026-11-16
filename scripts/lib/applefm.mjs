// The optional Apple Intelligence helper (panel/apple-fm/main.swift): how to build it, or null when it cannot be built here.

/** `{build: [command, ...args]}` (run from the repository root) on macOS with `swiftc`, else null. */
export function appleHelperPlan(platform, hasSwiftc) {
  if (platform !== "darwin" || !hasSwiftc) return null;
  return { build: ["swiftc", "-parse-as-library", "-O", "-o", "target/apple-fm-agent", "panel/apple-fm/main.swift"] };
}
