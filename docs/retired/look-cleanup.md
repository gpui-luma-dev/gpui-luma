# Purging Shadcn-isms from the SDK: Transition Checklist

This document records the migration to remove Shadcn/Radix-specific assumptions and naming conventions ("Shadcn-isms") from the core [gpui-luma SDK](../crates/sdk).

**Related:** Concrete slice of Phase B in [`fundamental-templates.md`](fundamental-templates.md) — the SDK exposes structural tokens and complete `Appearance` types; product styling (`primary`, `secondary`, ghost buttons, etc.) belongs in `look-shadcn` and optional theme crates.

## Status (June 2026)

**Phases 1–3 and most of Phase 5 are complete.** Remaining work is primarily **Phase 4** (`MetricTokens` deduplication) and a **docs sweep** (`docs/theme.md`, `control-design.md`, etc.).

| Phase | Status |
|---|---|
| 1 — SDK default resolvers | Done |
| 2 — look-shadcn bridge + legacy TOML | Done |
| 3 — Schema cleanup (`ActionPalette`, TOML pipeline) | Done |
| 4 — Metric / variant cleanup | **Open** — top-level `MetricTokens.sm/md/lg` still duplicates `control.*` |
| 5 — Docs, tests, apps | Partial — this doc + gallery updated; see Phase 5 table |

---

## Background: SDK vs themes

The SDK does **not** own product theming. Apps theme controls by injecting a look (`ShadcnLook::from_css_path(...)`) that resolves `*Appearance` at runtime. What remains in the SDK is:

- **Structural token types** (`LumaPalette`, `MetricTokens`, …) — the vocabulary `Default*Theme` resolvers read when no look is attached.
- **Default resolvers** (`DefaultButtonFamilyTheme`, etc.) — unthemed fallbacks so controls render in tests and bare builder paths without panicking.
- **Templates** — paint `Appearance` structs; they never choose `primary` vs `secondary`.

Product themes live in `look-shadcn` (CSS catalog → `ShadcnPalette` → control appearances). The SDK does not ship a polished, named theme file.

### Legacy `default-theme.toml` (removed)

[`default-theme.toml`](../crates/sdk/src/theme/default-theme.toml) was **deleted**. It was legacy from when the design system and controls lived in one crate (old `lexicon.toml` + `default-theme.toml` → `theme.toml` pipeline).

**Current defaults:**

- `ThemeTokens::default()` → `LumaThemeMode::light()` (Rust constructors in `tokens.rs`).
- `ShadcnLook::native()` → embedded shadcn CSS in `look-shadcn` (`assets/native.css`), **not** `LumaTheme::native()`.
- TOML import (`from_toml_str`, `RawTheme` deserializer) removed from the SDK.

## Design philosophy

1. **Rendering templates remain in the SDK** — developers can override layouts locally via `.with_template(...)`.
2. **No styling taxonomy leaks into the SDK** — reject names like `primary`, `secondary`, `prominent`, and `ghost` in core palette types. The SDK represents universal structural primitives (surfaces, boundaries, text, and states).
3. **No product theme in the SDK** — the SDK keeps only minimal structural defaults for unthemed resolver paths.
4. **Migrate resolvers before deleting schema** — completed: `palette.action.*` removed from default themes, then `ActionPalette` deleted from `LumaPalette`.

---

## Execution order (historical)

```text
Phase 1 — SDK default resolvers     ✓ stop reading palette.action.* in crates/sdk
Phase 2 — look-shadcn + legacy TOML ✓ delete from_luma_tokens bridge; kill default-theme.toml
Phase 3 — Schema cleanup            ✓ delete ActionPalette, ColorTokens shadcn-isms, RawTheme pipeline
Phase 4 — Variant / metric cleanup  ◐ Surface/Soft in look-shadcn done; MetricTokens duplication open
Phase 5 — Docs, tests, apps           ◐ gallery + theme-studio updated; broader doc sweep open
```

---

## Acceptance criteria

