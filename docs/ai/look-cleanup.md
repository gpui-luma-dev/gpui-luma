# Purging Shadcn-isms from the SDK: Transition Checklist

This document details the incremental steps needed to remove Shadcn/Radix-specific assumptions and naming conventions ("Shadcn-isms") from the core [gpui-luma SDK](../crates/sdk).

**Related:** This is a concrete slice of Phase B in [`fundamental-templates.md`](fundamental-templates.md) — the SDK should expose structural tokens and complete `Appearance` types; product styling (`primary`, `secondary`, ghost buttons, etc.) belongs in `look-shadcn` and optional theme crates.

## Background: SDK vs themes

The SDK does **not** own product theming. Apps theme controls by injecting a look (`ShadcnLook::from_css_path(...)`) that resolves `*Appearance` at runtime. What remains in the SDK is:

- **Structural token types** (`LumaPalette`, `MetricTokens`, …) — the vocabulary `Default*Theme` resolvers read when no look is attached.
- **Default resolvers** (`DefaultButtonFamilyTheme`, etc.) — unthemed fallbacks so controls render in tests and bare builder paths without panicking.
- **Templates** — paint `Appearance` structs; they never choose `primary` vs `secondary`.

Product themes live in `look-shadcn` (CSS catalog → `ShadcnPalette` → control appearances). The SDK should not ship a polished, named theme file.

### What `default-theme.toml` is (and why it should go)

[`default-theme.toml`](../crates/sdk/src/theme/default-theme.toml) is **legacy** from when the design system and controls lived in one crate. It was the base layer in the old import pipeline (`lexicon.toml` + `default-theme.toml` → `theme.toml`). That pipeline is retired; gallery and theme-studio load tweakcn CSS directly.

Today the file is a ~600-line duplicate of data that already exists as Rust:

| Source | Used by |
|---|---|
| `default-theme.toml` → `LumaTheme::native()` → TOML parse | `ThemeTokens::default()`, `ShadcnLook::native()` (legacy bridge) |
| `LumaPalette::light()` / `dark()`, `MetricTokens::default()`, … | `LumaThemeMode::light()` / `dark()` — already compile-time Rust |

`ThemeTokens::default()` currently chains through `LumaTheme::native()` and parses TOML at first use, even though `LumaThemeMode::light()` could be used directly. **`default-theme.toml` should be deleted, not migrated to another module.** Do not add `default_theme.rs` as a replacement full theme — that would preserve the wrong abstraction.

After cleanup:

- `ThemeTokens::default()` → `LumaThemeMode::light()` (existing Rust constructors).
- `ShadcnLook::native()` → embedded shadcn CSS or compile-time shadcn defaults in `look-shadcn`, **not** `LumaTheme::native()`.
- `LumaTheme`, `from_toml_str`, and the `RawTheme` deserializer pipeline → remove from SDK (or relocate to a theme crate if TOML import is ever revived).

## Design philosophy

1. **Rendering templates remain in the SDK** — developers can override layouts locally via `.with_template(...)`.
2. **No styling taxonomy leaks into the SDK** — reject names like `primary`, `secondary`, `prominent`, and `ghost` in core palette types. The SDK represents universal structural primitives (surfaces, boundaries, text, and states).
3. **No product theme in the SDK** — delete `default-theme.toml` and shrink `LumaTheme::native()`; the SDK keeps only minimal structural defaults for unthemed resolver paths.
4. **Migrate resolvers before deleting schema** — remove `palette.action.*` usage from default themes first, then delete `ActionPalette` from `LumaPalette`.

---

## Execution order

Do **not** delete `ActionPalette` until default resolvers and the `look-shadcn` bridge are migrated. Recommended phases:

```text
Phase 1 — SDK default resolvers     → stop reading palette.action.* in crates/sdk
Phase 2 — look-shadcn + legacy TOML → delete from_luma_tokens bridge; kill default-theme.toml
Phase 3 — Schema cleanup              → delete ActionPalette, ColorTokens shadcn-isms, RawTheme pipeline
Phase 4 — Variant / metric cleanup    → TextFieldVariant::Ghost, MetricTokens duplication
Phase 5 — Docs, tests, apps           → gallery, theme-studio, docs/theme.md
```

Within Phase 1, prefer this order (each step is independently committable):

1. Menu/list hovers (§3) — low risk, no public API change.
2. Toggle indicators (§4) — checkbox, switch, radio.
3. Remaining `action.prominent` resolvers (§4b) — slider, progress, tabs, nav sidebar.
4. Button family defaults (§1) — includes toggle-segment rules.
5. Resizable panel handle hover (§4b).
6. Text field ghost variant (§2).

---

## Acceptance criteria

The cleanup is complete when all of the following hold:

