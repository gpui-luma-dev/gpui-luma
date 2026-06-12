# Issue #14: Theme Studio Tab Control Theme Update Bug

## Description
When changes are made to the active theme/mode inside the Theme Studio, the tab control located on the right content screen (switching between "Cards", "Dashboard", and "Palette") does not update its styling. It remains rendered with the theme variables/mode active at startup.

## Proposed Solution
Currently, the `TabsNavigation` entity does not expose an API to change its template/theme reference after creation. We need to implement a template updater method on the control and call it when the theme changes.

## Tasks
- [ ] Add `set_template(&mut self, template: Arc<dyn TabsNavigationTemplate>, cx: &mut Context<Self>)` inside [control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/tabs_navigation/control.rs).
- [ ] In the Theme Studio, update `sync_board_snapshot` in [content_pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/theme-studio/src/studio/content_pane.rs) to resolve a new tab template via `gpui_luma_look_shadcn::tabs_navigation_template(look)` and apply it to the tabs element:
  ```rust
  let template = gpui_luma_look_shadcn::tabs_navigation_template(look.clone());
  self.tabs.update(cx, |tabs, cx| tabs.set_template(template, cx));
  ```

## Acceptance Criteria
- Toggling the dark/light mode or changing color variables in Theme Studio instantly updates the visual look and theme colors of the tab navigation bar.
