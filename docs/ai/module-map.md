# GPUI-Luma Module Map

This map is intended as an onboarding index. It focuses on crate/module purpose, primary exports, and how modules relate.

**Apps (`apps/*`):** see `project-overview.md` § Apps using the SDK — compose SDK controls only; do not home-brew interactive widgets in app crates.

## Crate Map

## `crates/sdk` (`gpui-luma`)

- `lib.rs`
  - exports: `controls`, `focus`, `init`, `keyhandling`, `layout`, `macros`, `shell`, `theme`
  - re-exports: `init`, `DockPanel`
- `layout.rs`
  - lightweight WPF-style edge-docked composition primitive
  - `DockPanel::new().top(...).left(...).child(...).last_child_fill(true)` (`fill(...)` remains available as an alias for `child(...)`)
  - fills the parent and applies `min_w(0)` / `min_h(0)` remainder constraints for scrollable remainder content
- `macros.rs`
  - layout helpers: `dock_panel!`, `vstack!`, `hstack!`, `wrappanel!`, `flow!`
  - form helpers: `declare_form!`, `form_field!` (uses `controls::label::field_label` internally)
- `controls/label.rs`
  - `field_label(text, color)` — standard form field label styling

- `init.rs`
  - `init(cx: &mut App) -> anyhow::Result<()>`
  - registers Lucide icon font bytes with GPUI text system

- `focus.rs`
  - actions/context for focus traversal
  - `bind_default_focus_keys`
  - `LumaFocusScopeExt` for attaching focus behavior to interactive elements

- `keyhandling.rs`
  - control action set (`actions!`)
  - `ControlKeyProfile`
  - `default_control_key_bindings`, `bind_default_control_keys`

- `shell/mod.rs`, `shell/title_bar.rs`
  - `TitleBar` component + `TITLE_BAR_HEIGHT`
  - custom title-bar rendering and platform-specific window control behavior

- `theme/mod.rs`
  - submodules: `adorner`, `cache`, `interaction`, `layout`, `pack`, `tokens`, `registry`
  - re-exports token APIs, layout cache helpers, shared layout scales (no Radix symbols)

## `crates/look-shadcn` (`gpui-luma-look-shadcn`)

- `lib.rs`
  - `ShadcnLook`, `ShadcnButtonStyle`, `ShadcnModeTokens`, CSS catalog types
  - `ShadcnLookControlExt` and builder style helpers (`ShadcnButtonStyleExt`, …)
  - look-specific visual add-ons such as `Badge`, `BadgeVariant`, and `BadgeIconPlacement`
  - `all_shadcn_theme_usages`, per-control resolvers and template factories
- `stylesheet/`
  - `mod.rs`: Embedded styles compilation wrapper and lookup interface (`find_*_color_rule`).
  - `config.rs`: Strongly typed Serde deserialize layouts and matching engines for all themed controls and visual add-ons.
  - `selector.rs`: String selector mappings (`AsSelectorState`) for look states.
  - `resolve.rs`: Local variable interpolation (`@field`), px/rem parsing, and final color mappings.
- `ext.rs` — layout utility extensions (`bg_cn`, `text_cn`, `gap_cn`, …)
- `context.rs` — `with_look` thread-local scope for token resolution
- depends on `gpui-luma`; consumed by gallery and theme-studio alongside the SDK

## `apps/theme-studio` (`gpui-luma-theme-studio`)

- `main.rs`
  - app bootstrap; initializes SDK and opens Theme Studio window
- `app_shell.rs`
  - `open(cx)` for window creation and root entity mount
- `theme.rs`
  - `StudioThemeChoice` (CLI `default` or built-in shadcn theme id from `gpui-luma-look-shadcn`)
- `studio/app.rs`
  - `ThemeStudioApp` — single-board dashboard, size/mode toolbar, inspector overlay
- `studio/panels/*`
  - six shadcn-style demo cards (upgrade, account, team, chat, cookies, report)
- `studio/demo_controls.rs`
  - spawns panel entities from active `ShadcnLook`
- `studio/inspector.rs`
  - click-to-inspect settings sheet (scale metrics, token colors, export)
- `studio/export.rs`
  - small token-to-CSS export helpers used by the inspector and palette panels

Run: `just theme-studio` or `cargo run -p gpui-luma-theme-studio -- <theme-stem>`

## `apps/gallery` (`gpui-luma-gallery`)

- `main.rs`
  - app bootstrap; initializes SDK and opens main window
- `app_shell.rs`
  - `open(cx)` for window creation and root entity mount
- `assets/assets.rs`
  - custom `AssetSource` for embedded SVG labels
- `gallery/mod.rs`
  - exports `GalleryApp`
