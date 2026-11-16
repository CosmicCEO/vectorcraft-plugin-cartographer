# Spike S4: can a filter create text objects?

Run on 2026-11-16 against VectorCraft 0.24.1 (headless, macOS) with a throwaway filter (`spikes/text-filter`, run by `node scripts/spikes/s4-text.mjs <vectorcraft-cli>`) that returns the kept objects plus one new object of `"type": "type"` shaped as the plug-in docs describe (`text.kind` point, area and path).

## Result: refused, all three kinds

```
- text kind "point": REFUSED: command_run: plug-in output: objects[1]: `type` must be "path" or "compound"
- text kind "area":  REFUSED: command_run: plug-in output: objects[1]: `type` must be "path" or "compound"
- text kind "path":  REFUSED: command_run: plug-in output: objects[1]: `type` must be "path" or "compound"
```

The plug-in docs say the same in words: text objects are read only to a filter, and a new object of that type is refused. A filter can read existing text (and must list it back unchanged) but never create or edit it.

## Decision for the labels release

Two ways remain; the labels release decides between them with a second spike:

1. **Outlined glyph paths** made by the filter from a bundled open-licence font. Always works, in every export, at A0 and N scale. It costs fuel (each glyph is a number-heavy path) and the text is no longer editable text.
2. **The panel creates real text objects** through VectorCraft's own commands over the control port (the panel is already a control-port client, and VectorCraft's tools include adding text and wrapping text to a path). Editable and small, but made by the panel, not by a build, so a rebuild must not delete or duplicate them, and they exist only where the panel was used.

My lean is option 2 for names along lines (editable, light on fuel) and option 1 only for the scale bar and legend glyphs if they must be rebuilt by the filter. The data reserved now (a `name` on every feature, a text layer id) serves both.

Nothing in the first release depends on this: version 1.0 has no labels.
