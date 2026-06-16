# SDK API Consistency Audit Findings

Scope: audit of the public SDK control surface in `crates/sdk/src/controls`, wrapper/factory usage in `crates/look-shadcn/src/controls/ext.rs`, and downstream consumption in `apps/`.

This report records only issues verified directly in source.

## Summary

Most common inconsistency patterns found:
- runtime mutators using `with_...` on spawned controls
- missing top-level `enabled` parity on interactive controls
- builder/runtime template parity gaps
- naming drift in the color control family (`with_size`, `disabled`)

---

## 1. `SelectionPanel` runtime mutators use `with_...` on the spawned control

- **Control Name:** `SelectionPanel`
- **Layer:** `runtime`, `look-factory`, `app-usage`
- **Current API:** `SelectionPanelControl` exposes `with_template(...)`, `with_item_template(...)`, and `with_scrollbar_template(...)` on the spawned control.
- **Expected API / Violated Rule:** Rule B. Runtime mutators on spawned controls should use `set_...` names and must not use `with_...`.
- **File Path:** `crates/sdk/src/controls/selection_panel/control.rs`
- **Downstream Usages:**
  - `crates/look-shadcn/src/controls/ext.rs` uses `panel.with_scrollbar_template(...)`
  - `apps/gallery/src/gallery/panes/selection_panel/pane.rs` uses `panel.with_item_template(...)` and `panel.with_template(...)`
- **Migration Impact:** Medium. Both `look-shadcn` and gallery currently call these runtime APIs.

---

## 2. `ListViewControl` exposes runtime `with_...` mutators

- **Control Name:** `ListView`
- **Layer:** `runtime`
- **Current API:** `ListViewControl` exposes `with_header_template(...)` and `with_row_template(...)` in addition to `set_header_template(...)`.
- **Expected API / Violated Rule:** Rule B. Spawned-control mutators should not use `with_...`; runtime mutation should be expressed through `set_...` APIs.
- **File Path:** `crates/sdk/src/controls/list_view/control.rs`
- **Downstream Usages:** no app/runtime call sites found in `apps/`, but the API is publicly exposed on the spawned control.
- **Migration Impact:** Low. Likely a direct internal rename/update cleanup.

---

## 3. `ListView` has builder-side `appearance_override(...)` but no runtime parity

- **Control Name:** `ListView`
- **Layer:** `builder`, `runtime`
- **Current API:** `ListViewBuilder` exposes `appearance_override(...)`, but `ListViewControl` does not expose `set_appearance_override(...)`.
- **Expected API / Violated Rule:** Rule C. Post-resolution appearance overrides should standardize on builder `appearance_override(...)` plus runtime `set_appearance_override(..., cx)` parity.
- **File Path:**
  - `crates/sdk/src/controls/list_view/model.rs`
  - `crates/sdk/src/controls/list_view/control.rs`
- **Downstream Usages:** builder-side appearance override is part of the public API; no runtime usage exists because no runtime setter exists.
- **Migration Impact:** Low. Additive runtime API.

---

## 4. `AutocompleteTextBox` is missing top-level enabled-state parity

- **Control Name:** `AutocompleteTextBox`
- **Layer:** `builder`, `runtime`, `look-factory`, `app-usage`
- **Current API:** `AutocompleteTextBoxModel` has no `enabled: bool`; `AutocompleteTextBoxBuilder` has no `.enabled(bool)`; `AutocompleteTextBoxControl` has no `set_enabled(bool, cx)`.
- **Expected API / Violated Rule:** Rule E. Interactive controls should support top-level enabled parity across model, builder, and runtime control.
- **File Path:**
  - `crates/sdk/src/controls/autocomplete/model.rs`
  - `crates/sdk/src/controls/autocomplete/control.rs`
- **Downstream Usages:**
  - `crates/look-shadcn/src/controls/ext.rs` exposes `look.autocomplete(...)`
  - `apps/gallery/src/gallery/panes/autocomplete/pane.rs` constructs it via `look.autocomplete(...)`
- **Migration Impact:** Medium. Requires disabled-state wiring through interaction and rendering.

