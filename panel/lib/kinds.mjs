// The two kinds of line the plug-in builds from: what names a line, what its built shapes are called, and the layer they live in.
import { segmentId, trackId } from "./doc.mjs";

export const KINDS = {
  road: { line: segmentId, generated: /^gen\.road\./, layer: (id) => `road ${id}` },
  track: { line: trackId, generated: /^gen\.track\./, layer: (id) => `track ${id}` },
};

/** "road" or "track" for the name of a line you drew, else null (built shapes and other names are not lines). */
export function kindOfLine(name = "") {
  return Object.keys(KINDS).find((kind) => KINDS[kind].line(name)) ?? null;
}
