# Look theme introspection: Theme Parts and resolver truth

This document proposes a **system-wide fix** for gallery **Theme Parts** sidebars and the **Theme Usage** pane. The problem is not button-specific: static metadata in `look-shadcn` drifts from actual resolver behavior across most gallery controls.

**Related:**

| Doc / path | Role |
|---|---|
| [`look-cleanup.md`](look-cleanup.md) | SDK vs look-shadcn boundary; structural tokens vs product styling |
| [`architecture.md`](architecture.md) | Crate layout; gallery loads CSS via `ShadcnLook` |
| [`docs/retired/radix-shadcn.md`](../retired/radix-shadcn.md) | Informal token glossary (partially stale) |
| [`crates/look-shadcn/src/usage.rs`](../crates/look-shadcn/src/usage.rs) | Current hand-written `ThemePartUsage` registry |
| [`apps/gallery/src/gallery/panes/shared/mod.rs`](../../apps/gallery/src/gallery/panes/shared/mod.rs) | Per-pane **Theme Parts** sidebar renderer |
| [`apps/gallery/src/gallery/panes/theme_usage/pane.rs`](../../apps/gallery/src/gallery/panes/theme_usage/pane.rs) | Global **Theme Usage** pane |

---

## Problem statement

Gallery panes show live controls on the left and a **Theme Parts** panel on the right. Developers use this to answer: *which CSS custom properties drive which visual parts, in which interaction states?*

Today that panel is **misleading** for most controls because:

1. **Metadata drifts from resolvers** — `usage.rs` is hand-maintained and lags `look-shadcn` control code (e.g. outline/ghost button mode splits, opacity composites, hover foreground).
2. **Schema is too coarse** — `ThemePartUsage` has only `part`, `token`, `states`, `appearance_fields`. It cannot express variants (outline vs ghost), theme mode (light vs dark), opacity (`input/30`, `accent/50`), conditional logic, or non-catalog sources (`transparent`, `palette.accent_foreground`).
3. **Swatches show catalog defaults, not computed appearance** — the sidebar calls `ShadcnLook::token_color(name)`, which reads the raw CSS catalog. It never runs the same resolver path as the control, so composites and state-specific colors are wrong or absent.
4. **Naming is legacy** — entries still use pre-cleanup SDK labels (`prominent`, `standard`, `subtle`) instead of shadcn variant names (`primary`, `secondary`, `outline`, `ghost`).
5. **Pane demos and metadata are loosely coupled** — a pane may show state matrices or variant rows that expose more truth than Theme Parts lists (button pane is the clearest example), with no link between them.

This is the same class of bug everywhere Theme Parts appears: **Button**, **Checkbox**, **Switch**, **Slider**, **TextField**, **Selector**, **Navigation Sidebar**, etc. (~25 component entries in `all_shadcn_theme_usages()`).

We do **not** need a tweakcn-style DOM inspector (hover pick + CSS selector stack). We need **resolver-accurate, variant-aware documentation rendered next to the demo**.

---

## Current architecture

```text
tweakcn CSS (:root / .dark)
        │
        ▼
ShadcnLook::from_css_str / from_css_path
        │
        ├── CssTokenCatalog (light + dark maps)
        └── ShadcnModeTokens per mode
                 ├── ShadcnPalette (action roles, app colors, …)
                 ├── StateColorTable (primary/secondary hover algorithm)
                 └── MetricTokens + typography from catalog

look-shadcn control resolvers (button.rs, checkbox.rs, …)
        │
        ▼
*Appearance / *Palette structs (SDK types)
        │
        ▼
SDK templates paint GPUI elements

Parallel path (documentation only):
usage.rs  ──►  ThemePartUsage[]  ──►  gallery Theme Parts sidebar
              (static, often stale)
```

**Gallery wiring:**

- Most panes call `gallery_pane_with_usage(title, usage_component, content, look)`.
- `usage_component` is a string key matched against `ThemeUsage.label` (e.g. `"Button"`, `"Checkbox"`).
- Some panes pass **multiple** keys (e.g. ComboBox → `["ComboBox", "Floating Menu"]`).
- **Theme Usage** pane renders the same registry globally (by token, by component, shared values).

**What panes already do well:**

- Many panes include **state preview matrices** (default / hover / focused / pressed / disabled) that call `ShadcnLook::resolve_*` or equivalent — these show **computed** colors.
- Theme Parts does not reference those matrices.

---

## Design goals

