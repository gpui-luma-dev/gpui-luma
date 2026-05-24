# Next step: active theme wiring

**Scope:** Runtime connection between the app’s **`LumaThemePack`** (light/dark mode) and SDK **default templates** — so gallery stock examples do not need `.template(theme.*_template())`.

**Not in scope here:** Which `theme.toml` is embedded, palette content, shadcn import, new button kinds, or theme file hot-reload. Those are separate theme-*authoring* tasks. This doc is only **active theme resolution**.

**Status:** SDK incomplete — gallery may pass pack-built templates as a temporary workaround until defaults read the active pack.

---

## Problem

Gallery stock controls were updated to pass templates from `GalleryThemePack`:

```rust
Button::new("intro-submit")
    .template(theme.button_template())  // workaround — should not be required for stock UI
    .spawn(cx);
```

Without that, stock controls use SDK defaults that **ignore the app’s active mode**. The gallery starts in dark mode (`control.rs`: `theme.set_mode(ThemeMode::Dark)`), but defaults still resolve a **light token snapshot**.

Symptoms seen in the gallery:

- Introduction payment Submit/Cancel buttons look light while the template matrix row is correct.
- Text fields, switches, sliders, and progress rings in intro panels mismatch their dedicated panes.
- State preview matrices (which often use `theme.button_family_theme()` or pack templates) look correct while top-of-pane demo controls do not.

### Root cause (active theme only)

```text
App owns LumaThemePack (mode = Dark)
       ✗ not connected
SDK default_*_template() → static default_*_theme() → ThemeTokens::default()  (= light snapshot)
```

| Piece | Issue |
|---|---|
| `LumaThemePack` + `LumaLiveTheme` | **Works** — `resolve()` reads current mode from pack |
| `theme.button_template()` etc. | Wraps `LumaLiveTheme`; gallery uses these explicitly |
| `default_button_template()` | Uses `default_button_family_theme()` — **no pack** |
| `default_*_theme()` helpers | `OnceLock` singletons holding `Default*Theme { tokens: ThemeTokens::default() }` |
| `gpui_luma::init(cx)` | Does not register an active pack |

Templates themselves are fine: **mode-agnostic structure**, colors from theme at resolve time. The gap is **defaults not pointing at the live pack**.

### What is *not* broken

- **Templates are mode-agnostic** in the right sense: structure, modifiers, layout — not baked light/dark colors.
- **`LumaLiveTheme`** is mode-aware: `resolve()` calls `state.tokens()` for the current `ThemeMode`.
- **Custom templates** (radial context menu, prototype modifiers, sidebar leaf templates) should stay explicit `.template(...)`.

The bug is **default template → frozen light tokens**, not the template/theme split.

---

## Target behavior

```text
App registers LumaThemePack once
  → Button::new(...).spawn(cx)   // no .template(...) for stock controls
       → default template → live theme → current light/dark tokens
  → theme.toggle_mode() + notify → stock controls update
  → .template(custom) only for bespoke gallery/app presentation
```

Rules:

1. **Default** — control uses SDK default template; default reads **active pack** when registered.
2. **Override** — explicit `.template(...)` for radial menu, prototypes, matrix wrappers, modifiers.
3. **Mode toggle** — no per-control template injection; no re-spawn for stock controls.
4. **Gallery examples** — rely on SDK `default_*_template()` (or omit `.template()` where the builder already defaults). Do **not** route every stock control through `theme.*_template()`.

### Important distinction (gallery cleanup)

When removing workarounds, **do not** remove all `.template(...)` usage.

| Remove | Keep |
|---|---|
| `.template(theme.button_template())` on stock buttons | `modified_button_template(theme)` |
| `.template(theme.textfield_template())` on stock fields | Custom textfield appearance overrides in gallery |
| `.template(theme.checkbox_template())` on stock checkboxes | Template matrix panes that intentionally render a cached template |
| Pack template on navigation sidebar / combobox nested defaults | `context_menu/radial.rs`, `registry.rs` sidebar templates |
| Passing `theme.*_template()` where SDK default is intended | `.with_modifier(...)` built on pack theme or custom theme |

