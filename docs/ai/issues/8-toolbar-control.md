# Issue #8: Create Toolbar Control

## Description
We need a cohesive Toolbar SDK control that arranges command buttons, toggle buttons, menu triggers, selector-style controls, and separators horizontally. Apps currently have to compose these manually with raw flex rows, which makes keyboard behavior, spacing, separators, and theme variants inconsistent.

The toolbar must not create a second focus/navigation system. It should reuse the existing `control_group` infrastructure for item modeling, active item handling, roving keyboard behavior, item state, layout direction, and template seams.

## Proposed Solution
Create a new `Toolbar` SDK control as a toolbar-specific specialization around `control_group`. The toolbar should not predefine the concrete controls it can host. A proper toolbar is a composite row of generic hosted items plus separators; callers must be able to host buttons, toggles, menus, selectors, text fields, color pickers, or future controls without the toolbar API adding a new enum variant for each one.

Required architectural direction:
- `ToolbarItem` should implement `ControlGroupItemLike` or be adapted into a `ControlGroupItemLike` model.
- `ToolbarItem` should primarily model generic hosted content, not a closed set of child control types.
- `ToolbarBuilder` should delegate to, wrap, or closely mirror `ControlGroupBuilder<ToolbarItem>`.
- Keyboard focus, active item tracking, and arrow-key movement must come from `control_group`; do not manually store child `FocusHandle`s or reimplement `active_index` traversal in toolbar code.
- `ToolbarTemplate` should be implemented as a toolbar-specific `ControlGroupTemplate<ToolbarItem>` or a thin adapter over it.
- Separators should be toolbar items that render as non-interactive visual dividers and are skipped by control-group navigation.
- Hosted items need metadata for toolbar participation, not concrete type identity. Likely metadata includes item id, enabled state, optional label, whether the item participates in toolbar roving navigation, and whether the hosted control should own arrow-key handling while focused.
- The toolbar should expose two shell variants:
  - `ToolbarVariant::Outline`: bordered shell, matching the current outline-style toolbar direction.
  - `ToolbarVariant::Ghost`: transparent/borderless shell for embedding in denser UI.
- Avoid SDK traits such as `ToolbarAlignmentVariant` or broad `hosted_toolbar_variant()` adapters unless the abstraction is proven generally useful. App-specific sizing such as an alignment selector width should stay in the app/demo or become a clearly named toolbar item configuration.

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
- [ ] Ensure `ToolbarItem` carries only toolbar participation metadata and render/content hooks; it must not require callers to provide child focus handles.
- [ ] Support hosted controls that should consume arrow keys internally, such as text fields, without hardcoding those control types into the toolbar.
- [ ] Implement `ToolbarBuilder` with horizontal layout by default and `outline()` / `ghost()` variant helpers.
- [ ] Implement toolbar shell theming with `ToolbarVariant::{Outline, Ghost}`.
- [ ] Render separator items as clean vertical dividers using the toolbar/theme border or separator color.
- [ ] Add shadcn look resolving for toolbar shell variants, separator metrics, and hosted-control state decisions.
- [ ] Investigate whether hosted controls need toolbar-local theme specializations rather than generic ghost/outline control styling.
- [ ] Keep template previews outside the Gallery pane file, e.g. `apps/gallery/src/gallery/panes/toolbar/preview.rs`.
- [ ] Gallery `pane.rs` should show a working real-world toolbar example first; template/variant previews should be secondary or live in a dedicated preview module.
- [ ] If Theme Studio needs toolbar previews, reuse the same preview module/pattern where practical instead of duplicating large toolbar construction code.
- [ ] Update `docs/architecture.md` if the final toolbar abstraction changes the SDK customization or composition guidance.

## Acceptance Criteria
- Toolbar navigation uses `control_group` behavior; no manual toolbar-local child focus-handle registry.
- `Tab` enters/leaves the toolbar as one composite control; arrow keys move between focusable toolbar items.
- Disabled items and separators are skipped by keyboard navigation.
- Separator items draw clean vertical dividers between toolbar groups.
- Outline and ghost toolbar shell variants render distinctly and are theme-driven.
- Toggles and dropdown menus render correctly inside the toolbar.
- Toggle selected state is visually correct in toolbar context without forcing app-local hacks.
- Popup menu and selector triggers work inside the toolbar without trapping focus.
- A text field or other hosted input can be placed in the toolbar without adding a toolbar enum variant for that control type.
- Hosted controls that own arrow keys internally can do so without breaking toolbar navigation for neighboring items.
- Gallery has a real working toolbar example and does not keep the full template-preview implementation in `pane.rs`.