- `gallery/control.rs`
  - `GalleryApp` state + event wiring
- `gallery/template.rs`
  - render implementation for root gallery shell
- `gallery/theme.rs`
  - `GalleryThemeChoice` (CLI `default` or tweakcn CSS stem)
  - `tweakcn_dir()`, `available_theme_names()`, `shadcn_look()` loader
  - re-exports `LumaChrome` as `GalleryChrome`
- `gallery/panes/registry.rs`
  - page registry, nav model, pane constructor/subscription/dispatch
- `gallery/panes/dock_panel/*`
  - DockPanel + DockSplitter gallery/debug pane with resizable top/left/right/bottom boundaries
- `gallery/panes/*`
  - concrete demo panes per SDK control/feature

## Controls Subsystem Map (`crates/sdk/src/controls`)

## Shared infrastructure modules

- `template.rs`
  - `Modifier<M>`
  - `TemplateWithModifiers<M>`
  - `ControlTemplate<T, M>`
  - `define_control_template!` macro

- `state.rs`
  - `ControlFocusState`
  - `CompositeItemState`
  - `MenuPath`

- `value.rs`
  - `ControlRange`
  - range normalization/clamping/snapping helpers

- `motion.rs`
  - `MotionEasing`, `MotionSpec`
  - animation helpers for value transitions

- `presenter.rs`
  - `HostedContent`, `Presenter`, `IntoPresenter`
  - generic content/slot abstraction for controls

- `menu_item.rs`
  - `MenuItem`, `MenuItemIcon`
  - common menu tree item model
- `selector_panel/*`
  - selector-focused shared popup list primitives
  - exports selector-native item contracts (`SelectorItemLike`, `SelectorItemTemplate`, `SelectorPath`)
  - provides dedicated items-panel template contract (`SelectorItemsTemplate`) and default implementation
- `selection_panel/*`
  - first-class spawnable panel control for selector-family popup rows
  - owns row interaction state/eventing, active/hover/pressed lifecycle, keyboard navigation, and ensure-visible API
  - integrates `PopupScrollSurface` for viewport/scrollbar behavior
  - supports per-row customization via `SelectionPanelItemTemplate` / `with_item_template(...)`
  - exports builder/model/template/theme and `SelectionPanelEvent`

- `menu_navigation.rs` (crate-private)
  - keyboard navigation utilities over menu structures

- `popup_scroll_surface.rs`, `scroll_container.rs`
  - reusable scroll mechanics, viewport bounds, and scrollbar syncing

- `interaction.rs` (crate-private)
  - reusable pointer/focus interaction state helpers

## Command/button family

- `command/core.rs`
  - `CommandCore`, `CommandEvent`
- `command/button/*`
  - `Button<D>`, `ButtonBuilder<D>`, `ButtonModel`, `ButtonRenderModel`
  - `ButtonEvent`
  - template types (`ButtonTemplate`, `DefaultButtonTemplate`)
- `command/icon_button/mod.rs`
  - `IconButton` convenience constructor
- `button_family/*`
  - button variants, role resolution, appearance/theme traits
  - shared button theming across standard/ghost/prominent/toggle styles

## Choice controls

- `checkbox/*`
  - bool button wrapper with checkbox-specific template/theme
- `radio_button/*`
  - bool button wrapper with radio-specific template/theme
- `switch/*`
  - bool button wrapper with switch template/theme + motion aliases
- `toggle/mod.rs`
  - bool button convenience constructor using toggle template defaults
- `control_group/*`
  - generic composite selection engine with themed list chrome (`theme.rs`, `themed_template.rs`)
  - `button_item_template` bridges items to `ButtonTemplate`
  - owns selection semantics, active-item navigation, and managed/unmanaged selection state
- `button_group/*`
  - semantic wrapper over `control_group` for horizontal icon toolbars (`icon_toolbar`, `horizontal`)
- `radio_group/*`
  - semantic wrapper over `control_group` for radio-style single selection
  - exposes radio-oriented constructor names/types while delegating behavior to `control_group`

## Text controls

- `textfield/*`
  - single-line text input
  - supports `TextFieldVariant` (`Standard`, `Ghost`) via builder `.variant(...)`
  - supports per-instance visual specialization via builder `.appearance_override(...)`
    - closure shape: `Fn(TextFieldAppearance) -> TextFieldAppearance + Send + Sync + 'static`
    - applied after theme/template appearance resolution and before template render + text layout/shaping usage
  - theme resolution is variant-aware and palette-first (`TextFieldTheme::resolve(variant, state, enabled)`), while composed `TextFieldAppearance` is rebuilt from cached `StandardBoxScale`
  - exports builder/model/state/template/theme + `TextFieldEvent`