State preview entities (`*StatePreview`) that store a template for offline matrix rendering should switch to **`default_*_template()`** (not `theme.*_template()`) once the SDK fix lands — they are showing default templates, not pack-specific factories.

---

## Proposed fix (active theme only)

### Phase 1 — Register active pack at app startup

Gallery (or any app) sets the pack once before spawning controls, e.g.:

```rust
let theme = GalleryThemePack::new();
theme.set_mode(ThemeMode::Dark);
gpui_luma::theme::set_active_theme_pack(&theme);
```

Add `set_active_theme_pack(pack: &LumaThemePack)` (or extend `gpui_luma::init`) in the SDK. **No change to embedded TOML or theme file choice required for this step.**

Call order matters: register the pack **before** the first `default_*_template()` / `default_*_theme()` access, or `OnceLock`-cached defaults will freeze the static fallback.

### Phase 2 — Wire SDK defaults to active pack

Change `default_*_template()` / `default_*_theme()` to resolve through the same **`LumaLiveTheme`** path as `LumaThemePack::button_template()` when a pack is registered.

Sketch:

```rust
pub fn default_button_family_theme() -> Arc<dyn ButtonFamilyTheme> {
    if let Some(live) = active_live_theme() {
        return live; // Arc<LumaLiveTheme> as Arc<dyn ButtonFamilyTheme>
    }
    // tests / bare snippets: static OnceLock fallback
}
```

Apply the same pattern across ~17 `default_*_theme()` sites (button family, checkbox, switch, radio, slider, scrollbar, textfield, textarea, popup menu, selector, navigation sidebar, context menu, progress, tabs, listbox, control group, floating menu, autocomplete).

Controls keep calling `default_button_template()` internally; apps do not pass the pack into every spawn.

### Phase 3 — Composite controls

When omitted, nested defaults should also use the active pack:

- `ComboBox::textfield_template` / `scrollbar_template`
- `NavigationSidebar::scrollbar_template`
- `ThemedNavigationSidebarTemplate` floating menu theme
- `SelectionPanelControl` scrollbar + appearance provider

Same mechanism as Phase 2 — not separate theme authoring.

### Phase 4 — Gallery cleanup (default template reuse)

Remove redundant **`theme.*_template()`** passes where the stock SDK template is intended. Prefer:

- omit `.template(...)` when the control builder already installs the default, or
- `default_*_template()` when a pane needs an `Arc<dyn …Template>` reference (state previews, template pipeline baselines).

**Keep** explicit `.template(...)` for custom gallery templates (see inventory below).

### Phase 5 — Trim `notify_controls`

After Phases 1–2, mode toggle should need `pack.toggle_mode()` + view/entity notify. Large `notify_controls` cascades exist partly because every pane had to refresh pack-wired entities; re-audit after defaults are live. Preview structs that **cache** templates may still need explicit notify.

---

## Gallery inventory

Search:

```bash
rg '\.template\(theme\.' apps/gallery
rg 'theme\.\w+_template\(\)' apps/gallery
```

### Workaround sites (remove pack templates after SDK fix)

| Area | Typical pack templates |
|---|---|
| `introduction/payment_panel.rs` | button, textfield, checkbox, radio; combobox textfield + scrollbar |
| `introduction/system_panel.rs` | checkbox, switch, slider, progress |
| `introduction/workspace_panel.rs` | popup menu, toggle, radio (via group item templates) |
| `control.rs` | navigation sidebar, scrollbar |
| `button/pane.rs`, `icon_button/pane.rs` | `button_template` on stock demo buttons |
| `checkbox/`, `switch/`, `radio_button/`, `toggle/`, `slider/`, `progress/`, `scrollbar/` | matching `*_template` on stock spawns |
| `selector/pane.rs` | `selector_template` |
| `popup_menu/pane.rs` | `popup_menu_template` |
| `tabs_navigation/pane.rs` | stock tabs template (keep `local_tabs_navigation_template`) |
| `textfield/`, `textarea/` | textfield/textarea/button/checkbox on stock spawns |
| `toggle_group/pane.rs` | `control_group_theme`, `toggle_template` for item templates |
| `radio_group/pane.rs` | base `radio_button_template` passed into layout wrappers |
| `navigation_sidebar/pane.rs` | navigation + scrollbar |
| `choice_controls_template/`, `selector_controls_template/` | pack templates used as **default column** baselines → switch to `default_*_template()` |
| `prototypes/mod_button/pane.rs`, `prototypes/proto_button/pane.rs` | `button_template` on **stock** buttons only |

