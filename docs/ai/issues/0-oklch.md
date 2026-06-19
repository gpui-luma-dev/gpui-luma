# Issue #0-oklch: Migrate Color Representation to Native OKLCH

## Description
Migrate the internal color representation, state calculators, and theme studio serialization in the `gpui-luma` workspace from HSL/RGB to native OKLCH. 

## Rationale
* **Precision & Consistency**: Avoid precision loss and rounding shifts caused by continuous HSL $\leftrightarrow$ OKLCH round-trip conversions when computing dynamic state highlights (e.g., hover/pressed states) or updating studio overrides.
* **Unified State Calculations**: Lightness adjustments during hover/pressed transitions can be done directly on the lightness (`L`) parameter of OKLCH without complex conversions.
* **Preparation for Color Sliders**: Prepares the codebase for Issue #1, which introduces first-class OKLCH track rendering and interpolation.

---

## Proposed Tasks

### 1. Introduce Canonical OKLCH Types
In [color.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/color.rs), define a wrapper type that represents OKLCH with alpha using the `palette` crate.
* Wrap `palette::Alpha<palette::Oklch, f32>` in a custom struct (e.g., `OklchColor`) to handle conversion and serialization.
* Implement `From<OklchColor> for gpui::Hsla` for rendering compatibility at look boundaries.

### 2. Update Catalog Storage
In [catalog/mod.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/catalog/mod.rs):
* Update [CssTokenMap](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/catalog/mod.rs#L14) to parse and store values internally as `OklchColor` instead of `gpui::Hsla`.
* Update `color`, `color_first`, and `optional_color` to parse custom properties directly into `OklchColor`.

### 3. Simplify State Color Calculations
In [state_color.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/state_color.rs):
* Update [StateColorTable](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/state_color.rs#L15) and cache resolved state colors as `OklchColor`.
* Simplify [algorithmic_state_color](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/state_color.rs#L60) to modify the `L` parameter directly in OKLCH space, removing the conversion round-trip.

### 4. Update Override Serialization
In [look.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/look.rs):
* Modify [hsla_to_css_value](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/look.rs#L740) (or rename to `oklch_to_css_value`) to serialize overrides using the native `oklch(L C H / alpha)` format instead of `hsl()`.
* Update color override hash maps to store `OklchColor` or perform conversions when applying overrides.

### 5. Standardize Theme Assets
* Convert HSL color tokens in [native.css](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/assets/native.css) and key built-in theme files to `oklch(...)`.

---

## Verification Plan

### Automated Tests
* Run look tests to ensure parser and state resolution are correct:
  ```bash
  cargo test -p gpui-luma-look-shadcn
  ```

### Manual Verification
* Run the Theme Studio application:
  ```bash
  cargo run -p gpui-luma-theme-studio
  ```
* Verify adjusting temperature/vividness sliders works smoothly and does not trigger visual regressions or assertions.
