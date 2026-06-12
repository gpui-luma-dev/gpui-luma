# Issue #8: Create Toolbar Control

## Description
We need a cohesive Toolbar layout control shell that arranges buttons, toggles, menus, and separators horizontally. Currently, apps must place buttons side-by-side using raw flex columns/rows.

## Proposed Solution
Create a new `Toolbar` SDK control. Internally, it can leverage elements of `control_group` but should support nesting diverse sub-components (normal buttons, toggle states, popup triggers, and separators) within a continuous row.

## Tasks
- [ ] Define a `ToolbarItem` enum representing: `Button`, `Toggle`, `Menu`, and `Separator` variants under a new `crates/sdk/src/controls/toolbar/` folder.
- [ ] Implement `ToolbarBuilder` and `ToolbarControl` entity to manage selection and layouts.
- [ ] Render a vertical line with the theme's border color for the `Separator` variant.
- [ ] Add look resolving mappings in `crates/look-shadcn` to format item grouping corners and paddings.
- [ ] Create a Toolbar demo section in the Gallery.

## Acceptance Criteria
- Separator items draw clean vertical dividers between toolbar groups.
- Toggles and dropdown menus render correctly inside the toolbar element.