### Intentional custom templates (keep)

| Path | Reason |
|---|---|
| `context_menu/radial.rs` | Custom radial layout |
| `context_menu/template.rs` | Gallery wrapper around context menu theme |
| `prototypes/mod_button/pane.rs` | `modified_button_template` |
| `prototypes/proto_button/pane.rs` | demo modifiers |
| `button/pane.rs` | Matrix / uniform-width template wrappers |
| `registry.rs` | Sidebar leaf/disclosure templates |
| `radio_group/pane.rs` | `delivery_window_template`, vertical/horizontal/indented wrappers |
| `introduction/workspace_panel.rs` | `density_template` |
| `tabs_navigation/pane.rs` | `local_tabs_navigation_template` |

### State preview structs

These cache a template for matrix rendering — after SDK fix, prefer `default_*_template()` over `theme.*_template()`:

- `*StatePreview` in checkbox, switch, radio, toggle, slider, progress, scrollbar, popup menu, textfield, tabs navigation, button, icon button panes
- `selector/preview.rs`, `choice_controls_template/pane.rs`, `selector_controls_template/pane.rs`

---

## Verification checklist

After SDK Phases 1–2:

1. Gallery dark mode: `Button::new(...).spawn(cx)` with **no** `.template(theme.*)` on Introduction payment buttons — colors match pack.
2. Title bar light/dark toggle updates stock buttons, text fields, switches without re-spawn.
3. Template matrix rows still match top-of-pane stock demos (Button, Icon Button, Checkbox, etc.).
4. Custom templates (radial menu, proto demo buttons) unchanged when explicitly passed.
5. `cargo test -p gpui-luma` passes; tests without registered pack behave deterministically (document fallback).

Manual spot-check:

- Introduction → Payment / System / Workspace
- Button, Switch, Selector, Popup Menu, Slider
- Prototypes → Custom / Decorated buttons (custom templates only)

---

## Related code

| Topic | Path |
|---|---|
| Active pack + live resolve | `crates/sdk/src/theme/pack.rs` |
| Static default button theme | `crates/sdk/src/controls/button_family/theme.rs` |
| Default button template | `crates/sdk/src/controls/command/button/template.rs`, `model.rs` |
| Other `default_*_theme()` | `crates/sdk/src/controls/*/theme.rs` |
| Gallery pack + mode | `apps/gallery/src/gallery/control.rs`, `template.rs` |
| Gallery notify cascade | `apps/gallery/src/gallery/panes/registry.rs` |
| Control/theming design | `docs/control-design.md`, `docs/theme.md` |

---

## Checking in gallery workaround PRs

It is **OK to merge** gallery changes that add `.template(theme.*_template())` and `notify_controls` **until** Phases 1–2 land. Those changes:

- fix visible dark-mode bugs today,
- do not block the SDK fix,
- should be **removed incrementally** once defaults are wired (use inventory above).

When reviewing PRs:

- Flag new gallery stock spawns that **require** pack templates — temporary until SDK fix.
- Do **not** flag intentional custom `.template(...)` (prototypes, radial menu, matrix wrappers).
- After SDK fix, flag new `theme.*_template()` on stock controls — prefer SDK defaults.

---

## Explicitly elsewhere

Theme TOML authoring, tweakcn import, palette schema (`action.subtle`), and swapping `DEFAULT_THEME_TOML` — see `docs/ai/next-step-theme-import.md` and `docs/ai/next-step-variants.md`.