- `textarea/*`
  - multiline text editor with scroll + drag behavior
  - theme resolution is palette-first (`TextAreaTheme::resolve(state, enabled)`), while render/layout paths compose cached `StandardBoxScale`
  - exports builder/model/state/template/theme + `TextAreaEvent`
- `text/*` (crate-private)
  - shared editing/state primitives used by textfield/textarea

- `dock_splitter/*`
  - lightweight drag-to-resize separator control for use between docked edges and fill regions
  - builder/theme/template/control split with delta-based resize callbacks and events

## Selection controls

- `autocomplete/*`
  - text-based suggestion control
  - exports builder/model/template/theme + `AutocompleteTextBoxEvent`
- `combobox/*`
  - selectable input with typing policy and popup list
  - exports builder/model/template + `ComboBoxEvent`, `TypingPolicy`
- `search_selector/*`
  - read-only trigger selector with popup-hosted search field and filtered list
  - layered template composition:
    - control template (`SearchSelectorTemplate`)
    - panel template (`SearchSelectorPanelTemplate`)
    - item template (`SearchSelectorItemTemplate`)
  - builder/runtime supports `panel_template(...)` and `with_item_template(...)`
  - transitional `items_template(...)` hook remains for compatibility
  - exports builder/model/template + `SearchSelectorEvent`
- `selector/*`
  - popup selector (typed or built-in item model)
  - two-template composition: control template (`SelectorTemplate`) + items template (`SelectorItemsTemplate` via `selector_panel`)
  - trigger visuals resolve through `SelectorPalette`, with trigger geometry composed from cached `StandardBoxScale`
  - exports builder/model/template/theme + `SelectorEvent`
- `selector_panel/*`
  - selector-native item model and popup row renderer shared by selector-family controls
  - exports `SelectorItem`, `SelectorItemLike`, `SelectorItemIcon`, `render_selector_items_popup`

## Menu controls

- `floating_menu/*`
  - generic floating/nested menu state + themed rendering helpers
- `popup_menu/*`
  - trigger + floating menu composition
  - trigger visuals resolve through `PopupMenuPalette`, with trigger geometry composed from cached `StandardBoxScale`
  - exports builder/model/template/theme + `PopupMenuEvent`
- `context_menu/*`
  - context-triggered menu composition
  - exports builder/model/template/theme + `ContextMenuEvent`

## Navigation/layout/feedback controls

- `navigation_sidebar/*`
  - tree navigation surface with collapsible/rail behavior
  - exports nav node model and `NavigationSidebarEvent`
- `tabs_navigation/*`
  - tab-list navigation and activation events
- `accordion/*`
  - collapsible section groups with single/multiple expansion modes
  - exports builder/model/template/theme + `AccordionEvent`
- `listbox/*`
  - selector-style static list panel with single/multiple selection
  - thin wrapper over `control_group` with listbox-specific templates and theme
  - row visuals resolve through `ListBoxRowPalette`, with row geometry composed from cached `ListRowScale`
  - exports `ListBox`, `ListBoxItem`, constructors, and re-exports `ControlGroupEvent`
- `list_view/*`
  - virtualized list control backed by `gpui::list` / `ListState`
  - owns active-row navigation, selection state, fixed-header slot, and row virtualization
  - supports plain typed item models via stored label/enabled adapters instead of a required item trait
  - row visuals resolve through `ListViewRowPalette`, with row geometry composed from cached `ListRowScale`
  - scroll modes: `ListScrollMode` (`ScrollSmooth`, `ScrollSnap`, `Paged { page_size }`), optional `visible_rows` shell sizing
  - paging facade: `PagingListViewControl` + `PagingListViewBuilder` compose `ListViewControl` with SDK `PagingToolbar` (selection/page sync under the hood)
  - exports `ListViewControl`, `ScrollingListView`, `PagingListView`, builders, `PagingToolbar*`, `ListViewLabel`, `ListSelectionMode`, `ListScrollMode`, `new` / `new_typed`, declarative `list_view!` / `scrolling_list_view!` / `paging_list_view!` / `column!` macros, grid-column helpers, and theme/template hooks
  - roadmap and scope (not a data grid): `docs/retired/listview-vnext.md`; phase-2 paging/scroll/sizing: `docs/retired/listview_2.md`; facade re-unification: `docs/retired/listview_3.md`
- `split_view/*`
  - resizable/collapsible split pane shell + `SplitViewEvent`
  - `SplitViewTheme` resolves separator colors at render time; `set_theme` for hot-swaps
