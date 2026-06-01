# Design Debt: Leaky Layout Abstraction (`StandardBoxScale` Cleanup)

This document describes the leaky abstraction discovered inside the global sizing module ([`crates/sdk/src/theme/layout.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/theme/layout.rs)) and details a plan to relocate control-specific geometry calculations into their respective component directories as a separate, follow-up task.

---

## 1. The Issue Discovered

During the migration of the **Switch** and **Checkbox** controls to the decoupled layout metrics model, we established the "Pragmatic Compromise": grouping components under shared layout contracts (such as `StandardBoxScale` and `GlyphIndicatorScale`).

However, this design introduced a **leaky abstraction**:
* **`StandardBoxScale`** is intended to represent a generic, rectangular boundary box used by Buttons, TextFields, TextAreas, and Selector dropdown triggers.
* To avoid creating separate layout compute modules, Switch-specific track metrics (`track_width`, `track_height`, `track_padding`, `thumb_size`, `track_radius`) were added directly to `StandardBoxScale`.
* Similarly, **`GlyphIndicatorScale`** contains fields for both checkbox-specific glyph structures (`glyph_size`) and radio-specific dot structures (`dot_size`).

This means the generic layout structs are polluted with fields that are only read by a single specific control. If a developer edits Switch track proportions, they are modifying a shared struct used by Buttons and TextFields, which increases regression risk and makes file-chasing painful.

---

## 2. Refactoring Goal: Co-located Layout Scaling

To restore a clean separation of concerns and eliminate file-chasing, the layout math should be moved directly into the component directories:
* **Switch:** Sizing ratios and `SwitchScale::compute` move to [`crates/sdk/src/controls/switch/theme.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/switch/theme.rs) (or a new `layout.rs` inside the same directory).
* **Checkbox:** Sizing ratios and `CheckboxScale::compute` move to [`crates/sdk/src/controls/checkbox/theme.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/checkbox/theme.rs).
* **`StandardBoxScale`:** Stays in `theme/layout.rs` but is stripped down to generic box dimensions (`height`, `padding_x`, `padding_y`, `gap`, `radius`).

---

## 3. Plan of Action (Follow-Up Task)

This refactoring should be done as a separate, standalone chore once the HTML styling and spec exporter toolchain is fully completed.

### Step 1: Encapsulate Switch Layout
1. In `crates/sdk/src/controls/switch/theme.rs`, implement `SwitchScale` directly:
   ```rust
   impl SwitchScale {
       pub fn compute(size: ControlSize, metrics: &MetricTokens, scale_factor: f32) -> Self {
           let control_height = metrics.control_height(size);
           let width = snap_to_pixel(control_height * (34.0 / 36.0), scale_factor);
           let height = snap_to_pixel(control_height * (18.0 / 36.0), scale_factor);
           let thumb_size = snap_to_pixel(control_height * (14.0 / 36.0), scale_factor);
           let padding = snap_to_pixel(2.0, scale_factor);

           Self {
               width,
               height,
               thumb_size,
               padding,
               gap: snap_to_pixel(metrics.gap(size), scale_factor),
               radius: effective_pill_radius(width, height, metrics.radius.pill),
               label_baseline_shift: match size {
                   ControlSize::Sm => 0.5,
                   ControlSize::Md => 1.0,
                   ControlSize::Lg => 1.5,
               },
           }
       }
   }
   ```
2. Update `crates/sdk/src/controls/switch/template.rs` to compute `SwitchScale` directly instead of translating from `StandardBoxScale`.

### Step 2: Encapsulate Checkbox & Radio Layout
1. In `crates/sdk/src/controls/checkbox/theme.rs`, implement `CheckboxScale` directly using its own local glyph-indicator math.
2. In `crates/sdk/src/controls/radio_button/theme.rs`, implement `RadioScale` locally.

### Step 3: Sanitize `StandardBoxScale` & `GlyphIndicatorScale`
1. Open [`crates/sdk/src/theme/layout.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/theme/layout.rs).
2. Delete the fields `track_width`, `track_height`, `track_padding`, `thumb_size`, `track_radius` from `StandardBoxScale`.
3. Delete the fields `glyph_size`, `dot_size`, `icon_inset`, `icon_stroke_width` from `GlyphIndicatorScale`.
4. Delete `SwitchScale` and `CheckboxScale` entirely from `theme/layout.rs`.

---

## 4. Verification

After completing the move:
* Run `cargo test` to ensure all spec tests compile and pass.
* Run the styleguide exporter to verify that the generated JSON metrics in `styleguide.html` remain identical to the previous outputs.
