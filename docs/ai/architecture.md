# GPUI-Luma Architecture

## Architectural Goals

The project is structured around two complementary concerns:

1. **SDK crate (`gpui-luma`)**: provide reusable, themeable GPUI controls with consistent keyboard/focus behavior.
2. **Apps under `apps/`**: product shells and showcases that **only compose** SDK controls—they do not define parallel button/checkbox/input implementations.

## Workspace Architecture

- Workspace uses Cargo resolver v2 with shared dependency versions.
- `crates/sdk` is the primary product surface.
- `apps/*` are consumers; each app validates integration but shares the same control vocabulary.

## Consumer apps (`apps/*`)

Apps mount GPUI trees built from `gpui_luma::controls` and `gpui_luma::theme`, typically with a runtime `gpui_luma_look_shadcn::ShadcnLook` (or native fallback via `ShadcnLook::native()`).

**Invariant:** interactive UI lives in the SDK. App code wires models, layout, and subscriptions—it does not home-brew themed controls (hand-styled `div` click targets, inline checkmarks, ad-hoc disabled states, etc.). Missing capability is a gap in `crates/sdk`, not a license to fork visuals in an app.

**Allowed in apps:** page layout, static text, domain models, and thin glue (e.g. list column templates that *embed* `Entity<Checkbox>`, not reimplement checkbox paint).

**Not allowed in apps:** duplicate control semantics or appearance that bypass `*Builder`, templates, and theme resolvers.

Per-app structure (gallery registry, theme-studio panels, etc.) is documented in `module-map.md`; this rule applies to all current and future `apps/` members.

## SDK Architecture (`crates/sdk`)

## Layering

- **Initialization and input behavior**
  - `init.rs`: one-time UI runtime setup (fonts)
  - `focus.rs`: focus traversal actions/context
  - `keyhandling.rs`: control key profiles + default key bindings

- **Theming subsystem (`gpui-luma`)**
  - `theme/tokens.rs`: native theme schema + parsing + defaults + mode selection (`LumaTheme::native()` fallback)
  - `theme/pack.rs`: `LumaChrome` shell colors only (historical module name)
  - `theme/registry.rs`: palette token metadata/introspection API
  - `theme/interaction.rs`: generic interaction-state layer precedence
  - `theme/adorner.rs`: adorner/focus-ring descriptors (current policy: one optional adorner per appearance)

- **Shadcn look crate (`gpui-luma-look-shadcn`, `crates/look-shadcn`)**
  - CSS-first product theming — `ShadcnLook`, CSS catalog parse, control appearance resolvers, template factories
  - `usage.rs`: hand-maintained usage metadata (`all_shadcn_theme_usages`)
  - extension traits: `ShadcnLookControlExt`, `ShadcnButtonStyleExt`, etc.
  - depends on `gpui-luma`; apps depend on both crates

- **Controls subsystem** (`controls/`)
  - shared infra (`template`, `state`, `value`, `motion`, `menu_item`, `menu_navigation`, `presenter`)
  - generic selection engine: `control_group` (lookless composite selection/focus primitive)
  - concrete controls (buttons, text inputs, menu controls, nav controls, sliders/scrollbars, etc.)
  - each control module exports builder/event/template/theme types

- **Shell subsystem**
  - `shell/title_bar.rs`: reusable custom title bar with platform-specific behavior

## Control Module Pattern

Most concrete control modules follow a consistent split:

- `model.rs` – static model + builder + render model payloads
- `control.rs` – runtime behavior, events, interaction logic, GPUI entity wiring
- `template.rs` – render contract (`trait ...Template`) and handler callbacks
- `theme.rs` – control visual contracts, theme trait, default theme, theme usage metadata
- `mod.rs` – curated public API exports and convenience constructors

This pattern improves consistency and makes style/theming separable from behavior.

## Eventing and State Management

- GPUI entity model is used throughout:
  - controls are typically spawned via builder `.spawn(cx)`
  - control events are emitted via `EventEmitter`
  - consumers subscribe using `cx.subscribe(...)`
- Re-rendering is explicit with `cx.notify()`.
- Parent/child entity trees in apps must avoid reading or updating a leased entity from a nested render or subscription; see `docs/ai/reentrant-warning.md` (theme-studio `BoardSnapshot` pattern).
- Shared state abstractions (`ControlFocusState`, `CompositeItemState`, `InteractionState`) normalize interaction semantics across controls.

## Template and Theme Composition