1. **Truth** — Every row in Theme Parts must match what the resolver returns for the active theme, mode, and interaction state.
2. **Completeness** — Cover variants, states, and appearance fields developers care about (including opacity and mode splits).
3. **Stability** — Metadata must not drift when resolvers change; prefer generation or co-location over hand-edited lists.
4. **Simplicity** — No full CSS inspector; readable cards in the sidebar are enough.
5. **Scope** — Look-shadcn owns product token mapping; SDK keeps generic `ThemeUsage` types and gallery rendering.

Non-goals:

- Parsing Tailwind class strings from shadcn source.
- Runtime hover-pick of arbitrary GPUI elements.
- Documenting SDK `Default*Theme` fallbacks (gallery always uses `ShadcnLook` + CSS).

---

## Proposed model: `ThemeBinding`

Replace the flat `ThemePartUsage` row with a richer **binding** that describes one slice of resolved appearance.

### Suggested fields

| Field | Purpose | Example |
|---|---|---|
| `component` | Gallery / usage label | `"Button"` |
| `variant` | Product variant or sub-style | `"outline"`, `"ghost"`, `"standard"`, `"checked"` |
| `interaction` | Normalized interaction layer | `default`, `hover`, `pressed`, `focused`, `disabled` |
| `mode` | Light/dark applicability | `light`, `dark`, `both` |
| `appearance_field` | SDK struct field | `ButtonFamilyAppearance.foreground` |
| `source` | How the color is obtained | See `ThemeSource` below |
| `note` | Optional human hint | `"shadcn: hover:text-accent-foreground"` |

### `ThemeSource` (token expression)

Bindings need more than a bare CSS key:

```rust
enum ThemeSource {
    /// Raw catalog token: `--accent`
    Token { name: &'static str },
    /// Catalog token with alpha: `--input` at 30% (dark outline bg)
    TokenAlpha { name: &'static str, alpha: f32 },
    /// Fully transparent
    Transparent,
    /// Derived from ShadcnPalette field (not a CSS key)
    Palette { field: &'static str },
    /// From StateColorTable / algorithmic hover on a semantic token
    StateColor { token: ShadcnToken, layer: InteractionLayer },
    /// Fixed resolver branch (document only)
    Resolver { path: &'static str },
}
```

Opacity and mode splits become explicit without pretending `input/30` exists in CSS.

### Backward compatibility

Keep `ThemePartUsage` as a **projection** for simple consumers, or deprecate gradually:

```text
ThemeBinding (canonical)  →  display adapter  →  Theme Parts card
```

---

## Source of truth: three tiers

Pick one primary strategy per control; mix tiers during migration.

### Tier A — **Derived bindings** (preferred)

At compile time or in tests, enumerate `(variant, interaction, mode)` tuples, call the public resolver (`resolve_outline_button`, `checkbox_appearance`, …), and **diff** the resulting appearance against catalog tokens to emit bindings.

**Pros:** Cannot drift; swatches always match demo.  
**Cons:** Upfront harness; naming/variant enumeration must be curated.

**Implementation sketch:**

```text
crates/look-shadcn/src/theme/bindings/
  mod.rs           // ThemeBinding types + registry
  button.rs        // enumerate ShadcnButtonStyle × InteractionState × ThemeMode
  checkbox.rs
  …
  generate.rs      // optional: test-only generator that fails CI on drift
```

CI test pattern:

```text
for each binding in derived_button_bindings():
    assert_resolved_color(binding) matches binding.source
```

When a developer changes `outline_role`, the derived table updates or CI fails until bindings are regenerated.

### Tier B — **Co-located static tables**

Define `theme_bindings()` next to each resolver module (`controls/button.rs`, `controls/checkbox.rs`, …), same file as `button_palette`.

**Pros:** Low ceremony; review happens in the same PR as resolver changes.  
**Cons:** Still manual; easy to forget new states.

### Tier C — **Hand-maintained central registry** (status quo)

Single `usage.rs` edited independently.

**Verdict:** Retire Tier C as authoritative source. Keep `usage.rs` only as a re-export of Tier A/B until migration completes.

---

## Gallery UX improvements

### 1. Resolved swatches

Theme Parts cards should show:

- **Computed color** from the resolver (primary display)
- **Catalog token** reference (secondary monospace line)
- **Mode badge** when light ≠ dark (`L` / `D` / `L+D`)

Replace `look.token_color(part.token)` with something like:

```text
look.resolve_part(component, variant, interaction) → Hsla
```

### 2. Align interaction vocabulary

Use the same labels as pane state matrices:

| UI label | `InteractionState` |
|---|---|
| default | `{}` |
| hover | `{ hovered: true }` |
| pressed | `{ hovered: true, pressed: true }` |
| focused | `{ focused: true }` |
| disabled | `{ disabled: true }` |

Retire legacy state strings (`prominent default`, `ghost hovered`, `subtle default`).

### 3. Variant grouping

For multi-variant controls, group Theme Parts by variant:

```text
Button
  Primary
    default · background · primary
    hover   · background · primary (state table)
    …
  Outline
    default · background · background (light) / input@30% (dark)
    hover   · foreground · accent-foreground
    …
  Ghost
    …
```

Icon Button and Toggle reuse Button bindings with `role` annotations.

### 4. Optional matrix ↔ sidebar link (lightweight inspector)

Without hover-pick:

- When a pane has a state matrix (button, checkbox, …), clicking a **column header** (`hover`) filters Theme Parts to `interaction = hover`.
- When a pane has variant **rows** (Outline, Ghost), clicking the row label filters `variant = outline`.

No new GPUI hit-testing on arbitrary elements — reuse existing preview chrome.

### 5. Theme Usage pane upgrades

Extend the global pane to:

- Show **derived vs catalog** columns for each token
- List **opacity composites** (`input/30`, `accent/50`) explicitly
- Flag **resolver-only** colors (`transparent`, algorithmic hover)
- Report **coverage**: N bindings / M appearance fields for component

---

## Component inventory and migration notes

All entries in `all_shadcn_theme_usages()` need Tier A or B treatment. Priority by drift risk and gallery traffic:

| Component | Gallery pane(s) | Resolver module(s) | Known gaps in current metadata |
|---|---|---|---|
| Button | button, icon_button, toggle | `controls/button.rs`, `palette.rs` | Variants, mode splits, hover fg, opacity, legacy names |
| Checkbox | checkbox, toggle_group | `controls/checkbox.rs` | Style variants (`ShadcnButtonStyle`), unchecked vs checked |
| Switch | switch | `controls/switch.rs` | On/off track vs thumb; style variants |
| Radio Button | radio_button, radio_group | `controls/radio.rs` | Same family as checkbox |
| Slider | slider | `controls/slider.rs` | Track vs fill vs thumb |
| Scrollbar | scrollbar | `controls/scrollbar.rs` | Minimal but state-limited |
| TextField | textfield, search flows | `controls/textfield.rs` | Surface/Soft via `ShadcnTextFieldStyle`; bindings in `usage.rs` (soft fg TBD) |
| TextArea | textarea | `controls/textarea.rs` | Same Surface/Soft path as textfield |
| ComboBox | combobox | textfield + selector panel | Multi-component usage array |
| AutocompleteTextField | autocomplete | autocomplete + floating menu | Partial |
| SearchSelector | search_selector | search_selector + textfield | Partial |
| Selector | selector | `controls/selector.rs` | Ghost trigger vs accent panel |
| Popup Menu | popup_menu | `controls/popup_menu.rs` | Ghost trigger foreground |
| Context Menu | context_menu | `controls/context_menu.rs` | Same |
| Floating Menu | floating_menu | `controls/floating_menu.rs` | Item hover pair |
| Accordion | accordion | `controls/accordion.rs` | Trigger vs content palettes |
| Tabs Navigation | tabs_navigation | `controls/tabs_navigation.rs` | Active uses primary state table |
| Navigation Sidebar | navigation_sidebar | `controls/navigation_sidebar.rs` | Sidebar-* tokens |
| TreeView | tree_view | `controls/tree_view.rs` | Sidebar tokens |
| Control Group | toggle_group | `controls/control_group.rs` | Muted surface |
| ListBox | listbox | `controls/listbox.rs` | Row hover accent |
| ListView | list_view (×2) | `controls/list_view.rs` | Row hover uses accent @ 40% alpha — not in schema |
| ResizablePanels | resizable_panels | `controls/resizable_panels.rs` | Handle/grip emphasis |
| SplitView | split_view (×4) | split_view in look-shadcn | Separator hover |
| Progress | progress | `controls/progress.rs` | Track vs fill |
| Selection Panel | selection_panel | `controls/selection_panel.rs` | Popover surface |

**Panes without Theme Parts today** (introduction, prototypes, choice_controls_template, …) can adopt the same binding registry when useful — not blocking.

---

## Reference: Button bindings target (outline + ghost)

Use this checklist when implementing Tier A/B for buttons. Other variants follow the same pattern.

### Outline