---

## 5. `ComboBox` is missing enabled parity and per-item enabled support

- **Control Name:** `ComboBox`
- **Layer:** `builder`, `runtime`, `look-factory`, `app-usage`
- **Current API:**
  - `ComboBoxModel` has no top-level `enabled: bool`
  - `ComboBoxBuilder` has no `.enabled(bool)`
  - `ComboBoxControl` has no `set_enabled(bool, cx)`
  - `SelectionItem` only has `id` and `label`
  - render models hardcode `enabled: true`
- **Expected API / Violated Rule:** Rule E. Missing top-level enabled parity, and item-based selection controls should audit/support per-item enabled state.
- **File Path:**
  - `crates/sdk/src/controls/combobox/model.rs`
  - `crates/sdk/src/controls/combobox/control.rs`
  - `crates/sdk/src/controls/combobox/behavior.rs`
- **Downstream Usages:**
  - `crates/look-shadcn/src/controls/ext.rs` exposes `look.combobox(...)`
  - `apps/gallery/src/gallery/panes/combobox/pane.rs`
  - `apps/gallery/src/gallery/panes/introduction/payment_panel.rs`
- **Migration Impact:** Medium. Requires state/threading through behavior, templates, and factories.

---

## 6. `SearchSelector` is missing enabled parity and per-item enabled support

- **Control Name:** `SearchSelector`
- **Layer:** `builder`, `runtime`, `look-factory`, `app-usage`
- **Current API:**
  - `SearchSelectorModel` has no top-level `enabled: bool`
  - `SearchSelectorBuilder` has no `.enabled(bool)`
  - `SearchSelectorControl` has no `set_enabled(bool, cx)`
  - `SelectionItem` only has `id` and `label`
  - `SearchSelectorControl` creates `ControlInteraction::new(true, cx)` and render models pass `enabled: true`
- **Expected API / Violated Rule:** Rule E. Missing top-level enabled parity, and item-based selection controls should audit/support per-item enabled state.
- **File Path:**
  - `crates/sdk/src/controls/search_selector/model.rs`
  - `crates/sdk/src/controls/search_selector/control.rs`
  - `crates/sdk/src/controls/search_selector/behavior.rs`
- **Downstream Usages:**
  - `crates/look-shadcn/src/controls/ext.rs` exposes `look.search_selector(...)`
  - `apps/gallery/src/gallery/panes/search_selector/pane.rs`
- **Migration Impact:** Medium. Requires behavior, template, and interaction-state wiring.

---

## 7. `NavigationSidebar` lacks top-level enabled parity

- **Control Name:** `NavigationSidebar`
- **Layer:** `builder`, `runtime`, `look-factory`, `app-usage`
- **Current API:** `NavNode` supports per-item `.enabled(...)`, but the top-level sidebar model has no `enabled: bool`, the builder has no `.enabled(bool)`, and the control has no `set_enabled(bool, cx)`.
- **Expected API / Violated Rule:** Rule E. Top-level interactive controls should support enabled parity even when child items already do.
- **File Path:**
  - `crates/sdk/src/controls/navigation_sidebar/model.rs`
  - `crates/sdk/src/controls/navigation_sidebar/control.rs`
- **Downstream Usages:**
  - `crates/look-shadcn/src/controls/ext.rs` exposes `look.navigation_sidebar(...)`
  - `apps/gallery/src/gallery/control.rs`
  - `apps/gallery/src/gallery/panes/navigation_sidebar/pane.rs`
  - `apps/theme-studio/src/studio/panels/dashboard.rs`
- **Migration Impact:** Medium. Requires keyboard/mouse interaction gating at the top-level sidebar.

---

## 8. `AccordionControl` lacks runtime enabled parity

- **Control Name:** `Accordion`
- **Layer:** `runtime`, `app-usage`
- **Current API:** `AccordionModel` and `AccordionBuilder` support `enabled`, and `AccordionItem` supports per-item `.enabled(...)`, but `AccordionControl` only exposes `set_template(...)` and toggle/navigation methods; there is no `set_enabled(bool, cx)`.
- **Expected API / Violated Rule:** Rule E. Top-level controls should expose runtime enabled parity.
- **File Path:**
  - `crates/sdk/src/controls/accordion/model.rs`
  - `crates/sdk/src/controls/accordion/control.rs`
