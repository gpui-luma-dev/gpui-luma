# Next step: shadcn / tweakcn / Radix CSS → Luma `theme.toml`

Guide for translating [shadcn/ui](https://ui.shadcn.com/) and [tweakcn](https://tweakcn.com/) theme exports into Luma’s native TOML format (`default-theme.toml` shape).

**Related:** variant weight mapping (`docs/ai/next-step-variants.md`), active theme wiring (`docs/ai/next-step-theme.md`), strategy overview (`docs/theme.md`).

**Status:** Manual conversion validated in-repo; automated importer / lexicon resolver not built yet.

---

## Reference files in this repo

| Source CSS | Example TOML | Notes |
|---|---|---|
| `crates/sdk/src/theme/tweakcdn-white-orange.css` | `crates/sdk/src/theme/theme-astrovista2.toml` | Orange primary, light card panels |
| `crates/sdk/src/theme/tweakcn-astrovista.css` | `crates/sdk/src/theme/theme-astrovista.toml` | Grey-blue canvas, coral primary, navy `--secondary` |
| — | `crates/sdk/src/theme/default-theme.toml` | Native Luma theme (SDK default via `DEFAULT_THEME_TOML`) |

CSS sources live under `crates/sdk/src/theme/` for diffing during manual import.

---

## Two formats

### shadcn / tweakcn (CSS)

Exports are usually:

```css
:root { --background: …; --primary: …; }
.dark { --background: …; … }
```

Plus optional `@theme inline { … }` (Tailwind v4 wiring — **ignore for Luma**), font stacks, `--radius`, shadow presets.

Variables are **semantic for the web stack** (`--card`, `--popover`, `--primary`, …). They describe ~40 tokens per mode, mostly base colors without hover/pressed pairs.

### Luma (`theme.toml`)

Full semantic model per mode:

- **Palette** (~57 semantic slots): app, surfaces, action roles, state, form, navigation, data accents
- **Metrics**: spacing scale, radius, border widths, control heights/padding, focus geometry
- **Typography**: families, sizes, line heights, weights per role
- **Elevation**: structured multi-layer shadows per semantic role (menu, panel, thumb, dialog, …)

Colors are normalized **`hsl(...)` / `hsla(...)`** only. Every action role includes **hover** and **pressed** variants even when CSS does not export them.

---

## Pipeline (do not skip layers)

```text
tweakcn / shadcn CSS
  → parse :root and .dark into flat token catalog
  → explicit CSS-token → Luma-path mapping (see tables below)
  → derive missing states (hover, pressed, invalid, disabled)
  → inherit metrics / typography / elevation from base Luma theme where CSS is silent
  → validate via LumaTheme::from_toml_str
  → gallery visual check (Palette + Theme Usage panes)
```

**Do not** map CSS directly into control appearance structs or template code.

**Do not** teach templates shadcn variable names.

---

## Core palette mapping

Apply independently for **`[light.*]`** and **`[dark.*]`** (always author both modes).

### App chrome

| CSS variable | Luma path | Notes |
|---|---|---|
| `--background` | `palette.app.background` | Page canvas |
| `--foreground` | `palette.app.foreground` | Primary text on canvas |
| `--muted-foreground` | `palette.app.muted_foreground` | Secondary text |

### Surfaces

| CSS variable | Luma path | Notes |
|---|---|---|
| `--card` | `palette.surface.panel.background` | Raised panels, cards |
| `--card-foreground` | `palette.surface.panel.foreground` | |
| `--popover` | `palette.surface.floating.background` | Menus, popovers, dropdown panels |
| `--popover-foreground` | `palette.surface.floating.foreground` | |
| `--muted` | `palette.surface.subtle.background` | Muted fills, subtle rows |
| `--muted-foreground` | `palette.surface.subtle.foreground` | Often also used for disabled text |

Panel/floating **border** usually from `--border`, not a separate card-border variable.

### Action roles (current three-weight model)

Until `action.subtle` lands (`docs/ai/next-step-variants.md`), use this **interim** mapping for imports:

| CSS variable | Luma path (today) | Target path (after Subtle) |
|---|---|---|
| `--primary` | `palette.action.prominent.*` | same |
| `--primary-foreground` | prominent foreground | same |
| `--secondary` | `palette.action.standard.*` | Filled alternate tone (shadcn `secondary`) |
| `--secondary-foreground` | standard foreground | |
| `--destructive` | — | deferred (no action role yet) |
| `--destructive-foreground` | — | deferred |
| `--card` + `--border` | `palette.action.subtle.*` (outline recipe) | shadcn `outline` — visually quiet |
| transparent / `--accent` hover | `palette.action.ghost.*` | same |

**Outline vs secondary:** shadcn **outline** = bordered neutral (card bg + border) → Luma **`action.subtle`**. shadcn **secondary** = filled alternate tone → Luma **`action.standard`**. Do not put navy `--secondary` on `action.subtle` — that row is labeled Subtle in the gallery and should look bordered, not filled.

### State & interaction

| CSS variable | Luma path | Notes |
|---|---|---|
| `--accent` | `palette.state.hover.background` | Light mode row/control hover |
| `--accent-foreground` | `palette.state.hover.foreground` | |
| `--ring` | `palette.focus.ring` | Focus ring color |
| `--border` | `palette.border.default` | |
| — | `palette.border.strong` | Derive or inherit from base theme |
| `--input` | `palette.form.input.border` and/or background | Often border in shadcn |
| `--destructive` | `palette.form.input.invalid_border` | Validation errors only; no destructive button role yet |

Derive when CSS is silent:

```text
hover_background   = lighten(base background, ~4–8% L) or map --accent
pressed_background = darken(base background, ~4–8% L)
```

Use consistent derivation rules per theme; document them in the TOML header comment.

### Form inputs

| CSS variable | Luma path |
|---|---|
| `--card` or `--background` | `palette.form.input.background` |
| `--foreground` | `palette.form.input.foreground` |
| `--input` | `palette.form.input.border` |
| `--muted-foreground` | `palette.form.input.placeholder` |
| `--destructive` | `palette.form.input.invalid_border` | Validation errors only; no destructive button role yet |

Prefer explicit choices over implicit “card vs background” — shadcn themes disagree.

### Navigation (sidebar)

tweakcn exports `--sidebar-*` variables. Map to Luma navigation palette:

| CSS variable | Luma path |
|---|---|
| `--sidebar` | `palette.navigation.background` |
| `--sidebar-foreground` | `palette.navigation.foreground` |
| `--sidebar-primary` | `palette.navigation.selected_background` |
| `--sidebar-primary-foreground` | `palette.navigation.selected_foreground` |
| `--sidebar-accent` | `palette.navigation.hover_background` |
| `--sidebar-accent-foreground` | hover/selected foreground as needed |
| `--sidebar-border` | `palette.navigation.border` |
| `--sidebar-ring` | `palette.navigation.focus_ring` or `palette.focus.ring` |

Dark mode: Astrovista uses `--sidebar-accent` for hover where light mode uses `--accent`.

### Data / charts

| CSS variable | Luma path |
|---|---|
| `--chart-1` … `--chart-5` | `palette.data.accent_1` … `accent_5` |
| `--primary` | often `accent_1` when charts should match brand |

---

## Astrovista example (tweakcn-astrovista.css)

Documented conversion targets:

| Token | Light | Dark |
|---|---|---|
| `--background` | Grey-blue canvas `hsl(204 12.2% 92%)` | Near-black `hsl(0 0% 10.2%)` |
| `--primary` | Coral `hsl(15.2 72.6% 54.1%)` | Same coral |
| `--card` / `--popover` | White panels/menus | `hsl(0 0% 12.5%)` |
| `--secondary` | Navy `hsl(217.3 44% 32.9%)` | Navy `hsl(216.2 44.1% 28%)` → **`action.standard`** |
| `--muted` | Subtle surfaces / disabled | Elevated chrome |
| `--accent` | Hover states | Dark: `--sidebar-accent` for hover |
| `--destructive` | `form.input.invalid_border` | Validation errors only |
| `--sidebar-*` | Navigation palette | Navigation palette |

**Subtle** (outline): white/card fill + grey `--border`. **Standard**: navy `--secondary` fill.

---

## white-orange example (tweakcdn-white-orange.css)

| Token | Role |
|---|---|
| `--primary` `hsl(24 100% 50%)` | Prominent / brand orange |
| `--secondary` | Second orange in this theme — map to **Subtle**, not Standard |
| `--card` | Slightly tinted panel bg vs pure white app bg |
| `--muted` | Subtle surface grey |
| `--ring` | Focus ring (orange family) |

Reference TOML: `theme-astrovista2.toml` (despite the name, maps white-orange CSS comments in header).

---

## Non-palette sections

CSS exports partial metrics. **Inherit** from `default-theme.toml` unless the CSS provides a clear override:

| CSS | Luma section | Rule |
|---|---|---|
| `--radius` | `metrics.radius.md` (and scale sm/lg relative) | Convert rem → px (`0.5rem` → 8px at 16px root) |
| `--spacing` | base for `metrics.spacing.*` | Often 4px grid |
| `--font-sans` | `typography.font_family.sans` | Strip `@import`; Luma loads fonts separately |
| `--font-mono` | `typography.font_family.mono` | |
| `--shadow-sm` … `--shadow-xl` | `elevation.*` | Prefer parsing layers; fallback to native elevation tokens |
| `--tracking-normal` | label typography tracking | Optional |

Keep full metrics/typography/elevation blocks in imported TOML — copy from native theme, then override only what CSS defines.

---

## Color syntax normalization

Luma loader accepts space-separated HSL/HSLA:

```text
hsl(210 40% 98%)
hsla(0 0% 0% / 0)
```

Convert from CSS forms:

| CSS input | Luma output |
|---|---|
| `hsl(204, 12.2%, 91.96%)` | `hsl(204 12.2% 91.96%)` |
| `oklch(...)` | convert to HSL (manual or tool) before TOML |
| `#rrggbb` / `rgb()` | convert to HSL |
| bad HSL channels in exports | fix during import (validate visually) |

---

## Derivation & missing states

shadcn exports rarely include hover/pressed. Standard derivation for action roles:

```text
prominent.hover_background   = lighten(prominent.background, 4–6% L)
prominent.pressed_background = darken(prominent.background, 6–10% L)
standard.hover_background    = lighten(standard.background, 4–6% L)  # --secondary fill
subtle.border                = --border  # outline recipe
ghost.hover_background       = map --accent or muted wash
ghost.border                 = transparent (hsla 0 alpha)
```

Document chosen percentages in the TOML file header for reproducibility.

---

## Configurable mapping (future importer)

Because the same CSS token means different things per theme, a future importer should use **binding rules**, not hardcoded 1:1 names:

```toml
# Conceptual binding (not implemented)
[light.palette.action.prominent]
background = [{ token = "primary" }, { inherit = "base" }]

[light.palette.form.input]
background = [{ token = "card" }, { token = "background" }, { inherit = "base" }]
```

Binding kinds discussed in planning:

| Kind | Meaning |
|---|---|
| `{ token = "primary" }` | Lookup in parsed CSS catalog for current mode |
| `{ path = "palette.app.background" }` | Another resolved Luma path |
| `{ derive = "lighten", from = "primary", amount = "8%" }` | Transform |
| `{ inherit = "base" }` | Keep native `default-theme.toml` value |
| `[ { token = "card" }, { token = "background" }, { inherit = "base" } ]` | Cascade / default-if-missing |

Lexicon should be **source-agnostic**; CSS files register as named catalogs (`astrovista`, `white-orange`, …).

---

## Validation checklist

After writing TOML:

1. `LumaTheme::from_toml_str` parses without error (both modes complete).
2. `cargo test -p gpui-luma -- theme` passes.
3. Gallery **Palette** pane shows expected semantic groups.
4. Gallery **Theme Usage** pane: no unexpected “Reserved / unused” for imported roles you care about.
5. Spot-check controls: Prominent, Standard (outline), Ghost, form fields, navigation sidebar, floating menus.
6. Toggle light/dark in gallery title bar — pairs remain coherent.

---

## Common mistakes

| Mistake | Symptom |
|---|---|
| Map `--secondary` → `action.subtle` | Subtle row shows filled navy instead of outline Cancel |
| Use `--background` for panel and app interchangeably | Flat UI; cards disappear |
| Skip hover/pressed derivation | Buttons feel dead; SDK expects full action role |
| Copy `@theme inline` into Luma | Irrelevant Tailwind wiring |
| Import only `:root` | Dark mode broken or falls back incorrectly |
| Map `--accent` only to ghost | Hover states wrong on lists/menus |
| Put `--destructive` only in orphan `[*.colors]` tables | Loader ignores it; wire to `form.input.invalid_border` or drop |

---

## SDK schema notes

Ensure Rust loader matches authored TOML:

- `palette.action.subtle.*` — add when variant work lands
- Orphan `[*.colors].destructive_*` tables should wire into `form.input.invalid_border` or be removed to avoid false confidence

Check `crates/sdk/src/theme/tokens.rs` and `registry.rs` when adding new palette roles.

---

## Explicitly future work

- CLI: `luma-theme import tweakcn-astrovista.css --bindings lexicon.toml`
- Primitive palette layer (`[light.primitives]` + `{ ref = "brand" }`) for deduplicated authoring
- Automated contrast warnings on import
- oklch → hsl conversion in tooling

For variant naming when importing, always follow `docs/ai/next-step-variants.md` — map CSS tokens to the **target** Luma ladder even before `ButtonKind::Subtle` exists in code.