- [ ] Zero `palette.action` / `ActionPalette` / `ActionRolePalette` references in `crates/sdk`.
- [ ] `ColorTokens` no longer exposes `prominent*` or ghost-derived `surface_*` aliases (or the struct is removed if unused).
- [ ] `ShadcnPalette::from_luma_tokens` and `ShadcnModeTokens::from_luma_tokens` are deleted; `look-shadcn` loads styling only from CSS catalog or compile-time shadcn defaults.
- [ ] `default-theme.toml` is deleted; `ThemeTokens::default()` uses `LumaThemeMode::light()` (Rust constructors), not TOML parsing.
- [ ] `ShadcnLook::native()` does not call `LumaTheme::native()`.
- [ ] `LumaTheme::from_toml_str` and the `RawTheme` deserializer pipeline are removed from the SDK (TOML import is not an SDK concern).
- [ ] `TextFieldVariant` has no ghost branch in SDK or look-shadcn resolvers.
- [ ] `MetricTokens` has a single size scale (`control.sm/md/lg` via `for_size()`), not duplicated top-level `sm/md/lg`.
- [ ] Gallery and theme-studio run with `ShadcnLook::native()` / CSS themes without referencing SDK action types.
- [ ] `cargo fmt`, `cargo clippy`, and SDK theme tests pass.

---

## Phase 1 — SDK default resolvers

### 1. Button family theme

[`crates/sdk/src/controls/button_family/theme.rs`](../crates/sdk/src/controls/button_family/theme.rs) — `native_action_role` currently maps roles to `palette.action.subtle` / `palette.action.standard`.

Replace with structural resolution:

| Role | Default | Hover | Pressed | Border |
|---|---|---|---|---|
| `Text`, `Icon` | `surface.subtle.background` or transparent | `state.hover.background` | `state.pressed.background` | `border.default` |
| `Toggle { selected: true }` | `state.selected.background` | `state.selected.background` (or blend) | `state.pressed.background` | `border.default` |
| `Toggle { selected: false }` | `surface.subtle.background` or transparent | `state.hover.background` | `state.pressed.background` | `border.default` |

Foreground: `app.foreground` (default), `state.selected.foreground` when toggle selected, `state.disabled.foreground` when disabled.

Delete `native_action_role` and any import of `ActionRolePalette`.

### 2. Removing `TextFieldVariant::Ghost`

A ghost input is a flat design variation, not a structural SDK variant.

**SDK**

1. In [`crates/sdk/src/controls/textfield/theme.rs`](../crates/sdk/src/controls/textfield/theme.rs), remove `Ghost` from `TextFieldVariant` (keep `Standard` as the sole variant, or drop the enum from the theme trait if only one shape remains).
2. Delete ghost branches in `DefaultTextFieldTheme::resolve`; use `form.input.*` and `focus.ring` only.
3. In [`crates/sdk/src/controls/search_selector/control.rs`](../crates/sdk/src/controls/search_selector/control.rs), remove `.variant(TextFieldVariant::Ghost)`; apply transparent background via template override or local appearance hook.

**look-shadcn**

4. In [`crates/look-shadcn/src/controls/textfield.rs`](../crates/look-shadcn/src/controls/textfield.rs), remove `TextFieldVariant::Ghost` match arms; expose a look-specific ghost appearance via `ShadcnLook` resolver or template factory instead.

**Apps**

5. Update [`apps/gallery/src/gallery/panes/textfield/pane.rs`](../apps/gallery/src/gallery/panes/textfield/pane.rs) — replace ghost demos with look-shadcn template examples.

### 3. Menu and list item hovers

Default item themes assume list rows are hoverable "ghost buttons." Switch to generic state tokens.

| File | Change |
|---|---|
| [`popup_menu/theme.rs`](../crates/sdk/src/controls/popup_menu/theme.rs) | Item + trigger hovers → `state.hover` / `state.pressed` / `state.disabled`; default layer → `app.background` or `surface.panel.background` (not `action.ghost`) |
| [`selector/theme.rs`](../crates/sdk/src/controls/selector/theme.rs) | Same |
| [`context_menu/theme.rs`](../crates/sdk/src/controls/context_menu/theme.rs) | Same |

Example trigger background resolution:

```rust
let trigger_background = match state.layer() {
    InteractionLayer::Disabled => palette.state.disabled.background,
    InteractionLayer::Pressed => palette.state.pressed.background,
    InteractionLayer::Hovered => palette.state.hover.background,
    InteractionLayer::Default => palette.app.background,
};
```

Foreground for items: `app.foreground` (not `action.ghost.foreground`).

### 4. Toggle and selection indicators

Replace `palette.action.prominent` with structural selected-state tokens.

