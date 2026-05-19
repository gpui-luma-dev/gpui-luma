# GPUI-Luma Architecture

## Architectural Goals

The project is structured around two complementary concerns:

1. **SDK crate (`gpui-luma`)**: provide reusable, themeable GPUI controls with consistent keyboard/focus behavior.
2. **Gallery app (`gpui-luma-gallery`)**: provide a living showcase and integration surface for SDK controls.

## Workspace Architecture

- Workspace uses Cargo resolver v2 with shared dependency versions.
- `crates/sdk` is the primary product surface.
- `apps/gallery` is a consumer app that validates ergonomics and behavior.

## SDK Architecture (`crates/sdk`)

## Layering

- **Initialization and input behavior**
  - `init.rs`: one-time UI runtime setup (fonts)
  - `focus.rs`: focus traversal actions/context
  - `keyhandling.rs`: control key profiles + default key bindings

- **Theming subsystem**
  - `theme/tokens.rs`: theme schema + parsing + defaults + mode selection
  - `theme/pack.rs`: runtime theme pack (`LumaThemePack`) with light/dark toggling and live trait-object providers
  - `theme/registry.rs`: palette token metadata/introspection API
  - `theme/interaction.rs`: generic interaction-state layer precedence
  - `theme/adorner.rs`: adorner/focus-ring descriptors (current policy: one optional adorner per appearance)

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
- `theme.rs` – appearance struct, theme trait, default theme, theme usage metadata
- `mod.rs` – curated public API exports and convenience constructors

This pattern improves consistency and makes style/theming separable from behavior.

## Eventing and State Management

- GPUI entity model is used throughout:
  - controls are typically spawned via builder `.spawn(cx)`
  - control events are emitted via `EventEmitter`
  - consumers subscribe using `cx.subscribe(...)`
- Re-rendering is explicit with `cx.notify()`.
- Shared state abstractions (`ControlFocusState`, `CompositeItemState`, `InteractionState`) normalize interaction semantics across controls.

## Template and Theme Composition

- Templates are trait-object based (`Arc<dyn ...Template>`).
- Many templates accept handler bundles (`...TemplateHandlers`) to bridge control logic to rendering hooks.
- `ControlTemplate<T, M>` plus modifier pipelines provide composability.
- TextField supports per-instance appearance specialization (`.appearance_override(...)`) without adding global variants/tokens; overrides are applied after appearance resolution and used consistently by both template rendering and text layout/shaping.
- `LumaThemePack` serves as runtime adapter:
  - stores theme + active mode
  - provides pre-wired template/theme implementations for controls
  - supports live mode toggling (light/dark)
- `control_group` is intentionally theme-agnostic:
  - no `theme.rs`
  - exposes container + item template hooks for semantic wrappers to style explicitly
  - owns selection semantics (`SingleRequired`, `SingleAllowNone`, `Multiple`) and managed/unmanaged selection state

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
- Theme mode toggle in title bar updates `LumaThemePack` and triggers notify.

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

- Manual registries (`theme_registry`, gallery page registry) can drift.
- String-based IDs in gallery routing are typo-prone.
- Several large control modules centralize complex state logic, increasing regression surface.
- Runtime visuals rely on successful icon font initialization.
- Platform-specific title-bar behavior is nuanced and can regress by OS.
