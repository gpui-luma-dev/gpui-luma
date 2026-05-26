# Next step: shadcn / tweakcn / Radix CSS → Luma `theme.toml`

Guide for translating [shadcn/ui](https://ui.shadcn.com/) and [tweakcn](https://tweakcn.com/) theme exports into Luma’s native TOML format (`default-theme.toml` shape).

**Related:**

| Doc | Role |
|---|---|
| `docs/ai/next-step-variants.md` | Four-weight ladder — **implemented** |
| `docs/ai/next-step-theme.md` | Active theme wiring — **implemented** |
| `docs/ai/theme-lexicon.md` | Lexicon grammar + import workflow — **lexicon file in repo** |
| `docs/ai/next-step-codegen.md` | sdk-theme matrices / build codegen — **proposed** (post-import) |
| `docs/theme.md` | Strategy overview |

**Status (May 2026):**

| Track | State |
|---|---|
| Manual CSS → TOML | **Done** for Astrovista and Jarvis (gallery assets) |
| SDK native default | **`crates/sdk/src/theme/default-theme.toml`** only (`LumaTheme::native()` / `DEFAULT_THEME_TOML`) |
| Active theme at runtime | **Done** — gallery registers `LumaThemePack` + `set_active_theme_pack` (see `next-step-theme.md`) |
| Four-weight action ladder | **Done** — `action.subtle` / `ButtonKind::Subtle` (see `next-step-variants.md`) |
| Choice controls + `ButtonKind` | **Done (interim Rust)** — checkbox, switch, radio, toggle resolve checked/on/selected via `Standard` vs `Prominent`; toggle unselected pairing → `Subtle` in button-family theme (see below) |
| Gallery variant matrices | **Done** — Button, Toggle, Switch, Checkbox, Radio Button panes; Introduction panels use explicit kinds where needed |
| **Lexicon TOML + human guide** | **Done** — `crates/sdk/src/theme/lexicon.toml`, `docs/ai/theme-lexicon.md` (binding rules for importer; validated against Astrovista/Jarvis manual TOML) |
| **Automated import code** | **Closed (May 2026)** — `crates/luma-theme`; further import polish deferred — see [`fundamental-templates.md`](fundamental-templates.md) (SDK/theme split) |
| sdk-theme codegen | **Proposed** — move choice appearance matrices out of Rust (see `next-step-codegen.md`) |
| white-orange TOML | **Not in repo** |
| Full typography import from CSS | **Not done** — sizes still copied from native Luma scale |
| In-app theme picker | **Not done** — gallery uses CLI only |

### What “done” means for import

Foundation work is in place so the **importer can be implemented against stable targets**:

1. **Target schema** — full Luma `theme.toml` shape with four action roles, navigation, form, state pairs.
2. **Reference outputs** — hand-authored `tweakcn-astrovista.toml` and `tweakcn-jarvis.toml` as golden files the importer should reproduce (modulo documented derivation rules).
3. **Binding spec** — `lexicon.toml` cascade grammar (`token`, `palette`, `path`, `inherit`, transforms) documented in `theme-lexicon.md`.
4. **Runtime consumer** — gallery loads imported TOML via CLI; controls resolve colors from active pack (no per-control palette hacks in templates).
5. **Variant semantics** — importers map CSS to **`action.prominent` / `action.standard` / `action.subtle` / `action.ghost`**, not to control Rust code.

**Still manual / interim (not replaced by import yet):**

- Hover/pressed derivation percentages in TOML headers.
- Choice **policy** (which `ButtonKind` per control) — today `.kind(...)` in app/gallery code + Rust `match` in `*Theme::resolve`; future: sdk-theme + codegen (`next-step-codegen.md`).
- Typography sizes and gallery demo label `text_size(px(...))` hardcoding.

**Next engineering step (superseded):** import pipeline is implemented in `crates/luma-theme` — track closed for polish; **next:** SDK/theme split per [`fundamental-templates.md`](fundamental-templates.md).

---

## Repository layout (current)

Imported tweakcn themes live in the **gallery app**, not the SDK crate. The SDK ships one native theme for library defaults and tests.

| Role | Path |
|---|---|
| SDK native theme | `crates/sdk/src/theme/default-theme.toml` |
| Import lexicon (bindings) | `crates/sdk/src/theme/lexicon.toml` |
| Lexicon human guide | `docs/ai/theme-lexicon.md` |
| Imported Astrovista | `apps/gallery/src/assets/themes/tweakcn-astrovista.toml` |
| Imported Jarvis | `apps/gallery/src/assets/themes/tweakcn-jarvis.toml` |
| Gallery wiring | `apps/gallery/src/gallery/theme.rs` (`include_str!` + `GalleryThemeChoice`) |
| Jarvis sans font | `apps/gallery/src/assets/fonts/Rajdhani/Rajdhani-Variable.ttf` + `OFL.txt` |
| Font registration | `apps/gallery/src/assets/fonts.rs` — `load_rajdhani()` |

**Source CSS** for golden imports lives in **`apps/gallery/tweakcn/`**:

| CSS input | Golden TOML |
|---|---|
| `apps/gallery/tweakcn/astrovista.css` | `apps/gallery/src/assets/themes/tweakcn-astrovista.toml` |
| `apps/gallery/tweakcn/jarvis.css` | `apps/gallery/src/assets/themes/tweakcn-jarvis.toml` |

Other files in that folder (e.g. `enhance-material.css`, monochrome exports) are additional samples; the historical `apps/gallery/themes/` path is no longer used for Astrovista/Jarvis.

**Removed / renamed** (historical):

- `theme-astrovista.toml` / `jarvis-theme.toml` → **`tweakcn-astrovista.toml`** / **`tweakcn-jarvis.toml`**
- `theme-astrovista2.toml` (white-orange) — never added; still a future import

---

## Gallery preview

```bash
cargo run -p gpui-luma-gallery                    # SDK default-theme.toml
cargo run -p gpui-luma-gallery -- astrovista
cargo run -p gpui-luma-gallery -- jarvis          # loads Rajdhani before open
cargo run -p gpui-luma-gallery -- retro-arcade    # any stem in src/assets/themes/
```

- First positional arg: `default` or a theme file stem under `apps/gallery/src/assets/themes/` (e.g. `retro-arcade` → `retro-arcade.toml`, case-insensitive). Missing arg → **`default`**.
- Startup: `gpui_luma::init` → optional `fonts::load_rajdhani` (Jarvis only) → `set_active_theme_pack` → window.
- Parse tests: `apps/gallery/src/gallery/theme.rs` (`astrovista_theme_parses_*`, `jarvis_theme_sans_family_matches_embedded_font`).

Validate visually: **Palette** + **Theme Usage** panes, button matrix (four weights), **Toggle / Switch / Checkbox / Radio Button** template matrices (Standard vs Prominent rows), Introduction panels, light/dark title bar toggle.

**Interim product convention (until sdk-theme codegen):** use **`ButtonKind::Standard`** for stock forms and groups; set **`ButtonKind::Prominent`** only where the design needs primary accent (gallery matrices, explicit intro cases). Astrovista exercises the full color ladder; Jarvis leans on borders + Rajdhani with a quieter palette contrast.

---

## Reference files

| Source (tweakcn export) | TOML in repo | Notes |
|---|---|---|
| `apps/gallery/tweakcn/astrovista.css` | `apps/gallery/src/assets/themes/tweakcn-astrovista.toml` | Coral primary, navy `--secondary` → **Standard**, outline → **Subtle** |
| `apps/gallery/tweakcn/jarvis.css` | `apps/gallery/src/assets/themes/tweakcn-jarvis.toml` | Teal primary; Rajdhani sans (embedded) |
| tweakcn white-orange (planned) | — | Not imported; was planned as `theme-astrovista2.toml` |
| — | `crates/sdk/src/theme/default-theme.toml` | Native Luma; SDK + gallery `default` CLI |

---

## Fonts (GPUI / gallery)

Luma TOML only stores **family name strings**. GPUI does not load web fonts from CSS `@import`; apps must **`TextSystem::add_fonts`** with TTF/OTF bytes.

| Theme | TOML `typography.font.sans.family` | Gallery behavior |
|---|---|---|
| **Jarvis** | `Rajdhani Variable` | `load_rajdhani()` in `main.rs` when CLI is `jarvis`; variable TTF in `assets/fonts/Rajdhani/` |
| **Astrovista** | `Outfit` (from CSS intent) | **Not embedded** — system/fallback sans until Outfit is bundled |
| **Default** | Native stack in `default-theme.toml` | No extra gallery fonts |
| **Mono** (Jarvis TOML) | `JetBrains Mono` | System font only — not embedded |

**Lesson (Jarvis):** Fontshare’s variable file registers as **`Rajdhani Variable`**, not `Rajdhani`. TOML and `RAJDHANI_FAMILY` in `fonts.rs` must match the font’s **name table**, or GPUI falls back to system UI. See `apps/gallery/src/assets/fonts/Rajdhani/README.md`.

Trimmed Fontshare tree: keep **variable TTF + OFL** only; drop woff/eot duplicates.

---

## Typography (current gap vs tweakcn)

Imported TOMLs **copy metrics and typography sizes from `default-theme.toml`**, not from tweakcn `rem` / Tailwind scale. CSS `--font-sans` / `--font-mono` map to **family only**.

Typical Luma scale today:

| Role | Size (px) | Used by |
|---|---|---|
| `text.label` | 13 | Buttons, nav items, most control chrome |
| `text.body` | 14 | Text fields, text areas |
| `text.caption` | 11 | Secondary labels |
| `text.title` | 20 | Headings |

tweakcn/shadcn often renders controls at **`text-sm` (14px)** and page copy at **16px root**. The gallery can feel **~1–2px smaller** than the tweakcn preview even when colors match. Follow-up: bump `typography.text.*` in imported TOML and/or replace gallery **hardcoded** `text_size(px(11.0))` demo labels with theme roles.

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
  → parse :root and .dark into flat token catalog          ← import code (next)
  → apply lexicon.toml bindings (crates/sdk/src/theme/lexicon.toml)  ← spec done
  → derive missing states (hover, pressed, invalid, disabled)
  → inherit metrics / typography / elevation from base Luma theme where CSS is silent
  → validate via LumaTheme::from_toml_str
  → gallery visual check (Palette + Theme Usage panes; CLI theme arg)
  → (optional) diff against golden tweakcn-*.toml in gallery assets
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

### Action roles (four-weight model — implemented)

Map imports using `docs/ai/next-step-variants.md`:

| CSS / shadcn recipe | Luma path | `ButtonKind` |
|---|---|---|
| `--primary` | `palette.action.prominent.*` | Prominent |
| `--secondary` (filled) | `palette.action.standard.*` | Standard |
| `--card` + `--border` (outline) | `palette.action.subtle.*` | Subtle |
| transparent / `--accent` hover | `palette.action.ghost.*` | Ghost |

**Outline vs secondary:** shadcn **outline** → **`action.subtle`**. shadcn **secondary** (filled navy/orange) → **`action.standard`**. Do not map navy `--secondary` to `action.subtle` — the gallery Subtle row will look filled instead of bordered.

`--destructive` → `palette.form.input.invalid_border` only (no destructive button role yet).

### State & interaction

| CSS variable | Luma path | Notes |
|---|---|---|
| `--accent` | `palette.state.hover.background` | Light mode row/control hover |
| `--accent-foreground` | `palette.state.hover.foreground` | |
| `--ring` | `palette.focus.ring` | Focus ring color |
| `--border` | `palette.border.default` | |
| — | `palette.border.strong` | Derive or inherit from base theme |
| `--input` | `palette.form.input.border` and/or background | Often border in shadcn |

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
| `--destructive` | `palette.form.input.invalid_border` |

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

Documented conversion targets (validated in `tweakcn-astrovista.toml`):

| Token | Light | Dark |
|---|---|---|
| `--background` | Grey-blue canvas `hsl(204 12.2% 92%)` | Near-black `hsl(0 0% 10.2%)` |
| `--primary` | Coral `hsl(15.2 72.6% 54.1%)` | Same coral |
| `--card` / `--popover` | White panels/menus | `hsl(0 0% 12.5%)` |
| `--secondary` | Navy fill → **`action.standard`** | Navy → **`action.standard`** |
| `--muted` | Subtle surfaces / disabled | Elevated chrome |
| `--accent` | Hover states | Dark: `--sidebar-accent` for hover |
| `--destructive` | `form.input.invalid_border` | Validation errors only |
| `--sidebar-*` | Navigation palette | Navigation palette |

**Subtle** (outline): card fill + `--border`. **Standard**: navy `--secondary` fill. **Prominent**: coral `--primary`.

---

## white-orange example (tweakcdn-white-orange.css)

**Not imported yet.**

| Token | Role |
|---|---|
| `--primary` `hsl(24 100% 50%)` | Prominent / brand orange |
| `--secondary` | Second orange in this theme — map to **Standard** or **Subtle** per visual (see variants doc) |
| `--card` | Slightly tinted panel bg vs pure white app bg |
| `--muted` | Subtle surface grey |
| `--ring` | Focus ring (orange family) |

Planned TOML name was `theme-astrovista2.toml`; use `tweakcn-white-orange.toml` under gallery assets when added.

---

## Non-palette sections

CSS exports partial metrics. **Inherit** from `default-theme.toml` unless the CSS provides a clear override:

| CSS | Luma section | Rule |
|---|---|---|
| `--radius` | `metrics.radius.md` (and scale sm/lg relative) | Convert rem → px (`0.5rem` → 8px at 16px root) |
| `--spacing` | base for `metrics.spacing.*` | Often 4px grid |
| `--font-sans` | `typography.font.sans.family` | Strip `@import`; register TTF in gallery/SDK separately |
| `--font-mono` | `typography.font.mono.family` | Same |
| `--shadow-sm` … `--shadow-xl` | `elevation.*` | Prefer parsing layers; fallback to native elevation tokens |
| `--tracking-normal` | label typography tracking | Optional — no TOML slot yet |

Keep full metrics/typography/elevation blocks in imported TOML — copy from native theme, then override only what CSS defines (today: mostly **colors + radius + font families**).

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

## Configurable mapping (lexicon — spec done, resolver next)

The importer should use **binding rules**, not hardcoded 1:1 CSS names. The lexicon file and grammar are **authored**; the Rust resolver that applies them to a parsed CSS catalog is **not written yet**.

| Artifact | Path | Status |
|---|---|---|
| Binding lexicon | `crates/sdk/src/theme/lexicon.toml` | **In repo** — light/dark paths, `[palette]` semantic layer, `[[conflicts]]` notes |
| Human guide | `docs/ai/theme-lexicon.md` | **In repo** — cascade forms, import workflow, Astrovista/Jarvis validation notes |
| Catalog parser | — | **Not built** |
| Lexicon resolver + TOML emitter | — | **Not built** |
| CLI | — | **Not built** (planned interface below) |

Example binding shape (implemented in lexicon, not yet executed by tooling):

```toml
[light.palette.action.prominent]
background = [{ palette = "brand" }, { token = "primary" }, { inherit = "base" }]

[light.palette.form.input]
background = [{ token = "card" }, { token = "background" }, { inherit = "base" }]
```

Binding kinds (see `theme-lexicon.md`):

| Kind | Meaning |
|---|---|
| `{ token = "primary" }` | Lookup in parsed CSS catalog for current mode |
| `{ palette = "brand" }` | Semantic alias from `[palette]` in lexicon |
| `{ path = "palette.app.background" }` | Another resolved Luma path |
| `{ derive = "lighten", from = "primary", amount = "8%" }` | Transform |
| `{ inherit = "base" }` | Keep native `default-theme.toml` value |
| `[ { token = "card" }, { token = "background" }, { inherit = "base" } ]` | Cascade / first match wins |

Lexicon is **source-agnostic**; CSS files register as named catalogs (`astrovista`, `jarvis`, `white-orange`, …) when the importer lands.

Planned CLI (first milestone):

```bash
# Single file
luma-theme import apps/gallery/tweakcn/astrovista.css \
  --out apps/gallery/src/assets/themes/tweakcn-astrovista.toml

# Batch: all *.css in source-dir → *.toml in dest-dir (same basename)
luma-theme import --source-dir apps/gallery/tweakcn \
  --dest-dir apps/gallery/src/assets/themes
```

Success criteria for v1: output parses with `LumaTheme::from_toml_str` and **matches** hand-maintained Astrovista/Jarvis TOML within documented derivation tolerance (or produces a reviewable diff).

---

## Validation checklist

After writing or editing TOML:

1. `LumaTheme::from_toml_str` parses without error (both modes complete).
2. `cargo test -p gpui-luma -- theme` and gallery theme tests in `apps/gallery/src/gallery/theme.rs` pass.
3. Gallery **Palette** pane shows expected semantic groups (`-- astrovista` / `-- jarvis`).
4. Gallery **Theme Usage** pane: no unexpected “Reserved / unused” for imported roles you care about.
5. Spot-check controls: **Prominent, Subtle, Standard, Ghost**, form fields, navigation sidebar, floating menus; **choice controls** (toggle toolbar Standard selected + Subtle unselected, switch/checkbox/radio Standard vs Prominent matrix rows).
6. Toggle light/dark in gallery title bar — pairs remain coherent.
7. **Fonts:** Jarvis — sans renders as Rajdhani (not system UI); Astrovista — accept fallback until Outfit is embedded.
8. After automated import exists: **diff** tool output against `tweakcn-astrovista.toml` / `tweakcn-jarvis.toml` golden files.

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
| TOML `family = "Rajdhani"` with variable TTF only | System fallback; use **`Rajdhani Variable`** or embed static cuts |
| Expect CSS `@import` to load fonts | GPUI needs `add_fonts` + matching family name |

---

## SDK schema notes

Loader and themes in repo already include:

- `palette.action.subtle.*` — required for imported tweakcn themes
- `LumaThemePack::from_toml_str` for gallery-embedded TOML

Orphan `[*.colors].destructive_*` tables should wire into `form.input.invalid_border` or be removed to avoid false confidence.

Check `crates/sdk/src/theme/tokens.rs` when adding new palette roles.

---

## Explicitly future work

### Import tooling (delivered — maintenance mode)

- **CSS catalog parser** — `crates/luma-theme`
- **Lexicon resolver + TOML emit + CLI** — `luma-theme import` / `catalog`; batch `--source-dir` / `--dest-dir`
- **Gallery** — `apps/gallery/tweakcn/*.css` → `apps/gallery/src/assets/themes/*.toml`; CLI `-- <stem>`

**Deferred** (Phase B in [`fundamental-templates.md`](fundamental-templates.md)): full golden diff, lexicon hover tuning, `shadow_parse`, studio app. Do not block SDK control work on these.

### After import (separate tracks)

- **sdk-theme codegen** — choice appearance matrices + defaults in data, not Rust `match` (`next-step-codegen.md`)
- **white-orange** → `apps/gallery/src/assets/themes/tweakcn-white-orange.toml`
- Re-check in **source CSS** under `crates/sdk/src/theme/` for diffing (optional hygiene)
- Typography: import tweakcn `rem` scale into `typography.text.*`; reduce gallery hardcoded demo sizes
- Embed **Outfit** (Astrovista) and optional **JetBrains Mono** (Jarvis)
- In-app theme picker (beyond CLI `default` / `astrovista` / `jarvis`)
- Primitive palette layer (`[light.primitives]` + `{ ref = "brand" }`) for deduplicated authoring
- Automated contrast warnings on import
- oklch → hsl conversion in tooling

For variant naming when importing, follow `docs/ai/next-step-variants.md` — map CSS tokens to the **four-weight** Luma ladder (`prominent` / `standard` / `subtle` / `ghost`).
