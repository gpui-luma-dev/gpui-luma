# Issue #8: Create Toolbar Control

## Description
We need a cohesive Toolbar SDK control that arranges command buttons, toggle buttons, menu triggers, selector-style controls, and separators horizontally. Apps currently have to compose these manually with raw flex rows, which makes keyboard behavior, spacing, separators, and theme variants inconsistent.

The toolbar must not create a second focus/navigation system. It should reuse the existing `control_group` infrastructure for item modeling, active item handling, roving focus, arrow-key ownership, item state, layout direction, and template seams.

This issue should be implemented after the `control_group` focus-strategy work. Toolbar should opt into `ControlGroupFocusStrategy::RovingItemFocus` and supply `ControlGroupFocusTarget`s for hosted controls that have their own GPUI focus handles.

## Proposed Solution
Create a new `Toolbar` SDK control as a toolbar-specific specialization around `control_group`. The toolbar should not predefine the concrete controls it can host. A proper toolbar is a composite row of generic hosted items plus separators; callers must be able to host buttons, toggles, menus, selectors, text fields, color pickers, or future controls without the toolbar API adding a new enum variant for each one.

Required architectural direction:
- `ToolbarItem` should implement `ControlGroupItemLike` or be adapted into a `ControlGroupItemLike` model.
- `ToolbarItem` should primarily model generic hosted content, not a closed set of child control types.
- `ToolbarBuilder` should delegate to or wrap `ControlGroupBuilder<ToolbarItem>` configured with horizontal layout and `ControlGroupFocusStrategy::RovingItemFocus`.
- Keyboard focus, active item tracking, focus-entry redirect, focus-visible item state, and arrow-key movement must come from `control_group`; do not manually store child `FocusHandle`s or reimplement `active_index` traversal in toolbar code.
- Hosted controls with focus handles should be exposed to `control_group` through `ControlGroupFocusTargetProvider<ToolbarItem>`.
- Hosted controls that need arrow keys while focused should use `ControlGroupArrowPolicy::ChildOwnsWhenFocused`.
- `ToolbarTemplate` should be implemented as a toolbar-specific `ControlGroupTemplate<ToolbarItem>` or a thin adapter over it.
- Separators should be toolbar items that render as non-interactive visual dividers and are skipped by control-group navigation.
- Hosted items need metadata for toolbar participation, not concrete type identity. Likely metadata includes item id, enabled state, optional label, whether the item participates in toolbar roving navigation, optional focus target, and whether the hosted control should own arrow-key handling while focused.
- The toolbar should expose two shell variants:
  - `ToolbarVariant::Outline`: bordered shell, matching the current outline-style toolbar direction.
  - `ToolbarVariant::Ghost`: transparent/borderless shell for embedding in denser UI.
- Avoid SDK traits such as `ToolbarAlignmentVariant` or broad `hosted_toolbar_variant()` adapters unless the abstraction is proven generally useful. App-specific sizing such as an alignment selector width should stay in the app/demo or become a clearly named toolbar item configuration.

Focus target guidance:
- The toolbar should not require a broad SDK `Control` trait.
- Ergonomic helpers for known SDK entities may derive focus handles from `Focusable` controls where possible.
- Generic hosted content may need an explicit optional focus target because `control_group` cannot infer a child focus handle from an arbitrary `AnyElement`.
- Items without focus targets may remain navigable, but roving focus will fall back to the group root for those items.
- Separators and group breaks should be excluded from navigation by returning `is_enabled() == false` or by using toolbar-local item metadata that maps to `ControlGroupItemLike::is_enabled() == false`.

Hosted controls need more design work than a quick builder modifier. Buttons, toggles, popup menus, selectors, text fields, and other controls may need toolbar-local visual specialization so their hover/pressed/selected/focused states look correct inside a toolbar. Prefer a theme-level or look-level toolbar-hosted specialization over ad hoc per-control styling. This requires reviewing the theming boundaries for:
- command/icon buttons hosted in toolbars
- toggle buttons hosted in toolbars, especially selected state foreground/background
- popup menu triggers hosted in toolbars
- selector triggers hosted in toolbars
- text fields or other input controls hosted in toolbars, especially arrow-key ownership while focused
- separators, grouping, padding, and shell variants

