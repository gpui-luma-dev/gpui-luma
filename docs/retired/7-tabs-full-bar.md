# Issue #7: Tab Control Full Bar Highlight

## Description
The existing tab control (`TabsNavigation`) displays active bottom indicators under each item individually, but lacks a full-width continuous bottom border that anchors the tabs. We need a "full bar" bottom rule spanning the entire width of the tabs container, upon which the active tab highlights overlay directly.

## Proposed Solution
1. Add `full_bar_color` to `TabsNavigationListLook`.
2. Update the list template rendering to draw a bottom border/bar.
3. Position individual active tab indicators to sit precisely on top of the list's bottom border.

## Tasks
- [ ] Add `full_bar_color: Option<Hsla>` to `TabsNavigationListLook` in [theme.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/tabs_navigation/theme.rs).
- [ ] Render a bottom border in [template.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/tabs_navigation/template.rs) when `full_bar_color` resolves to a color.
- [ ] Adjust active tab indicators inside `render_tabs_navigation_item_visual` to overlay the bottom line.
- [ ] Map `full_bar_color` in [tabs_navigation.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/tabs_navigation.rs).

## Acceptance Criteria
- Tab components show a continuous bottom border spanning the container.
- Selected indicators align on top of the bottom rule without creating vertical gap artifacts.