| Variant | Interaction | Field | Light source | Dark source |
|---|---|---|---|---|
| outline | default | background | `background` | `input` @ 30% |
| outline | default | foreground | `foreground` | `foreground` |
| outline | default | border | `border` | `input` |
| outline | hover | background | `accent` | `input` @ 50% |
| outline | hover | foreground | `accent-foreground` | `accent-foreground` |
| outline | pressed | (same as hover) | | |
| outline | disabled | background | `muted` | `muted` |
| outline | disabled | foreground | `muted-foreground` | `muted-foreground` |
| outline | focused | adorner | `ring` | `ring` |

### Ghost

| Variant | Interaction | Field | Light source | Dark source |
|---|---|---|---|---|
| ghost | default | background | transparent | transparent |
| ghost | default | foreground | `foreground` | `foreground` |
| ghost | hover | background | `accent` | `accent` @ 50% |
| ghost | hover | foreground | `accent-foreground` | `accent-foreground` |
| ghost | pressed | (same as hover) | | |
| ghost | focused | adorner | `ring` (inset) | `ring` (inset) |

Primary/secondary add **StateColorTable** rows for hover/pressed on `primary` / `secondary` tokens.

---

## Implementation phases

### Phase 1 — Schema and API (look-shadcn + sdk)

1. Add `ThemeBinding` / `ThemeSource` types (sdk or look-shadcn; prefer sdk if gallery already imports `ThemePartUsage` from sdk).
2. Add `ShadcnLook::resolve_binding(...)` or per-control helpers returning computed `Hsla` + source metadata.
3. Keep `ThemePartUsage` as deprecated adapter during transition.

### Phase 2 — Button family pilot

1. Implement derived or co-located bindings for `ShadcnButtonStyle` × interactions × modes.
2. Update gallery Theme Parts renderer to use resolved swatches + variant groups.
3. Wire button pane matrix column filter (optional but high value).
4. Add CI test: bindings match `resolve_*_button` for native CSS fixture theme.

### Phase 3 — Roll out by control family

Suggested order (reuse patterns from Phase 2):

1. **Toggle family** — checkbox, switch, radio (shared button-style variants)
2. **Form inputs** — textfield, textarea (Surface / Soft style split in look-shadcn)
3. **Menus / overlays** — selector, popup, context, floating, selection panel
4. **Navigation** — tabs, sidebar, tree
5. **Collections** — listbox, listview, control group
6. **Chrome** — slider, scrollbar, progress, resizable/split

Each phase: bindings module + gallery metadata key unchanged + CI drift test.

### Phase 4 — Theme Usage pane and docs

1. Upgrade Theme Usage pane to binding-aware views.
2. Mark `docs/retired/radix-shadcn.md` sections as superseded by this doc + live gallery.
3. Add `look-theme.md` reference to `AGENTS.md` / `module-map.md`.

---

## Acceptance criteria

Theme introspection is “done” when:

- [ ] **No authoritative hand list in `usage.rs`** — bindings come from Tier A or B per component.
- [ ] **Theme Parts swatches match pane state matrices** for the active theme and mode (spot-check: outline/ghost hover fg/bg, checkbox checked, switch on/off).
- [ ] **Every gallery pane with `gallery_pane_with_usage`** has complete bindings for its `usage_component` key(s).
- [ ] **CI fails on drift** — changing a resolver without updating bindings breaks a test.
- [ ] **Variant + interaction + mode** are visible in the sidebar (not only a flat token list).
- [ ] **Opacity composites** (`token@50%`) display correctly, not as raw catalog `--token`.
- [ ] **Legacy state names removed** (`prominent`, `standard`, `subtle`) from user-visible Theme Parts copy.

---

## Open questions

1. **Public API surface** — Should `ThemeBinding` live in `gpui_luma::theme` for reuse outside gallery, or stay in `gpui_luma_look_shadcn`?
2. **Metrics as theme parts** — Do we document `MetricTokens` (radius, padding, height) in the same sidebar, or a separate “Layout parts” section?
3. **Multi-theme preview** — Should Theme Parts show light and dark columns simultaneously, or follow `ShadcnLook::mode()` only?
4. **Codegen** — Long term, should bindings be generated from resolver macros (see [`next-step-codegen.md`](next-step-codegen.md))?

---

## Summary

Gallery **Theme Parts** is the right idea but the wrong implementation: a static, coarse registry that shows catalog colors instead of resolver output. The fix is **resolver-accurate bindings** (derived or co-located), a **richer schema** (variant, interaction, mode, opacity), and **gallery UI** that groups and filters like a lightweight inspector — applied **once across all controls**, starting with the button family pilot that motivated this work.