- `resizable_panels/*`
  - multi-panel groups (horizontal/vertical, nested via composition) with mixed `PanelSize::{Absolute,Weight}` sizing + `ResizablePanelsEvent`
  - overlay resize handles (no layout width); `ResizeHandleSize::{Sm,Md,Lg}` via `.resize_handle()` (lane, grip, hit scale together); `ResizablePanelSpec::bg(Hsla)` supplies panel/handle-half colors
  - runtime state: `PanelLayoutState` (px for absolute, weight coefficient for fill panes); legacy weight-only specs still supported via deprecated percent APIs
  - design spec: `docs/ai/fix-resize-panel.md`
- `color/*`
  - migrated color-control subsystem from `gpui-opal`
  - primitives: `color_field`, `color_slider`, `color_ring`, `color_arc`
  - shared helpers: `mouse_behavior`, `shape`, `style`
  - current surface remains state-centric and self-rendering; gallery uses it directly while future builder/template/theme reshaping can happen incrementally
- `slider/*`
  - ranged value control + drag semantics + `SliderEvent`
- `scrollbar/*`
  - scrollbar model/control + drag and change events
- `progress/*`
  - progress indicator model/template/theme

## Theme Subsystem Map (`crates/sdk/src/theme`)

- `tokens.rs`
  - core theme schema (palette, metrics, typography, elevation)
  - action role palettes include explicit `border` tokens (e.g. `action.ghost.border`)
  - mode support (`ThemeMode`, `ThemeModes`)
  - TOML parsing and conversion into typed theme model

- `layout.rs`
  - shared density/layout math utilities
  - `snap_to_pixel`, `StandardBoxScale`, `GlyphIndicatorScale`, `ListRowScale`

- `cache.rs`
  - app-global layout cache keyed by control size, display scale, and metric signature
  - `LayoutCacheKey`, `LumaLayoutCacheExt::use_cached_layout(...)`

- `pack.rs`
  - `LumaChrome`: app/gallery shell chrome colors

- `registry.rs`
  - palette token introspection (`PaletteColorToken`)
  - lookup helper `resolve_palette_color`
  - metadata types `ThemeUsage`, `ThemePartUsage`

- `interaction.rs`
  - `InteractionState` + layer precedence enum

- `adorner.rs`
  - adorner definitions for focus ring placement/specification

## Gallery Pane Map (`apps/gallery/src/gallery/panes`)

- Core wiring:
  - `mod.rs` – pane module declarations
  - `registry.rs` – page constants/groups, nav construction, pane ownership/subscription/render dispatch
  - `shared/mod.rs` – shared pane rendering helpers

- Pane categories:
  - intro/meta: `introduction`, `search`, `settings`, `palette`, `theme_usage`
  - command: `button`, `icon_button`, `prototypes/*`
  - choice: `toggle`, `toggle_group`, `switch`, `checkbox`, `accordion`, `radio_button`, `radio_group`, `listbox`, `choice_controls_template`
  - color: `color/*` for primitive demos plus composition pages (`Combinations`, `Color Picker`, `HSV Plane`, `HSV Wheel`, `SV Triangle`, `Multi Mixer`, `Split Ring`)
  - input: `textfield`, `textarea`, `slider`, `scrollbar`
  - menu/selection: `floating_menu`, `popup_menu`, `context_menu`, `autocomplete`, `combobox`, `search_selector`, `selector`, `selection_panel`
  - navigation/feedback: `navigation_sidebar`, `tabs_navigation`, `progress`

## Public API Landmarks (Fast Lookup)

- SDK root exports: `crates/sdk/src/lib.rs`
- Controls exports: `crates/sdk/src/controls/mod.rs` + each control `mod.rs`
- Theme exports: `crates/sdk/src/theme/mod.rs`
- Gallery registry/routing: `apps/gallery/src/gallery/panes/registry.rs`

## Testing Footprint

- Tests are concentrated in SDK modules (`#[cfg(test)]` unit tests).
- Commonly tested areas:
  - keybinding tables
  - range math
  - text editing engine
  - textarea/scrollbar state logic
  - menu/choice navigation
  - template placement rules
  - theme registry and token validity
- Gallery app has no dedicated test modules at this time.

## Risk Hotspots by Module Area

- `theme/tokens.rs`: parsing schema drift and default-theme invariant
- `controls/textarea/control.rs`, `controls/textfield/control.rs`: complex interaction state + async cursor/selection tasks
- `controls/navigation_sidebar/control.rs`: large state transitions and nested nav behavior
- `crates/look-shadcn/src/usage.rs`: manual Shadcn usage registration maintenance
- `apps/gallery/src/gallery/panes/registry.rs`: string-ID routing and broad wiring surface
