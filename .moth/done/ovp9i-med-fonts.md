drawio supports custom fonts
we start with ~/Downloads/elms-sans.zip

also the minimal width for generated png should be 1024px (including the goldens)

----- AI agent updates -------

## Decisions

- Elms Sans (zip from the font's distribution, with its LICENSE) is the font of all diagram text: node and group text, edge labels, diagram title and description. Rejected: node and group text only (mixed fonts in one picture).
- Two weights, two drawio font families, both Elms Sans, latin subset, `normal` style, woff2: weight 100 for regular text (role, description, edge labels, description of the diagram) and weight 400 for names (node name, group name, diagram title). A name is no longer marked bold (`fontStyle=1` dropped): the 400 file is the "strong" face, so drawio does not fake a bold over it. Rejected: variable woff2 (drawio support unclear), latin-ext/vietnamese subsets (not needed now; characters outside latin fall back to the viewer's default font), the italic styles (unused).
- The font reaches the output by embedding: each text cell names its family (`fontFamily`) and the font data travels inside the .drawio as a `fontSource` data URI (`data:font/woff2;base64,...`, url-encoded in the style), which the drawio CLI turns into an `@font-face` in the exported SVG and uses for the PNG. Verified by experiment with the drawio CLI (SVG and PNG). A family's `fontSource` need only be declared on one cell of the file; the other cells of that family just name the family. So the font data is written once per family (not per text cell, which would multiply the file size by the number of texts). Rejected: referencing a path/URL (output not self-contained), relying on the font being installed on the viewing machine.
- The font files are vendored in the repo (only the two used) and compiled into the binary, like the icons (self-contained output, nothing external). The upstream zip stays outside the repo; its license text is kept next to the vendored files.
- The text-width estimate used for line breaking and node/group sizing (c43-layout `drawio/text.rs`, tuned for Helvetica) is recalibrated to Elms Sans (geometric; the 100 weight comes out narrower than the Helvetica estimate, the 400 weight about as wide). Metrics are measured from the two font files, per weight. Node sizes, wrapped lines, group text widths and the layout therefore change: goldens are regenerated. Rejected: keeping the Helvetica estimate (nodes and labels would be sized for the wrong font).
- The font declaration belongs to the drawio rendering in c43-layout (the title, description and edge label cells are produced there), as an opt-in on the styling strategy so clarc and plain c43-layout output stays unchanged unless a font is given; c43 supplies the Elms Sans fonts through its pretty strategy. c43-layout's pin in c43 is bumped (commits touching c43-layout carry `[c43-layout][ovp9i]`).
- PNG minimum width 1024 px: the png renders of the goldens and of any drawio file (`scripts/render.sh`, the only place PNGs are generated; none are committed) are scaled up so the image is at least 1024 px wide, i.e. scale = max(1, 1024 / diagram width); the existing cap of about 4000 px on the longer side stays for very large diagrams. Small diagrams are never rendered below 1024 px wide; SVG output is unaffected (vector). Rejected: a PNG output in the c43 CLI (not a feature of c43).
- Strokes halved (supersedes the widths of f8ew1's decision, ratio kept): connection line and group frame 0.5 px, node outline 2 px, outer diagram border 1 px, i.e. still 4:2:1 (node : outer border : connection), t = 0.5 px. The strokes of the icons' parts are halved (icon files: 3.1 → 1.55), so icons keep their weight relative to the node. Icon placement with the thinner outline (supersedes f8ew1's "sit line on the outline"): the sit line lies half the node outline plus half the icon stroke below the node's top edge, so the icon's bottom line lies just inside the node, on the inner edge of the outline (the icon had looked slightly too high).
- Edge labels split earlier: a label wider than 48 px (about the lane an edge runs in between two nodes; was 80 px with the Helvetica estimate, which came out wider than Elms Sans Thin) is split into up to 3 lines, so a label never outruns its arrow. The split width is a parameter of the face-aware split function in c43-layout; clarc keeps 80. In the hello world mermaid both "says hello" and "persists hellos" are two lines.
- Text width follows the font drawn: Elms Sans weight 400 for names (title, node and group name), weight 100 for all other text (role, description, edge labels, diagram description); Helvetica estimates apply only when no font is given (clarc).

## Open points

- The embedded font shows in browsers and in the PNG, but non-browser SVG viewers (e.g. Inkscape, librsvg-based previews) ignore `@font-face` data URIs and fall back to a default font: the font has to be installed there. Rejected for now: outlining text to paths in the render script (loses selectable text); can be added as a second output if needed.
- The 100 weight is very thin; legibility of small text at 12 px in PNG is to be judged from the rendered goldens (a heavier regular weight is a one-line change of the file used).
- c43 depends on the c43-layout font support (face-aware text widths, `Fonts` option of the renderer, parametrised edge label split); c43 is pinned to the pushed c43-layout main revision that has it (cfa9ea8).
- When the 1024 px minimum and the ~4000 px cap on the longer side conflict (a very tall, narrow diagram), the cap wins.
- Characters outside latin (cyrillic, CJK, ...) are not covered by the subset.

## Implementation (abstract)

- Family names: `Elms Sans` (400, names) and `Elms Sans Thin` (100, everything else). The percent-encoded data URI also encodes `+ / =` of the base64. Width tables (printable ASCII, per weight, 1/1000 em; fallback 0.6 em for other characters) are generated from the font files by a script and compiled in.
- Font files and license vendored and compiled into the binary; a small module hands out the font family names and the data URIs for the two faces.
- Pretty strategy: text cell styles name the family by role (regular vs name) and carry no bold; one empty 1 px carrier cell per family (ids `font-regular`, `font-name`; verified to register the font in the export) carries the `fontSource`; title/description/edge label cells get the same via the renderer's font option.
- Text width estimate in c43-layout measured per weight from the font metrics instead of the Helvetica character classes.
- render.sh computes the scale from the diagram width (margin cell) so the PNG is at least 1024 px wide, capped as before.
- Acceptance: goldens regenerated; tests check strokes (4:2:1 with the halved widths) and the icon sit line (half outline + half icon stroke), that text cells name an Elms Sans family and carry no bold style, that the font data appears once per family, that the weights used are the 100 and 400 ones, that no text overflows its node with the new width estimate; render.sh yields PNGs >= 1024 px wide for small diagrams.







## clarc

- Carried to clarc (commit `[clarc][ovp9i]`): same two vendored fonts and families (`Elms Sans` for group names, `Elms Sans Thin` for the rest), c43-layout pinned to cfa9ea8, node label lines re-wrapped and node/group widths measured with the Elms Sans faces, no bold, `render.sh` with the 1024 px minimum. Goldens regenerated; a test checks the families, no bold, and the font data once per family. Strokes and icon placement are not changed in clarc (not part of this carry).
