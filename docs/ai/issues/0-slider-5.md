# Slider 5: Legacy Slider Replacement & Migration Plan

This plan details the migration path from the legacy single-thumb `Slider` control to the unified `Slider2` engine.

## Goal
Replace the old `Slider` implementation with `Slider2` while preserving maximum API compatibility for existing consumers, and without modifying the isolated `ColorSlider` controls.

---

## 1. API & Type Mapping

The new builder/model methods are designed symmetrically to the old ones. The main type adjustments required are:
* **Event Subscription**: Client code subscribing to `SliderEvent::Change` or `SliderEvent::Release` must destructure or ignore the new `thumb_id` field:
  ```rust
  // Old
  SliderEvent::Change { value } => { ... }
  // New
  SliderEvent::Change { value, .. } => { ... }
  ```
* **Enums/Types Mapping**:
  * `SliderInputStrategy` $\rightarrow$ `Slider2InputStrategy`
  * `SliderOrientation` $\rightarrow$ `Slider2Orientation`
  * `SliderThumbSize` $\rightarrow$ `Slider2ThumbSize`

---

## 2. Export & File System Impact

* **Renaming `slider2` to `slider`**:
  - Delete the old `crates/sdk/src/controls/slider/` package.
  - Move/rename the contents of `crates/sdk/src/controls/slider2/` into its place.
* **Alias Compatibility**:
  To minimize compilation breakages across the repository, we can re-export the new types under their legacy names inside `crates/sdk/src/controls/slider/mod.rs`:
  ```rust
  pub type Slider = Entity<Slider2Control>;
  pub type SliderEvent = Slider2Event;
  pub type SliderBuilder = Slider2Builder;
  pub type SliderOrientation = Slider2Orientation;
  pub type SliderThumbSize = Slider2ThumbSize;
  ```

---

## 3. Look & Theme System Impact

* **`look-shadcn` Template Registration**:
  - In [look.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/look.rs), the `.slider_template()` method will return the new linear template.
  - In [ext.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/ext.rs), the `.slider()` extension builder will call `slider2::new()` and bind the new template. The `.slider2()` builder can then be removed.
* **Theme Styling Rules**: 
  The new templates in `slider2` already reuse `crates::controls::slider::SliderLook` (which draws from the existing CSS stylesheet definitions under the hood), meaning existing Shadcn light/dark look colors will map over seamlessly.

---

## 4. Client Code Impact

The old `Slider` is currently used in the following locations, which must be updated:
1. **Gallery App (`apps/gallery/`)**:
   * [pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/slider/pane.rs): Merge the old `pane.rs` and the new `slider2_pane.rs` into a single unified `slider_pane.rs` showing off all variants (standard fill, vertical, reversed, domain, blocked, angular, wrapping, and multi-stop).
   * [system_panel.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/introduction/system_panel.rs) & [shadow_button/pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/prototypes/shadow_button/pane.rs): Update imports and handle event destructuring.
2. **Theme Studio (`apps/theme-studio/`)**:
   * [system_preferences.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/theme-studio/src/studio/panels/system_preferences.rs) & [other.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/theme-studio/src/studio/theme_sidebar/panels/other.rs): Update imports and handle event destructuring.
3. **Neumorphic Demo (`apps/neumorphic-demo/`)**:
   * [app.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/neumorphic-demo/src/app.rs), [slider_template.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/neumorphic-demo/src/slider_template.rs), and [dial.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/neumorphic-demo/src/components/dial.rs): Update custom templates to implement `Slider2Template` (adding `active_thumb_id` and using `Slider2RenderModel`).

---

## 5. Color Slider Isolation (Safety)

* **Zero Impact on `ColorSlider`**: 
  The existing `ColorSlider` module (`crates/sdk/src/controls/color/color_slider/`) remains completely untouched and independent, satisfying the safety requirement of keeping color controls isolated during this migration.
