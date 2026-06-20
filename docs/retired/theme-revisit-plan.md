# Implementation Plan: Structural Density & Theme Decoupling

This document outlines a phased strategy to refactor the GPUI-Luma component SDK. It shifts the architecture from flat look models to decoupled **Layout Scales** (SDK-owned) and **Visual Palettes** (Theme-owned), referencing the design spec in [`theme-revisit.md`](theme-revisit.md).

To keep the refactoring surface manageable, we adopt the **Pragmatic Compromise**: grouping the 30+ controls under three shared layout contracts rather than creating 30 separate scale structs.

---

## Phase 1: Sizing Infrastructure & Caching
Before touching any UI controls, implement the shared math-scaling, pixel-snapping, and global layout caching systems.

### 1. Sizing and Pixel Snapping Utility
* **Target File:** Create `crates/sdk/src/theme/layout.rs` (or add to `theme/mod.rs`).
* **Tasks:**
  * Implement `snap_to_pixel(value: f32, scale_factor: f32) -> f32`.
  * Add unit tests verifying physical snapping at `1.0`, `1.5`, and `2.0` display scale factors.

### 2. Layout Scale Cache
* **Target File:** Create `crates/sdk/src/theme/cache.rs`.
* **Tasks:**
  * Define `LayoutCacheKey` containing `ControlSize` and `scale_factor_bits: u32`.
  * Define `LumaLayoutCacheExt` extension trait for GPUI context (`WindowContext` / `App`).
  * Integrate the type-map layout cache into `ThemeTokens` (cleared automatically whenever the active theme changes).

### 3. Core Shared Layout Scales
* **Target File:** Add to `crates/sdk/src/theme/layout.rs`.
* **Tasks:**
  * Define **`StandardBoxScale`**: For Switches, Buttons, Inputs, and Selectors.
  * Define **`GlyphIndicatorScale`**: For Checkboxes, Radios, and Accordion Chevrons.
  * Define **`ListRowScale`**: For List Rows and Menu Items.

---

## Phase 2: Pilot Components (Switch & Checkbox)
Refactor the two reference controls to validate the decoupling and caching patterns before rolling them out globally.

### 1. Refactor Switch
* **Target Files:** 
  * [`crates/sdk/src/controls/switch/theme.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/switch/theme.rs)
  * [`crates/sdk/src/controls/switch/template.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/switch/template.rs)
* **Tasks:**
  * Remove layout variables (`width`, `height`, `thumb_size`, etc.) from `SwitchLook` (rename to `SwitchPalette`).
  * Modify `SwitchTheme` trait to resolve `SwitchPalette` (no size parameter needed).
  * Wire `ThemedSwitchTemplate` to fetch cached layout metrics via `cx.use_cached_layout(..., |metrics| SwitchScale::compute(...))`.
  * Update default switch theme and Radix switch theme mappings.

### 2. Refactor Checkbox
* **Target Files:**
  * [`crates/sdk/src/controls/checkbox/theme.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/checkbox/theme.rs)
  * [`crates/sdk/src/controls/checkbox/template.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/checkbox/template.rs)
* **Tasks:**
  * Replace flat sizing metrics in `CheckboxLook` with `GlyphIndicatorScale`.
  * Update `CheckboxTheme::resolve` to return visual `CheckboxPalette`.
  * Adapt `ThemedCheckboxTemplate` to apply the `label_baseline_shift` top-margin nudge on the label node.

---

## Phase 3: Rollout to Buttons & Toggles
Extend the pattern to the primary click and toggle targets.

### 1. Refactor Button & Icon Button Family
* **Target Files:**
  * `crates/sdk/src/controls/button_family/theme.rs`
  * `crates/sdk/src/controls/button_family/template.rs`
* **Tasks:**
  * Migrate buttons to use `StandardBoxScale` for bounding-box geometry (height, padding, gap, radius).
  * Separate theme-resolved colors into `ButtonPalette`.

### 2. Refactor Radio Buttons & Toggles
* **Target Files:**
  * `crates/sdk/src/controls/radio_button/theme.rs`
  * `crates/sdk/src/controls/toggle/theme.rs`
* **Tasks:**
  * Update Radio Buttons to use `GlyphIndicatorScale` to compute track circle and indicator dot dimensions.

---

## Phase 4: Rollout to Text Anchors & Inputs
Refactor components with text input boxes and drop-down selectors.

### 1. Refactor TextField & TextArea
* **Target Files:**
  * `crates/sdk/src/controls/textfield/theme.rs`
  * `crates/sdk/src/controls/textarea/theme.rs`
* **Tasks:**
  * Bind text fields to `StandardBoxScale` (padding, heights, radius).
  * Connect the typographic properties to the coordinated typography scale (e.g., using curated `typography.text` sizes instead of computing text size mathematically).

### 2. Refactor Selectors, Comboboxes, and Popups
* **Target Files:**
  * `crates/sdk/src/controls/selector/theme.rs`
  * `crates/sdk/src/controls/popup_menu/theme.rs`
* **Tasks:**
  * Decouple the popup trigger container metrics (`StandardBoxScale`) from the item layout metrics (`ListRowScale`).

---

## Phase 5: Rollout to List & Group Collections
Apply structural scaling to composite data rendering surfaces.

### 1. Refactor ListView & ListBox Rows
* **Target Files:**
  * `crates/sdk/src/controls/list_view/theme.rs`
  * `crates/sdk/src/controls/list_view/row.rs`
* **Tasks:**
  * Update row templates to compute `ListRowScale` for heights, horizontal paddings, and alignment baselines.
  * Separate row visual states (hover background, selected highlights) into a theme-resolved `ListViewRowPalette`.

---