- Templates are trait-object based (`Arc<dyn ...Template>`).
- Many templates accept handler bundles (`...TemplateHandlers`) to bridge control logic to rendering hooks.
- `ControlTemplate<T, M>` plus modifier pipelines provide composability.
- Phase-5 density refactoring is now active on `switch`, `checkbox`, `button_family`, `radio_button`, `textfield`, `textarea`, `selector`, `popup_menu`, `list_view`, and `listbox`: those theme traits now resolve palette-only structs for control visuals, while templates or control layout paths compose cached shared layout scales from `MetricTokens` + `window.scale_factor()`.
- TextField supports per-instance appearance specialization (`.appearance_override(...)`) without adding global variants/tokens; overrides are applied after appearance resolution and used consistently by both template rendering and text layout/shaping.
- `ShadcnLook` serves as the product runtime theme:
  - Loads CSS catalogs (`ShadcnLook::from_css_path`) and parses/holds the embedded `style.toml` stylesheet configuration (`ShadcnLook::stylesheet()`), or loads both custom CSS and TOML dynamically at runtime.
  - Keeps one shared live look identity per app while allowing the underlying catalog, mode token snapshots, and stylesheet-backed overrides to be replaced in place at runtime.
  - Exposes control template and theme factories powered by the dynamic stylesheet engine.
  - Apps keep one `Arc<ShadcnLook>` in app state and pass it explicitly into control/theme factory helpers.
  - SDK `default_*_theme()` helpers remain native-token defaults unless a caller opts into look-shadcn template/theme factories.
- `control_group` provides optional themed list chrome via `ControlGroupTheme` + `ThemedControlGroupTemplate` (`ControlTemplate` + modifiers from `controls/template.rs`):
  - group border/background/radius/padding from `border.default` and `surface.subtle.background`
  - item visuals still come from `ControlGroupItemTemplate` (e.g. `button_item_template` + `ButtonTemplate`)
  - owns selection semantics (`SingleRequired`, `SingleAllowNone`, `Multiple`) and managed/unmanaged selection state
- `button_group` is a semantic wrapper presetting horizontal icon-toolbar layout + rounded group modifier

## Error-Handling Architecture

- Setup/parsing uses `anyhow::Result` and contextual errors.
- Theme parsing converts untyped TOML into strongly typed theme structures.
- Startup path in gallery is fail-fast for window initialization, with error printout.
- Intentional invariant panic exists in `LumaTheme::native()` (`expect` on embedded default theme parsing).

## Runtime and Async Model

- Primarily synchronous event-driven UI operations via GPUI.
- Async work is control-local and lightweight (`cx.spawn(...)` tasks), e.g. caret blinking and selection auto-scroll.
- No explicit Tokio/async-std requirement in crate APIs.
- Cross-platform shell behavior includes explicit branches for macOS, Linux, and Windows.

## Gallery Architecture (`apps/gallery`)

## Theme loading

```
apps/gallery/tweakcn/<stem>.css
        ↓
GalleryThemeChoice (CLI: default or CSS stem)
        ↓
ShadcnLook::from_css_path / ShadcnLook::native()
        ↓
GalleryApp state (Arc<ShadcnLook>)
        ↓
GalleryPanes + per-pane render(&ShadcnLook)
```

## Main components

- `main.rs` – application setup + SDK bootstrap + window open
- `app_shell.rs` – window options and root entity creation
- `gallery/control.rs` – `GalleryApp` state and cross-component event coordination
- `gallery/template.rs` – root render tree
- `gallery/panes/registry.rs` – central page registry, nav tree construction, pane dispatch
- `gallery/panes/*` – per-control demo panes

## Navigation and pane composition

- Registry defines static page IDs, labels, icons, groups, and page kind enum.
- Navigation sidebar and route buttons are generated from registry metadata.
- Selection is string-ID based and mapped to pane render functions.
- Unknown IDs render an explicit fallback pane.

## Gallery event flow

- `SplitView` and `NavigationSidebar` are synchronized bidirectionally (collapsed state + width updates).
- Pane entities subscribe to events in centralized `GalleryPanes::subscribe`.
- Theme mode toggle in title bar updates `ShadcnLook` mode and triggers notify.

## Public API Surface (Most Important)

- SDK root: `gpui_luma::init`
- Controls root: `gpui_luma::controls::*`
- Theme root: `gpui_luma::theme::*`
- Focus/input helpers: `gpui_luma::focus::*`, `gpui_luma::keyhandling::*`
- Shell: `gpui_luma::shell::TitleBar`

## Test Architecture

- SDK contains broad unit-test coverage focused on deterministic behavior and state transitions.
- Theming has integrity tests for token registry/usage metadata.
- Gallery is currently manual-test oriented (no dedicated automated tests).

## Known Architectural Risks

- Manual registries (`theme/radix/usage.rs`, gallery page registry) can drift.
- String-based IDs in gallery routing are typo-prone.
- Several large control modules centralize complex state logic, increasing regression surface.
- Runtime visuals rely on successful icon font initialization.
- Platform-specific title-bar behavior is nuanced and can regress by OS.