## Tasks
- [ ] Rework or implement `crates/sdk/src/controls/toolbar/` on top of `control_group`.
- [ ] Define `ToolbarItem` as generic hosted content plus separator/group-break items; do not enumerate allowable child control types.
- [ ] Make `ToolbarItem` implement or adapt into `ControlGroupItemLike`, with separators/non-navigable items excluded from navigation.
- [ ] Configure toolbar's underlying control group with horizontal layout and `ControlGroupFocusStrategy::RovingItemFocus`.
- [ ] Supply hosted control focus targets through a `ControlGroupFocusTargetProvider<ToolbarItem>` or equivalent adapter.
- [ ] Ensure `ToolbarItem` carries toolbar participation metadata and render/content hooks without requiring a broad SDK control trait.
- [ ] Support hosted controls that should consume arrow keys internally, such as text fields, without hardcoding those control types into the toolbar.
- [ ] Map hosted controls that own arrows to `ControlGroupArrowPolicy::ChildOwnsWhenFocused`.
- [ ] Implement `ToolbarBuilder` with horizontal layout by default and `outline()` / `ghost()` variant helpers.
- [ ] Implement toolbar shell theming with `ToolbarVariant::{Outline, Ghost}`.
- [ ] Render separator items as clean vertical dividers using the toolbar/theme border or separator color.
- [ ] Add shadcn look resolving for toolbar shell variants, separator metrics, and hosted-control state decisions.
- [ ] Investigate whether hosted controls need toolbar-local theme specializations rather than generic ghost/outline control styling.
- [ ] Keep template previews outside the Gallery pane file, e.g. `apps/gallery/src/gallery/panes/toolbar/preview.rs`.
- [ ] Gallery `pane.rs` should show a working real-world toolbar example first; template/variant previews should be secondary or live in a dedicated preview module.
- [ ] If Luma Studio needs toolbar previews, reuse the same preview module/pattern where practical instead of duplicating large toolbar construction code.
- [ ] Update `docs/architecture.md` if the final toolbar abstraction changes the SDK customization or composition guidance.

## Acceptance Criteria
- Toolbar is backed by `ControlGroupControl<ToolbarItem>` or a thin wrapper around it.
- Toolbar navigation uses `control_group` behavior; no manual toolbar-local child focus-handle registry or active-index traversal.
- `Tab` enters/leaves the toolbar as one composite control; when roving targets exist, focus enters the active hosted control through `control_group`.
- Arrow keys move between focusable toolbar items unless the focused hosted control is configured with `ControlGroupArrowPolicy::ChildOwnsWhenFocused`.
- Disabled items and separators are skipped by keyboard navigation.
- Separator items draw clean vertical dividers between toolbar groups.
- Outline and ghost toolbar shell variants render distinctly and are theme-driven.
- Toggles and dropdown menus render correctly inside the toolbar.
- Toggle selected state is visually correct in toolbar context without forcing app-local hacks.
- Popup menu and selector triggers work inside the toolbar without trapping focus.
- A text field or other hosted input can be placed in the toolbar without adding a toolbar enum variant for that control type.
- Hosted controls that own arrow keys internally can do so without breaking toolbar navigation for neighboring items once focus leaves that hosted control.
- Toolbar item active/focus-visible visuals come from `CompositeItemState` produced by `control_group`, including roving child focus.
- Gallery has a real working toolbar example and does not keep the full template-preview implementation in `pane.rs`.

## Addendum: Focus Lessons Learned

Failed implementation attempts exposed an important architecture boundary: the existing toolbar cannot be fixed reliably by adding focus callbacks, stdout instrumentation, or a toolbar-local active-index/focus-handle registry. The current toolbar hosts separate child entities (`Button`, `PopupMenu`, `Selector`, etc.) and then tries to coordinate their independent GPUI focus handles from outside. That differs from the working radio/check/toggle group pattern, where `control_group` owns item rendering, item state, active tracking, keyboard handling, and focus-visible visuals as one composite control.

Key lessons:
- Do not patch the old toolbar focus model. It is structurally different from working `control_group` consumers.
- Do not introduce a parallel toolbar focus state machine. That recreates bugs already solved by `control_group`.
- Do not rely on raw `FocusHandle::focus(...)` calls alone. Moving GPUI focus does not automatically synchronize active item, arrow ownership, focus-visible state, or parent composite semantics.
- Do not diagnose toolbar focus using only visual paint. Popup menu and ghost/icon triggers may fail to show focus even when focus moved; tests and instrumentation must prove focus ownership and active item state.
- The correct baseline is to make toolbar follow the working `control_group` rendering/ownership pattern first, before adding text fields or selector internals.

Recommended implementation sequence:
- First build a minimal toolbar specialization that renders command/toggle/menu-trigger faces through `ControlGroupTemplate<ToolbarItem>` and `ControlGroupItemRenderModel`, not by manually tracking child focus handles in toolbar code.
- Verify `Tab` enters the toolbar as one composite and visibly focuses the first item, matching radio/check/toggle group behavior.
- Verify arrow navigation, disabled items, and separators using the `control_group` path.
- Only after the baseline works, add hosted child-focus support for controls that truly need their own focus target, such as text fields or searchable selectors.
- For child-focus controls, define an explicit `control_group` yield contract so a child can return keyboard ownership to the group while preserving the active item.
- Keep the Gallery focus harness (`Focus Before`, toolbar, `Focus After`) while developing this issue so enter/leave behavior is always manually testable.