- [x] Zero `palette.action` / `ActionPalette` / `ActionRolePalette` references in `crates/sdk`.
- [x] `ColorTokens` removed (no `prominent*` or ghost-derived aliases in SDK palette).
- [x] `ShadcnPalette::from_luma_tokens` and `ShadcnModeTokens::from_luma_tokens` deleted; `look-shadcn` loads styling only from CSS catalog or embedded native CSS.
- [x] `default-theme.toml` deleted; `ThemeTokens::default()` uses `LumaThemeMode::light()` (Rust constructors), not TOML parsing.
- [x] `ShadcnLook::native()` does not call `LumaTheme::native()`.
- [x] `LumaTheme::from_toml_str` and the `RawTheme` deserializer pipeline removed from the SDK.
- [x] `TextFieldVariant` has no ghost branch in the SDK (sole variant: `Standard`). Look-specific **Surface / Soft** live in `ShadcnTextFieldStyle`, not in the SDK enum.
- [ ] `MetricTokens` has a single size scale (`control.sm/md/lg` via `for_size()`), not duplicated top-level `sm/md/lg`.
- [x] Gallery and theme-studio run with `ShadcnLook::native()` / CSS themes without referencing SDK action types.
- [x] `cargo fmt`, `cargo clippy`, and SDK / look-shadcn theme tests pass.

---

## Phase 1 — SDK default resolvers (done)

### 1. Button family theme

[`crates/sdk/src/controls/button_family/theme.rs`](../crates/sdk/src/controls/button_family/theme.rs) — `native_action_role` removed; roles resolve from structural tokens (`surface.*`, `state.*`, `border.*`).

### 2. Text field: SDK variant vs look Surface / Soft

**SDK (done):** `TextFieldVariant` is **`Standard` only** in [`textfield/theme.rs`](../crates/sdk/src/controls/textfield/theme.rs). `DefaultTextFieldTheme` uses `form.input.*`, `state.selected.*`, and `focus.ring`. No ghost branch.

**look-shadcn (done):** Product variants are **`ShadcnTextFieldStyle::Surface` | `Soft`** in [`controls/textfield.rs`](../crates/look-shadcn/src/controls/textfield.rs):

| Variant | Chrome | Text / selection (current) |
|---|---|---|
| **Surface** | shadcn Input: `border-input`, light transparent fill, dark `input/30` | `foreground`, `muted-foreground` placeholder, `primary` / `primary-foreground` selection |
| **Soft** | Radix soft: `muted` fill, no border | Same text/selection tokens as surface (soft-specific fg TBD when Radix specs land) |

**APIs:**

- `ShadcnLook::textfield_theme()` → surface (default interactive path)
- `ShadcnLook::soft_textfield_theme()` → soft (gallery state matrix, optional app wiring)

**TextArea (done):** [`controls/textarea.rs`](../crates/look-shadcn/src/controls/textarea.rs) delegates to `textfield_palette(..., style, ...)`. Default control uses **surface**; `soft_textarea_theme()` mirrors text field.

**SDK templates (done):** `TextFieldPalette` and `TextAreaPalette` include `selection_foreground`; templates and TextArea native paint use solid `primary` highlight with contrasting selected text.

**Apps (done):**

- [`textfield/pane.rs`](../apps/gallery/src/gallery/panes/textfield/pane.rs) — Surface / Soft state previews; removed `appearance_override` demo fields.
- [`textarea/pane.rs`](../apps/gallery/src/gallery/panes/textarea/pane.rs) — Surface / Soft state previews.

**Search selector (done):** uses `TextFieldVariant::Standard` only ([`search_selector/control.rs`](../crates/sdk/src/controls/search_selector/control.rs)).

> **Naming note:** Button **Ghost** (`ShadcnButtonStyle::Ghost`) remains a valid *button* variant in look-shadcn. Input **Soft** is not button ghost — do not conflate them.

### 3. Menu and list item hovers (done)

Popup menu, selector, and context menu default themes use `state.hover` / `state.pressed` / `app.background`, not `action.ghost`.

### 4. Toggle and selection indicators (done)

Checkbox, switch, and radio use `state.selected.*` instead of `action.prominent`.

### 4b. Remaining action resolvers (done)

Slider, progress, tabs navigation, navigation sidebar, and resizable panels migrated off `action.prominent` / `action.ghost`.

---

## Phase 2 — look-shadcn bridge and legacy TOML removal (done)

- `from_luma_tokens`, `ShadcnLook::from_theme`, `ShadcnLook::from_toml_str` removed.
- `ShadcnLook::native()` loads embedded CSS (`assets/native.css`).
- `default-theme.toml` and TOML parse path deleted from SDK.
- [`chat.rs`](../apps/theme-studio/src/studio/panels/chat.rs) uses `ShadcnLook` resolver APIs (e.g. `palette.primary`), not `palette.action(...)`.

