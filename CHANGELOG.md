# Changelog

## 0.1.0 Foundation
- One Cargo workspace (`core`, `filter`, `cli`) with the framework merged from Road Builder and Track Layer: plug-in glue, geometry, offsetting, text output, document reading (closed shapes are told from open lines).
- Units (1 document point is 1 metre) and the target print scale (`true` or `1:N`).
- Versioned document storage that keeps unknown data and refuses a newer schema.
- The material library schema with twelve seed materials, all unverified.
- Command Set Settings (target print scale, style, seed, north) and the native `cartographer-cli` (`version`, `parse-settings`, `schema settings`).
- A companion panel with a settings card built from a schema, on port 8789.
- Portable Node scripts (check, build, install, package, start VectorCraft, control-port client) and CI on Ubuntu and macOS.
- Spike results for S1 to S4 in `docs/spikes/`.
