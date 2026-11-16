// The Node version the scripts and the panel need, as one plain line.

const MIN = 22;

/** null when `version` (like "v22.1.0") is new enough, else one plain line saying what to do. */
export function nodeProblem(version) {
  const major = Number(/^v?(\d+)\./.exec(version ?? "")?.[1]);
  return major >= MIN ? null : `Cartographer's scripts need Node ${MIN} or newer; this is ${version}. Install a current Node (https://nodejs.org).`;
}