---

## Phase 3 — Schema cleanup (done)

- `ActionPalette`, `ActionRolePalette`, and `action` field removed from `LumaPalette`.
- `ColorTokens` and `RawTheme` / `from_toml_str` pipeline removed from SDK.
- `LumaPalette` is structural only (`app`, `surface`, `state`, `form`, `focus`, `border`, `navigation`, `data`).

---

## Phase 4 — Metric cleanup (open)

### Simplify `MetricTokens`

[`MetricTokens`](../crates/sdk/src/theme/tokens.rs) still duplicates size scales:

- `control.sm` / `control.md` / `control.lg` — used by `for_size()` ✓ keep
- Top-level `sm` / `md` / `lg` — redundant copy of heights (`28.0`, `36.0`, `44.0`) ✗ remove

**Steps:**

1. Remove top-level `sm`, `md`, `lg` from `MetricTokens`.
2. Keep `ControlSize` enum and `control: ControlMetricScale` as the single SDK size ladder.
3. Update [`theme/cache.rs`](../crates/sdk/src/theme/cache.rs) if layout cache keys hash top-level `sm/md/lg`.
4. Let `look-shadcn` templates map `ControlSize` → pixel heights from CSS catalog or local constants.

Specific pixel heights are styling details; the SDK only needs relative size steps.

---

## Phase 5 — Docs, tests, and validation (partial)

### Documentation still to sweep

| Doc | What to fix |
|---|---|
| [`docs/theme.md`](../theme.md) | Remove any remaining `action.*` token catalog; document structural palette only |
| [`docs/ai/module-map.md`](module-map.md) | TextField: Surface/Soft in look-shadcn; SDK `Standard` only |
| [`docs/control-design.md`](../control-design.md) | Toggle segment guidance referencing `action.subtle` |
| [`docs/ai/fundamental-templates.md`](fundamental-templates.md) | Mark look-cleanup complete when Phase 4 lands |
| [`docs/ai/look-theme.md`](look-theme.md) | Theme Parts bindings for Surface/Soft text inputs |

### Tests and CI

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p gpui-luma
cargo test -p gpui-luma-look-shadcn
just gallery    # smoke: native + CSS themes
just theme-studio
```

### Inventory reference

Verify completion / guard against regressions:

```bash
rg 'palette\.action|ActionPalette|ActionRolePalette' crates/sdk
rg 'from_luma_tokens|LumaTheme::native|default-theme\.toml|from_toml_str' crates apps
rg 'TextFieldVariant::Ghost|ShadcnTextFieldStyle::Ghost|ghost_textfield' crates apps
rg 'prominent' crates/sdk/src/theme/tokens.rs
rg 'pub sm: f32' crates/sdk/src/theme/tokens.rs   # Phase 4: should be 0 after dedup
```

---

## File index (quick reference)

| Area | Primary files |
|---|---|
| Palette schema | `crates/sdk/src/theme/tokens.rs`, `mod.rs` |
| SDK unthemed defaults | `LumaThemeMode::light()` / `dark()` in `tokens.rs` |
| Product theme | `look-shadcn` CSS catalog (`ShadcnLook::from_css_path`, `native.css`) |
| Button family | `crates/sdk/src/controls/button_family/theme.rs` |
| Text field / area | `crates/sdk/src/controls/textfield/theme.rs`, `textarea/theme.rs`; look: `controls/textfield.rs`, `controls/textarea.rs` |
| Menus / lists | `popup_menu/theme.rs`, `selector/theme.rs`, `context_menu/theme.rs` |
| Toggles | `checkbox/theme.rs`, `switch/theme.rs`, `radio_button/theme.rs` |
| Other migrated resolvers | `slider/theme.rs`, `progress/theme.rs`, `tabs_navigation/theme.rs`, `navigation_sidebar/theme.rs`, `resizable_panels/theme.rs` |
| look-shadcn | `palette.rs`, `mode.rs`, `look.rs`, `usage.rs`, `controls/*.rs` |
| Apps | `apps/gallery/.../textfield/pane.rs`, `textarea/pane.rs`, `apps/theme-studio/.../chat.rs` |