| Control | File | Suggested tokens |
|---|---|---|
| Checkbox | [`checkbox/theme.rs`](../crates/sdk/src/controls/checkbox/theme.rs) | Checked fill: `state.selected.background`; checkmark: `state.selected.foreground` |
| Switch | [`switch/theme.rs`](../crates/sdk/src/controls/switch/theme.rs) | Track on: `state.selected.background`; thumb: `state.selected.foreground` or `surface.panel.background` |
| Radio | [`radio_button/theme.rs`](../crates/sdk/src/controls/radio_button/theme.rs) | Same pattern as checkbox |

### 4b. Remaining `action.prominent` / `action.ghost` resolvers

These files also read action palette slots and must be updated in Phase 1:

| Control | File | Current usage | Suggested replacement |
|---|---|---|---|
| Slider | [`slider/theme.rs`](../crates/sdk/src/controls/slider/theme.rs) | `action.prominent` fill/thumb | `state.selected.background` |
| Progress | [`progress/theme.rs`](../crates/sdk/src/controls/progress/theme.rs) | `action.prominent.background` | `state.selected.background` |
| Tabs navigation | [`tabs_navigation/theme.rs`](../crates/sdk/src/controls/tabs_navigation/theme.rs) | `action.prominent` active tab | `state.selected.background` / `navigation.selected_background` |
| Navigation sidebar | [`navigation_sidebar/theme.rs`](../crates/sdk/src/controls/navigation_sidebar/theme.rs) | `action.prominent` selected item | `navigation.selected_background` |
| Resizable panels | [`resizable_panels/theme.rs`](../crates/sdk/src/controls/resizable_panels/theme.rs) | `action.ghost.hover_background` | `state.hover.background` |

Prefer existing structural slots (`navigation.*`, `state.selected.*`) where the control already has a semantic home; fall back to `state.selected` for generic "active fill" semantics.

---

## Phase 2 — look-shadcn bridge and legacy TOML removal

[`crates/look-shadcn/src/palette.rs`](../crates/look-shadcn/src/palette.rs) already owns `ShadcnPalette` with `primary`, `secondary`, `outline`, and `ghost` loaded from CSS via `from_catalog`. That is the real theme path. Phase 2 removes everything that still pretends the SDK ships a theme.

### Delete the Luma → Shadcn bridge

