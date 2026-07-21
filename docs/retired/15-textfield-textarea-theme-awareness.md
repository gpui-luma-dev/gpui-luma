# Issue #15: Textfield/Textarea Theme Awareness Bug in Luma Studio

## Description
When global color overrides are applied, the Luma Studio recreates the demo panels (`refresh_demos`), updating them with the overridden look. However, when the light/dark mode switch is toggled in the title bar afterwards, the demo panels are not recreated or notified. Because GPUI caches view rendering for existing child entities unless they are explicitly notified or recreated, none of the controls inside the cards (especially text fields and textareas) update their visual representation to reflect the new mode.

## Proposed Solution
Recreate the demo panels when toggling the theme mode in the Luma Studio title bar. Calling `refresh_demos(cx)` during the mode toggle handler will spawn fresh panel entities referencing the updated look, forcing GPUI to re-render them under the correct theme mode.

## Tasks
- [ ] Update the light/dark mode toggle click listener in [app.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/luma-studio/src/studio/app.rs) to call `this.refresh_demos(cx)`:
  ```rust
  this.look.set_mode(mode);
  let theme = this.look.clone();
  this.refresh_demos(cx); // Recreate panels with the updated mode state
  this.theme_sidebar.update(cx, |sidebar, cx| {
      sidebar.sync_control_templates(&theme, cx);
  });
  ```

## Acceptance Criteria
- Toggling the light/dark mode switch in the Luma Studio correctly updates all text fields, textareas, buttons, and card containers on the content board, even after global color overrides have been set.