- **Downstream Usages:**
  - `apps/gallery/src/gallery/panes/accordion/pane.rs` uses `AccordionItem::enabled(false)` at build time
  - `apps/theme-studio/src/studio/panels/accordion.rs`
  - `apps/theme-studio/src/studio/theme_sidebar.rs`
- **Migration Impact:** Low. Additive runtime setter.

---

## 9. `Selector` drops disabled items instead of preserving them as disabled choices

- **Control Name:** `Selector` / `SelectorItem`
- **Layer:** `builder`, `runtime`, `app-usage`
- **Current API:** `SelectorItem::enabled(false)` exists, but `normalize_selector_items(...)` filters disabled items out entirely, and both `SelectorBuilder::items(...)` and `Selector::set_items(...)` use that normalizer.
- **Expected API / Violated Rule:** Rule E per-item parity. Disabled items should remain part of the control surface as disabled items, not be silently removed from the item set.
- **File Path:**
  - `crates/sdk/src/controls/selector_panel/model.rs`
  - `crates/sdk/src/controls/selector/model.rs`
  - `crates/sdk/src/controls/selector/control.rs`
- **Downstream Usages:** `apps/gallery/src/gallery/panes/selector/pane.rs` uses selector item templating on top of the public `SelectorItem` surface.
- **Migration Impact:** Medium. This is a behavior change, not just a rename.

---

## 10. Builder/runtime template parity is missing across multiple controls

- **Control Name:** `PopupMenu`, `ContextMenu`, `Progress`, `Slider`, `Scrollbar`, `SplitView`, `ResizablePanels`, `NavigationSidebar`, `ControlGroup`, `TextArea`
- **Layer:** `builder`, `runtime`, `look-factory`, `app-usage`
- **Current API:** these builders expose `.template(...)` (and in some cases related builder-only presentation hooks like `item_template(...)`, `layout(...)`, `scrollbar_template(...)`, or `.theme(...)`), but the spawned controls do not expose matching runtime setters.
- **Expected API / Violated Rule:** Step 3 parity audit. Where the builder exposes a mutable model field for template/presentation configuration, the spawned control should expose corresponding `set_...` runtime parity where runtime mutation is part of the control style.
- **File Path:**
  - `crates/sdk/src/controls/popup_menu/model.rs`, `crates/sdk/src/controls/popup_menu/control.rs`
  - `crates/sdk/src/controls/context_menu/model.rs`, `crates/sdk/src/controls/context_menu/control.rs`
  - `crates/sdk/src/controls/progress/model.rs`, `crates/sdk/src/controls/progress/control.rs`
  - `crates/sdk/src/controls/slider/model.rs`, `crates/sdk/src/controls/slider/control.rs`
  - `crates/sdk/src/controls/scrollbar/model.rs`, `crates/sdk/src/controls/scrollbar/control.rs`
  - `crates/sdk/src/controls/split_view/model.rs`, `crates/sdk/src/controls/split_view/control.rs`
  - `crates/sdk/src/controls/resizable_panels/model.rs`, `crates/sdk/src/controls/resizable_panels/control.rs`
  - `crates/sdk/src/controls/navigation_sidebar/model.rs`, `crates/sdk/src/controls/navigation_sidebar/control.rs`
  - `crates/sdk/src/controls/control_group/model.rs`, `crates/sdk/src/controls/control_group/control.rs`
  - `crates/sdk/src/controls/textarea/model.rs`, `crates/sdk/src/controls/textarea/control.rs`