1. Remove `ShadcnPalette::from_luma_tokens` and its use of `ActionRolePalette` from `gpui_luma::theme`.
2. Remove `ShadcnModeTokens::from_luma_tokens` in [`mode.rs`](../crates/look-shadcn/src/mode.rs).
3. Remove `ShadcnLook::from_theme` and `ShadcnLook::from_toml_str` — TOML-shaped themes are not an SDK concern.
4. Rewrite [`look.rs`](../crates/look-shadcn/src/look.rs) `ShadcnLook::native()` to load from embedded shadcn CSS (same tokens as gallery's default tweakcn theme) or compile-time shadcn defaults — **never** `LumaTheme::native()`.
5. Fix [`look-shadcn/src/controls/button.rs`](../crates/look-shadcn/src/controls/button.rs) tests that still call `LumaTheme::native()`.

### Delete `default-theme.toml` from the SDK

6. Delete [`default-theme.toml`](../crates/sdk/src/theme/default-theme.toml), `DEFAULT_THEME_TOML`, and `include_str!`.
7. Rewire defaults in [`tokens.rs`](../crates/sdk/src/theme/tokens.rs):
   - `ThemeTokens::default()` / `LumaThemeMode::default()` → `LumaThemeMode::light()` (already uses Rust constructors).
   - Remove `cached_native_light_mode()` / `cached_native_dark_mode()` that parse TOML via `LumaTheme::native()`.
   - `LumaTheme::native()` → delete, or make a thin alias over `ThemeModes { light: LumaThemeMode::light(), dark: LumaThemeMode::dark() }` if anything still needs the type temporarily.
8. Gallery / theme-studio `"Default"` theme preset → `ShadcnLook::from_css_path(...)` to a bundled CSS file, not `ShadcnLook::native()` through Luma TOML.

**Apps**

9. [`apps/theme-studio/src/studio/panels/chat.rs`](../apps/theme-studio/src/studio/panels/chat.rs) — replace `mode_tokens().palette.action(ShadcnButtonStyle::Primary)` with `ShadcnLook` resolver APIs.

---

## Phase 3 — Schema cleanup

### Delete `ActionPalette`

After Phase 1 and 2:

1. Delete `ActionPalette` and `ActionRolePalette` from [`crates/sdk/src/theme/tokens.rs`](../crates/sdk/src/theme/tokens.rs).
2. Remove `action` field from `LumaPalette`.
3. Remove re-exports from [`crates/sdk/src/theme/mod.rs`](../crates/sdk/src/theme/mod.rs).
4. Delete the entire `RawTheme` / `from_toml_str` deserializer pipeline from `tokens.rs` (including `RawActionPalette`, `RawMetricTokens`, etc.) — Phase 2 already removed the only consumer.

Do **not** replace action slots with `primary` / `secondary` in core `LumaPalette`.

### Purge `ColorTokens` shadcn-isms

[`ColorTokens`](../crates/sdk/src/theme/tokens.rs) is a second leak — `from_palette` derives `prominent*` and ghost-based `surface_*` from `ActionPalette`.

Options (pick one):

- **A (preferred):** Remove `ColorTokens` from `LumaThemeMode` if nothing in SDK controls reads `tokens.colors.*` at runtime.
- **B:** Retain `ColorTokens` but rename to structural names only (`surface`, `surface_hover`, `selected`, `text`, `border`, …) and derive from `state.*` / `surface.*` / `app.*`.

Either way, delete `prominent`, `prominent_hover`, and `prominent_pressed` fields.

### Public API note

Removing `ActionPalette`, `ActionRolePalette`, and possibly `ColorTokens` fields is a **breaking change** for any external crate that imported them from `gpui_luma::theme`. Document in changelog; acceptable for pre-1.0 SDK.

---

## Phase 4 — Metric and variant cleanup

### Simplify `MetricTokens`

[`MetricTokens`](../crates/sdk/src/theme/tokens.rs) currently duplicates size scales:

- `control.sm` / `control.md` / `control.lg` — used by `for_size()`
- Top-level `sm` / `md` / `lg` — redundant copy of the same heights (`28.0`, `36.0`, `44.0`)

**Steps:**

1. Remove top-level `sm`, `md`, `lg` from `MetricTokens` (and from any remaining raw deserializer types if Phase 3 hasn't landed yet).
2. Keep `ControlSize` enum and `control: ControlMetricScale` as the single SDK size ladder.
3. Update [`theme/cache.rs`](../crates/sdk/src/theme/cache.rs) — layout cache keys currently hash both `control.*` and top-level `sm/md/lg`.
4. Let `look-shadcn` templates map `ControlSize` → pixel heights from CSS catalog or local constants (e.g. shadcn `h-9` → 36px for medium).

Specific pixel heights are styling details; the SDK only needs relative size steps.

---

## Phase 5 — Docs, tests, and validation

### Documentation to update

| Doc | What to fix |
|---|---|
| [`docs/theme.md`](../theme.md) | Remove `action.*` token catalog; document structural palette only |
| [`docs/ai/module-map.md`](module-map.md) | Update palette / theme sections |
| [`docs/control-design.md`](../control-design.md) | Toggle segment guidance referencing `action.subtle` |
| [`docs/ai/fundamental-templates.md`](fundamental-templates.md) | Mark look-cleanup complete when acceptance criteria met |

### Tests and CI

After each phase:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p gpui-luma
cargo test -p gpui-luma-look-shadcn
just gallery    # smoke: native + CSS themes
just theme-studio
```

Update or delete SDK tests that assert on action role differentiation or parse `DEFAULT_THEME_TOML`.

### Inventory reference

Full grep targets when verifying completion:

```bash
rg 'palette\.action|ActionPalette|ActionRolePalette' crates/sdk
rg 'from_luma_tokens|LumaTheme::native|default-theme\.toml|from_toml_str' crates apps
rg 'TextFieldVariant::Ghost' crates apps
rg 'prominent' crates/sdk/src/theme/tokens.rs
```

---

## File index (quick reference)

| Area | Primary files |
|---|---|
| Palette schema | `crates/sdk/src/theme/tokens.rs`, `mod.rs` |
| Legacy theme file | `crates/sdk/src/theme/default-theme.toml` (delete; do not replace) |
| SDK unthemed defaults | `LumaThemeMode::light()` / `dark()` in `tokens.rs` |
| Product theme | `look-shadcn` CSS catalog (`ShadcnLook::from_css_path`) |
| Button family | `crates/sdk/src/controls/button_family/theme.rs` |
| Text field | `crates/sdk/src/controls/textfield/theme.rs`, `search_selector/control.rs` |
| Menus / lists | `popup_menu/theme.rs`, `selector/theme.rs`, `context_menu/theme.rs` |
| Toggles | `checkbox/theme.rs`, `switch/theme.rs`, `radio_button/theme.rs` |
| Other action users | `slider/theme.rs`, `progress/theme.rs`, `tabs_navigation/theme.rs`, `navigation_sidebar/theme.rs`, `resizable_panels/theme.rs` |
| look-shadcn | `palette.rs`, `mode.rs`, `look.rs`, `controls/*.rs` |
| Apps | `apps/gallery/.../textfield/pane.rs`, `apps/theme-studio/.../chat.rs` |
