# Radix Studio Colors Tab

Create the `Colors` tab from the Radix Colors system. The tab is a visual and inspectable catalog of the complete Radix color data: neutral gray families first, chromatic families second, and alpha ramps for shadows, highlights, and overlays at the bottom.

The supplied screenshots are visual references for the table layout:

- The main table has twelve numbered columns and grouped semantic headings.
- The top rows are the gray/neutral families.
- The lower rows are chromatic color families.
- The final table shows Black and White alpha ramps over a checkerboard background.

## Source of truth

Use the upstream [`radix-ui/colors` `src` directory](https://github.com/radix-ui/colors/tree/main/src) as the source of the color data. Its [`index.ts`](https://github.com/radix-ui/colors/blob/main/src/index.ts) exports the light, dark, black-alpha, and white-alpha catalogs.

The source files are structured as named objects with twelve ordered steps:

```ts
export const gray = {
  gray1: "#fcfcfc",
  gray2: "#f9f9f9",
  // ...
  gray12: "#202020",
};
```

The light and dark files provide the standard sRGB hex values, alpha variants (`grayA1` through `grayA12`, for example), and P3 variants. The [`light.ts`](https://raw.githubusercontent.com/radix-ui/colors/main/src/light.ts) and [`dark.ts`](https://raw.githubusercontent.com/radix-ui/colors/main/src/dark.ts) files should be treated as versioned input data, not copied from the screenshots. The [`blackA.ts`](https://raw.githubusercontent.com/radix-ui/colors/main/src/blackA.ts) and [`whiteA.ts`](https://raw.githubusercontent.com/radix-ui/colors/main/src/whiteA.ts) files provide the twelve-step black and white alpha ramps.

## How to obtain and preserve the data

Prefer a deterministic, offline asset-generation workflow:

1. Pin a Radix Colors commit or release in the generator documentation.
2. Fetch or vendor `src/light.ts`, `src/dark.ts`, `src/blackA.ts`, and `src/whiteA.ts` once during an intentional data refresh.
3. Parse the exported object names and their ordered `1`–`12` properties.
4. Generate Rust source or a checked-in data asset for `look-radix`; the running app must not fetch GitHub or npm at startup.
5. Preserve the original family name, step number, source mode, value, and value kind (`Solid`, `Alpha`, or `P3`) for inspection and provenance.
6. Add a source/version marker to the generated catalog so future updates can be compared without guessing which Radix data was used.

For the first GPUI implementation, use the sRGB hex values and the `rgba(...)` alpha values because they map directly to the existing `Hsla`/RGBA rendering path. Keep the P3 values in the catalog or generation input for a later wide-gamut renderer path; do not silently substitute P3 values into the sRGB display.

The black and white source ramps are explicitly alpha data. For example, `blackA1` is black at low opacity and `blackA12` is black at high opacity; it is not a sequence of opaque gray colors. Preserve that distinction in the data model and renderer.

## Color family order

Render rows in this order so the gray portion is visibly the top part of the table:

### Neutral gray families

1. `Gray`
2. `Mauve`
3. `Slate`
4. `Sage`
5. `Olive`
6. `Sand`

### Chromatic families

1. `Tomato`
2. `Red`
3. `Ruby`
4. `Crimson`
5. `Pink`
6. `Plum`
7. `Purple`
8. `Violet`
9. `Iris`
10. `Indigo`
11. `Blue`
12. `Cyan`
13. `Teal`
14. `Jade`
15. `Green`
16. `Grass`
17. `Lime`
18. `Mint`
19. `Sky`
20. `Yellow`
21. `Amber`
22. `Orange`

Each family has twelve steps. In light mode, use the light catalog; in dark mode, use the dark catalog. Do not generate dark colors by mechanically inverting light colors.

## Data model direction

Expand the current Radix look model rather than keeping a second, app-only copy of the catalog. The existing `look-radix` implementation has a small `ScaleFamily`/`ColorScale` abstraction with `Gray`, `Color`, and `Destructive` families. The full Colors tab needs a catalog-level abstraction that can represent every named family while retaining the existing semantic resolver for controls.

A suitable shape is:

```text
RadixColorSystem
├── light: RadixColorMode
├── dark: RadixColorMode
├── black_alpha: AlphaScale
└── white_alpha: AlphaScale

RadixColorMode
└── families: ordered collection of RadixColorScale

RadixColorScale
├── family: RadixColorFamily
├── steps: [RadixColorValue; 12]
└── alpha_steps: optional [RadixColorValue; 12]
```

Keep the existing semantic roles (`Background`, `Surface`, `Border`, `Foreground`, `Primary`, `Destructive`, and so on) mapped to the catalog. The Colors tab should read the same resolved data used by controls, so the catalog and the live look cannot drift apart.

The current custom palette seed behavior can remain separate from the static catalog until the full Radix family-generation policy is defined. If seed editing later regenerates the catalog, make that an explicit model operation and retain the source/generated provenance.

## Table presentation

Use a scrollable, grid-based presentation with one label column and twelve equal step columns.

### Header

- First row: semantic group headings spanning the relevant columns:
  - `Backgrounds`: steps 1–2
  - `Interactive components`: steps 3–5
  - `Borders and separators`: steps 6–8
  - `Solid colors`: steps 9–10
  - `Accessible text`: steps 11–12
- Second row: numeric step labels `1` through `12`.
- Keep the group headings and step labels aligned with the swatch columns.
- Use a thin muted rule below each group heading, as shown in the reference.

### Rows and cells

- Render a family name in the leading label column.
- Render twelve rectangular swatches for each family, one per step.
- Use a small consistent gutter between cells; do not allow rounded card shells to interrupt the continuous color matrix.
- Set text/foreground contrast based on the actual swatch luminance. For very light cells, use a dark label; for dark cells, use a light label.
- Show the step value on hover/focus or in the existing swatch detail overlay. Include family, mode, step, value, and value kind.
- Preserve keyboard focus and click selection for inspectable cells.
- Keep the neutral six-row block at the top, with the chromatic block immediately below it.

### Responsive behavior

The twelve columns need a minimum usable width. On narrow windows, keep the label column fixed and place the matrix in a horizontal scroll surface rather than compressing cells until the colors are indistinguishable. The group header, numeric header, and data rows must share the same horizontal scroll offset.

## Shadows, highlights, and overlays

Append a second twelve-column matrix below the chromatic families, titled **Shadows, highlights, and overlays**.

- Use the same numbered columns `1` through `12`.
- Render two rows: `Black` from `blackA1`–`blackA12` and `White` from `whiteA1`–`whiteA12`.
- Paint a light/dark checkerboard behind every cell so alpha is visible.
- Composite the black or white alpha value over the checkerboard; do not render the alpha color as an opaque fill.
- Keep the row labels and columns aligned with the primary color matrix.
- Show the original alpha value in the detail overlay, including its opacity.
- Treat these as reusable overlay/elevation inputs in the look model, not as ordinary opaque color families.

The source alpha ramps are twelve values from low to high opacity. The canonical black values are documented in [`blackA.ts`](https://raw.githubusercontent.com/radix-ui/colors/main/src/blackA.ts), and the canonical white values in [`whiteA.ts`](https://raw.githubusercontent.com/radix-ui/colors/main/src/whiteA.ts).

## App and module boundaries

The `Colors` tab should own only composition and inspection layout. Keep color data, parsing/generated assets, and semantic resolution in `crates/look-radix`.

Suggested split:

```text
crates/look-radix/src/
├── colors.rs          # catalog types, family order, source/version metadata
├── colors_data.rs     # generated light/dark/alpha values
└── scale.rs           # compatibility and seed-derived scale behavior

apps/radix-studio/src/
├── colors.rs          # Colors tab composition and mode selection
└── components/
    └── color_matrix.rs # reusable grouped matrix and alpha matrix views
```

Use SDK layout and interaction primitives for the scroll surface, grid, focus, and any detail overlay. The app may compose swatch cells, labels, and headings, but should not create a replacement interactive control system with raw event handling.

## Acceptance criteria

- [ ] The `Colors` tab displays all twelve steps for every neutral and chromatic family.
- [ ] The six neutral gray families (`Gray` through `Sand`) appear above the chromatic families.
- [ ] Light/dark mode switches the catalog values from the corresponding Radix source data.
- [ ] Group headings and step numbers align with their twelve swatch columns.
- [ ] Cells can be focused/selected and expose family, mode, step, and value details.
- [ ] The bottom matrix displays `Black` and `White` alpha ramps over a checkerboard.
- [ ] Alpha values remain transparent overlays rather than opaque colors.
- [ ] The displayed catalog is sourced from the same `look-radix` data used by themed controls.
- [ ] The matrix remains usable through horizontal scrolling at narrow widths.
- [ ] Source/version provenance is retained for future Radix Colors updates.

## Verification

- Compare light and dark values against the pinned upstream `light.ts` and `dark.ts` sources.
- Verify every family has exactly twelve ordered steps and that no family is omitted or duplicated.
- Verify black/white alpha compositing against a checkerboard at steps 1, 6, and 12.
- Test keyboard focus, selection, detail inspection, and horizontal scrolling.
- Confirm semantic control colors continue to resolve through the expanded catalog.
- Run `cargo fmt`, relevant `cargo test` targets, and `cargo clippy` when implementation begins.

