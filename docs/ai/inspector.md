# Theme Inspector (design)

Design for a **live theme inspector** in Theme Studio: select a demo panel (or a control within it) and see a tree of controls with **resolved colors** and **CSS provenance** — where each value originated in the loaded theme file.

**Status:** design only — not implemented.

**Related:**

- [`look-theme.md`](look-theme.md) — how `ShadcnLook` resolves CSS catalogs at runtime
- [`look-cleanup.md`](look-cleanup.md) — SDK vs look-shadcn boundary (templates paint `*Appearance`; looks resolve tokens)
- [`introspection.md`](introspection.md) — compile-time `look_table!` matrices + static metadata (offline half)
- Gallery **Theme Parts** sidebar — static metadata via `all_shadcn_theme_usages()` in `crates/look-shadcn/src/usage.rs` (different layer; see [vs static metadata](#vs-static-metadata-usage-rs))

---

## Problem

When tuning a look or reverse-engineering shadcn behavior, you need to answer:

1. **What color is this control painting right now?** (with Theme Studio sidebar overrides applied)
2. **Which CSS custom property produced it?** (`--input`, `--primary-hover`, `--input/30`, etc.)
3. **Where is that in the source file?** (`:root` vs `.dark`, raw `hsl(...)` / `oklch(...)`, optional line number)

GPUI is not a browser. There is no DOM, no computed-style API, and no runtime reflection over Rust structs. A registry of “logical shadcn selectors” or a hand-maintained state table cannot answer instance-level questions (“row 3 checkbox, hovered, with my `--primary` override”).

The inspector must be **opt-in instrumentation**: controls and demo panels expose a probe tree; look resolvers return **provenance** alongside color values.

---

## Goals

| Goal | Notes |
|------|-------|
| Control tree for a demo panel | Composition reflects how the panel *holds* entities, not arbitrary GPUI element walks |
| Resolved `Hsla` per appearance field | Same values templates paint at probe time |
| CSS origin per field | Token name, selector block, raw declaration; not Rust type paths |
| Live overrides | Theme Studio sidebar edits appear as `Override`, not as fake CSS lines |
| Honest derived colors | Opacity suffixes (`--accent/40`), algorithmic hovers, template opacity — labeled, not collapsed into a bare `--token` |
| **`--token/xx` notation** | Tailwind/shadcn-style token + opacity (e.g. `--input/30`) even when alpha is applied in Rust, not written in the CSS file |

## Non-goals

- Pixel-pick / hit-test without registered control ids
- Automatic discovery of embedded controls inside opaque closures (panel must wire the tree)
- Literal browser CSS selectors or DevTools parity
- Replacing gallery Theme Parts static docs (complementary)

---

## Architecture overview

```text
┌─────────────────────────────────────────────────────────────┐
│  Theme Studio UI                                            │
│  select InspectableId → inspector popup / sidebar           │
└───────────────────────────┬─────────────────────────────────┘
                            │ inspect_tree(window, cx)
┌───────────────────────────▼─────────────────────────────────┐
│  Demo panel (e.g. PaymentsPanel)                            │
│  composes InspectNode tree from known Entity fields         │
└───────────────────────────┬─────────────────────────────────┘
                            │ ThemeInspectable::inspect per node
┌───────────────────────────▼─────────────────────────────────┐
│  SDK controls (Button, ListViewControl, …)                  │
│  render_model + resolved appearance with provenance           │
└───────────────────────────┬─────────────────────────────────┘
                            │ provenance-aware resolve
┌───────────────────────────▼─────────────────────────────────┐
│  look-shadcn resolvers                                      │
│  checkbox_appearance, list_view_row_palette, button_appearance│
│  return ResolvedColor { value, source }                     │
└───────────────────────────┬─────────────────────────────────┘
                            │ catalog lookup + spans
┌───────────────────────────▼─────────────────────────────────┐
│  CssTokenCatalog (+ optional line/selector metadata)        │
│  loaded from native.css or Theme Studio CSS path              │
└─────────────────────────────────────────────────────────────┘
```

Two faces of the same truth (optional later):

- **Live inspector** (Theme Studio) — one panel, current interaction state, current overrides
- **CLI state-table generator** (offline) — all variants × states for diffing looks; shares resolver + provenance types

---

## Core types (proposed)

### `InspectNode`

Tree node for the UI. Type names stay on the node; field rows use short names only.

```rust
struct InspectNode {
    id: SharedString,           // e.g. "studio-payments-row-2"
    label: SharedString,          // e.g. "Checkbox"
    kind: SharedString,           // e.g. "checkbox", "list_view", "button"
    state: InspectState,          // checked, hovered, selected, disabled, …
    fields: Vec<InspectField>,
    children: Vec<InspectNode>,
}

struct InspectField {
    name: SharedString,           // e.g. "indicator_background" — not "CheckboxPalette.indicator_background"
    color: ResolvedColor,
}
```

### `ResolvedColor`

```rust
struct ResolvedColor {
    value: Hsla,
    source: ColorSource,
}

enum ColorSource {
    /// Direct hit: `--input` in `:root` or `.dark`
    CssVar {
        token: String,
        selector: CssSelector,    // Root | Dark
        raw: String,              // "hsl(220 13% 91%)"
        line: Option<u32>,
    },
    /// Explicit state key in CSS, e.g. `--primary-hover`
    CssStateVar {
        token: String,
        layer: InteractionLayer,
        raw: String,
        line: Option<u32>,
    },
    /// Alpha baked into the CSS declaration, e.g. `hsl(...) / 30%`
    CssVarSlashAlpha {
        token: String,
        selector: CssSelector,
        raw: String,              // full declaration including `/ 30%`
        alpha_percent: u8,        // display as --token/30
        line: Option<u32>,
    },
    /// Alpha applied in look-shadcn on a catalog token (not in source CSS)
    TokenAlpha {
        token: String,            // display as --input/30
        alpha_percent: u8,
        base_raw: String,         // full-opacity `--input` from catalog
        applied_at: String,       // e.g. "outline_role · dark · default"
    },
    /// Multi-step recipe (when token/xx alone is not enough)
    Derived {
        label: String,
        inputs: Vec<ColorSource>,
    },
    /// Catalog miss path: `--primary` + hover/press delta
    Algorithmic {
        base_token: String,
        rule: String,             // e.g. "light filled hover −0.03 L"
    },
    /// Template-only (not from look resolver)
    Template {
        note: String,             // e.g. "disabled opacity 0.56 on composed fill"
    },
    /// Theme Studio sidebar override (not in source CSS file)
    Override {
        token: String,
    },
}
```

Field listing can be generated with a derive macro (`InspectAppearance`) on `*Appearance` / `*Palette` structs — compile-time “reflection” over named fields, not runtime struct introspection.

### `ThemeInspectable`

SDK trait implemented per control type:

```rust
trait ThemeInspectable {
    fn inspect(&self, window: &Window, cx: &mut App) -> InspectNode;
}
```

Implementations call the **same resolve path as templates** (e.g. mirror `TextFieldControl::resolved_appearance`). Requires public probe helpers where today only private `render_model` / `resolve_appearance` exist (`ListViewControl`, `Button`, etc.).

Demo panels implement **`inspect_tree`** (or `ThemeInspectable for PaymentsPanel`) by composing child nodes from explicit `Entity` fields — the panel already owns the graph.

---

## Output format (example)

Selecting **Payments** (`apps/theme-studio/src/studio/panels/payments.rs`) should look like:

```text
Payments
├─ list_view [studio-payments]
│    border           hsl(220 13% 91%)     :root  --input
│    background       hsl(0 0% 100%)       :root  --background
│    ├─ row 0 [selected=false, hovered=false]
│    │    background     transparent
│    │    label_color    hsl(222 47% 11%)   :root  --foreground
│    │    └─ checkbox [studio-payments-row-0, checked=false]
│    │         indicator_background   hsl(0 0% 100%)   :root  --background  (outline default)
│    │         indicator_border       hsl(214 32% 91%)  :root  --border
│    ├─ row 1 [selected=true, …]
│    │    background     hsl(210 40% 96%)   :root  --muted
│    │    …
├─ button [payments-prev, secondary, disabled]
│    background       …                    :root  --secondary-disabled  (or algorithmic)
│    foreground       …                    :root  --secondary-foreground
└─ button [payments-next, secondary, enabled]
     …
```

Expand a row to show the literal declaration when spans are available:

```text
  --input: hsl(220 13.0435% 90.9804%);    native.css:23
```

**Do not** show Rust paths like `ListViewAppearance.background` in the primary column. The node kind (`list_view`) carries type context; repeating it on every line is noise inherited from gallery Theme Parts (`usage.rs`), not inspector output.

Optional collapsed provenance (secondary column / expander):

```text
indicator_background   hsl(217 91% 60%)
  └─ :root --primary · checked · default
```

---

## Payments panel: composition model

`PaymentsPanel` is a good first target because every interesting control is an **explicit field**:

| Node | Entity / source | Look path |
|------|-----------------|-----------|
| List shell | `self.list_view` | `list_view_appearance` → `--background`, `--input`, … |
| Row chrome | per-index row state on list | `list_view_row_palette` → transparent / `--accent/40` / `--muted` |
| Row checkboxes | `self.row_checkboxes[i]` | `checkbox_appearance(Primary, …)` → `--primary`, `--border`, … |
| Prev / Next | `self.prev_button`, `self.next_button` | `button_appearance(Secondary, …)` |
| Status / Email / Amount text | not separate controls | row `label_color` |
| Ellipsis column | raw `div` + Lucide glyph | not themed — omit or manual chrome note |
| Card chrome | `look.chrome()` in `render()` | palette / `--border`, `--background` |

**Important:** checkboxes are embedded in list columns via a closure but **owned** by the panel (`row_checkboxes: Arc<Vec<Checkbox>>`). `ListViewControl` cannot auto-discover them; `PaymentsPanel::inspect_tree` must zip rows with `row_checkboxes[i]`.

---

## Provenance-aware resolve (look-shadcn)

Today resolvers return bare `Hsla` and discard token names. Example — list border:

```rust
// crates/look-shadcn/src/controls/list_view.rs
border: resolve_color(catalog, "input")?,
```

`resolve_color` (`crates/look-shadcn/src/catalog/mod.rs`) parses the value but returns only `Hsla`. The inspector requires threading **`ResolvedColor`** through:

- `resolve.rs` — `resolve_color`, `resolve_color_layer`, `resolve_action_layer`, `resolve_accent_whisper`, …
- per-control modules — `checkbox.rs`, `list_view.rs`, `button.rs`, …
- public probe helpers on `ShadcnLook` where panels probe without reaching into templates

### CSS parser spans

`parse_css_catalog` (`crates/look-shadcn/src/catalog/parse.rs`) currently stores `BTreeMap<String, String>` (token → raw value). Extend to optionally record **`(selector, line)`** per token when parsing the Theme Studio–loaded file or embedded `assets/native.css`.

State-specific keys already follow a convention (`primary-hover`, etc.) via `state_key` in `state_color.rs`.

### Source kinds the UI must distinguish

| Kind | Example | Display |
|------|---------|---------|
| Direct CSS var | list `border` → `--input` | `:root --input` + raw value |
| State var or algorithm | button hover | `--secondary-hover` if present, else `--secondary` + algorithm note |
| Token alpha (resolver) | outline dark default fill | `--input/30` (applied in `outline_role`, not in CSS) |
| CSS slash alpha | declaration `hsl(...) / 30%` | `--input/30` · from file |
| Derived | multi-step only | when `TokenAlpha` is insufficient |
| Template | disabled button | “template: opacity 0.56” |
| Studio override | sidebar edit | “override on --primary” |

Showing only `--accent` for row hover would be misleading without the `/40` suffix.

---

## Opacity notation (`--token/xx`)

shadcn/Tailwind express opacity as a **suffix on the token**: `bg-input/30` → **`--input/30`**. This is one of the hardest things to grep for in a CSS file because alpha often **never appears in the file** — the look applies it in Rust when building palettes or resolving controls.

Alpha enters the pipeline in three ways; the inspector must label all three but use the **same display token** where possible:

| Path | Where it happens | CSS file contains `/xx`? | Inspector display |
|------|------------------|--------------------------|-------------------|
| **Declaration slash** | `parse_css_color` parses `hsl(...) / 30%` or `oklch(...) / 0.3` | Yes | `:root --input/30` + raw line + optional line number |
| **Resolver `TokenAlpha`** | `with_alpha(catalog.color("input")?, 0.30)` in palette/resolvers | **No** — file has full `--input` only | `--input/30` + badge “applied in look” + `applied_at` path |
| **Whisper / composite** | e.g. `resolve_accent_whisper(..., 0.4)` for list row hover | No | `--accent/40` |

Canonical display: **`--{token}/{percent}`** where `percent` is 0–100 (`0.30` → `/30`). Aligns with `ShadcnStyle { token, opacity }` in `crates/look-shadcn/src/tokens.rs`.

### Example: outline button (dark)

`outline_role` in `crates/look-shadcn/src/palette.rs` reads full-opacity `--input` from the catalog then applies alpha in Rust:

```text
button [outline, dark, default]
  background   hsl(0 0% 25% / 30%)   --input/30   applied in look · outline_role · dark
  border       hsl(0 0% 25% / 100%)  --input      :root · native.css:78

button [outline, dark, hovered]
  background   hsl(0 0% 25% / 50%)   --input/50   applied in look · outline_role · dark · hover
  foreground   …                     --accent-foreground   …
```

Searching the CSS file for `/30` finds nothing — that is expected. The inspector is the tool that connects painted alpha back to **`--input/30`** as shadcn would name it.

Same pattern elsewhere: textfield dark surface (`--input/30` in `textfield.rs`), ghost dark hover (`--accent/50` in `palette.rs`), list row hover (`--accent/40` via `resolve_accent_whisper`).

**Do not** reverse-engineer `/xx` from final `Hsla.a` at inspect time — multiple steps can stack and the semantic suffix comes from the resolver call site (`with_alpha(input, 0.30)`), not from guessing. Provenance must be captured when `with_alpha` / `resolve_accent_whisper` run.

---

## Theme Studio integration

Existing groundwork:

- `InspectableId` — panel selection on the demo board (`apps/theme-studio/src/studio/inspectable.rs`)
- Selection border when a demo card is selected (`apps/theme-studio/src/studio/panels/mod.rs`)
- Live token overrides in the theme sidebar (`theme_sidebar.rs`, `with_color_overrides` on `ShadcnLook`)

Proposed flow:

1. User selects **Payments** (or toggles inspector pick mode).
2. App calls `PaymentsPanel::inspect_tree(window, cx)`.
3. Popup or side panel renders `InspectNode` recursively.
4. Optional: click a `--token` row → scroll/focus that token in the sidebar accordion.

Effective colors must use the **same** `ShadcnLook` instance (including overrides) as the rendered board.

---

## vs static metadata (`usage.rs`)

| | Gallery Theme Parts (`usage.rs`) | Live inspector |
|--|----------------------------------|----------------|
| Data | Hand-maintained `ThemePartUsage` | Resolver output at probe time |
| Scope | Component type | This panel instance |
| Field names | Rust paths (`ListViewAppearance.background`) | Short field names (`background`) |
| CSS link | Token name only | Token + selector + raw + line |
| Overrides | No | Yes |
| Derived colors | Often stale or simplified | Matches resolver code |

Keep `usage.rs` for gallery docs and onboarding. Do **not** drive the live inspector from it — it drifts (e.g. row hover is `--accent/40`, not a plain `--accent` fill; outline dark uses `--input/30` applied in `palette.rs`, not a CSS line).

---

## Implementation phases

1. **Types** — `InspectNode`, `ResolvedColor`, `ColorSource` in SDK or a small `gpui-luma-inspect` module; `InspectAppearance` derive for field enumeration.
2. **Provenance resolve** — extend look-shadcn catalog + `resolve_*` to return `ResolvedColor` (including `TokenAlpha` and `CssVarSlashAlpha`); optional parse spans; thread through `with_alpha` / `resolve_accent_whisper`.
3. **Control probes** — `ThemeInspectable` for `Button`, `Checkbox` (`Button<bool>`), `ListViewControl`; public `resolved_appearance` / `inspect` APIs.
4. **Payments panel** — `inspect_tree` wiring list + row checkboxes + prev/next; Theme Studio popup on `InspectableId::Payments`.
5. **Other demo panels** — Account, Chat, Accordion, … using the same panel composition pattern.
6. **(Optional) CLI generator** — reuse `ResolvedColor` types to dump full variant × state tables for look diffs.

---

## Limitations

- **Raw `div()` chrome** (`card()`, lucide ellipsis) — only inspectable if manually annotated; no automatic GPUI tree walk.
- **Embedded controls** — parent panel must register children in the inspect tree.
- **Pick mode** — requires stable control ids (already used: `studio-payments`, `payments-prev`, …); arbitrary pixels without ids are out of scope for v1.
- **Line numbers** — depend on parsed source being available (embedded `native.css` or file loaded via `ShadcnLook::from_css_path`); overrides have no source line.

---

## Key files (current)

| Area | Path |
|------|------|
| Payments demo panel | `apps/theme-studio/src/studio/panels/payments.rs` |
| Panel selection | `apps/theme-studio/src/studio/inspectable.rs`, `panels/mod.rs` |
| CSS catalog parse | `crates/look-shadcn/src/catalog/parse.rs`, `catalog/mod.rs` |
| Color resolve helpers | `crates/look-shadcn/src/resolve.rs`, `state_color.rs`, `color.rs` |
| Outline / ghost alpha (`--input/30`, …) | `crates/look-shadcn/src/palette.rs` — `outline_role`, `ghost_role` |
| Tailwind `/opacity` style type | `crates/look-shadcn/src/tokens.rs` — `ShadcnStyle` |
| Control resolvers (examples) | `crates/look-shadcn/src/controls/list_view.rs`, `checkbox.rs`, `button.rs` |
| Static usage metadata | `crates/look-shadcn/src/usage.rs` |
| Button render model (existing) | `crates/sdk/src/controls/command/button/control.rs` — `render_model` |
| TextArea resolved appearance pattern | `crates/sdk/src/controls/textfield/control.rs` — `resolved_appearance` |