- **Downstream Usages:**
  - `apps/gallery/src/gallery/control.rs` mutates `NavigationSidebar` and `SplitView` at runtime, but only on non-template properties
  - `apps/theme-studio/src/studio/app.rs` calls `ResizablePanels::set_theme(...)`
  - `apps/gallery/src/gallery/panes/context_menu/pane.rs` and `apps/gallery/src/gallery/panes/popup_menu/pane.rs` configure templates only at build time
  - `apps/gallery/src/gallery/panes/selection_panel/pane.rs` and `apps/gallery/src/gallery/panes/selector/pane.rs` show that related composite controls do support more runtime presentation mutation, highlighting the parity gap here
- **Migration Impact:** Medium. Mostly additive setters, but the surface area is broad.

---

## 11. `TextArea` lacks `appearance_override` parity and runtime theme/template mutation

- **Control Name:** `TextArea`
- **Layer:** `builder`, `runtime`, `look-factory`, `app-usage`
- **Current API:**
  - `TextAreaBuilder` exposes `.template(...)` and `.theme(...)`
  - `TextArea` exposes `set_enabled`, `set_value`, `set_placeholder`, `set_clean_on_escape`, and `set_validator`
  - unlike `TextField`, there is no builder `appearance_override(...)` and no runtime `set_appearance_override(...)`
  - there is also no `set_template(...)` or `set_theme(...)`
- **Expected API / Violated Rule:**
  - Rule C for post-resolution appearance override parity on text-input controls
  - Step 3 parity for builder/runtime theme/template mutation
- **File Path:**
  - `crates/sdk/src/controls/textarea/model.rs`
  - `crates/sdk/src/controls/textarea/control.rs`
- **Downstream Usages:**
  - `crates/look-shadcn/src/controls/ext.rs` exposes `look.textarea(...)`
  - `apps/gallery/src/gallery/panes/textarea/pane.rs`
  - `apps/theme-studio/src/studio/panels/report.rs`
  - `apps/theme-studio/src/studio/panels/upgrade.rs`
  - contrast: `apps/theme-studio/src/studio/theme_sidebar.rs` actively uses `TextField::set_template(...)` and `TextField::set_appearance_override(...)`
- **Migration Impact:** Medium. Mostly additive, but parity with `TextField` implies more than one new API.

---

## 12. Color-family builder naming is inconsistent with the audit rules

- **Control Name:** `ColorArc`, `ColorRing`, `ColorSlider`, `ColorField`
- **Layer:** `builder`, `runtime`, `app-usage`
- **Current API:**
  - direct size configuration uses `.with_size(...)`
  - construction-time enablement is expressed as `.disabled(bool)` rather than `.enabled(bool)`
  - runtime APIs expose both `set_disabled(...)` and `set_enabled(...)`
- **Expected API / Violated Rule:**
  - Rule A: direct field configuration should use `.size(...)`, not `with_size(...)`
  - Rule E: top-level enabled-state API should standardize on `enabled` / `set_enabled`
- **File Path:**
  - `crates/sdk/src/controls/color/style.rs`
  - `crates/sdk/src/controls/color/color_arc/model.rs`
  - `crates/sdk/src/controls/color/color_arc/arc.rs`
  - `crates/sdk/src/controls/color/color_ring/model.rs`
  - `crates/sdk/src/controls/color/color_ring/ring.rs`
  - `crates/sdk/src/controls/color/color_slider/model.rs`
  - `crates/sdk/src/controls/color/color_slider/slider.rs`
  - `crates/sdk/src/controls/color/color_field/field/state.rs`
- **Downstream Usages:** widespread gallery usage of `.with_size(...)`, including:
  - `apps/gallery/src/gallery/panes/color/arc_pane.rs`
  - `apps/gallery/src/gallery/panes/color/ring_pane.rs`
  - `apps/gallery/src/gallery/panes/color/picker_pane.rs`
  - `apps/gallery/src/gallery/panes/color/multi_mixer_pane.rs`
- **Migration Impact:** Medium. Public naming cleanup will require updating a broad set of internal gallery call sites.

---

## Notes

A few positive checks from this pass:
- public builder `set_...` methods were not found in `crates/sdk/src/controls/**/model.rs`
- `TextField` already matches the strongest form of the appearance override rule with both builder `appearance_override(...)` and runtime `set_appearance_override(...)`
