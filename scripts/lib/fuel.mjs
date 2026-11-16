// Reading the fuel tool's output and comparing two builds' cost.

/** The fuel number from `tools/fuel`'s line (`fuel <n> output_bytes <m> ...`). */
export function parseFuel(line) {
  const m = /^fuel (\d+)/.exec(line.trim());
  if (!m) throw new Error(`no fuel figure in: ${line.slice(0, 80)}`);
  return Number(m[1]);
}

/** `{ok, ratio}`: is the new cost at most `limit` times the old? */
export function withinParity(oldFuel, newFuel, limit) {
  const ratio = newFuel / oldFuel;
  return { ok: ratio <= limit, ratio };
}
