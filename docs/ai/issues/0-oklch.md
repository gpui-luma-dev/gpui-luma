# Issue #0-oklch: Migrate Color Representation to Native OKLCH

> [!IMPORTANT]
> **Status & Sequencing**: This issue is **blocked** and will not be considered until [Issue #2: Range-Constrained Slider Control](file:///Users/scg/Developer/GitHub/gpui-luma/docs/ai/issues/2-range-slider.md) is fully completed and verified. The interactive color mixer's gamut-clamping and non-contiguous hue track require the `RangeSlider` component.
> 
> **Color Library Decision**: All native color representation, parsing, and color-space conversions must leverage the standard `palette` crate (already configured in `Cargo.toml`). Avoid handwritten mathematical conversions, matrix multipliers, or custom bisection gamut clamps in the look or SDK layers.

## Description
Migrate the internal color representation, state calculators, and theme studio serialization in the `gpui-luma` workspace from HSL/RGB to native OKLCH.

## Rationale
* **Precision & Consistency**: Avoid precision loss and rounding shifts caused by continuous HSL $\leftrightarrow$ OKLCH round-trip conversions when computing dynamic state highlights (e.g., hover/pressed states) or updating studio overrides.
* **Unified State Calculations**: Lightness adjustments during hover/pressed transitions can be done directly on the lightness (`L`) parameter of OKLCH without complex conversions.
* **Non-Linear Color Mixing**: Support high-fidelity color mixing inside the gallery app using OKLCH, constrained by sRGB/P3 gamut limits.

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

### 6. Implement OKLCH Color Mixer using RangeSliders
In `apps/gallery/src/gallery/panes/color/color_mixer_pane.rs` (or a new color-mixer pane):
* Implement an interactive OKLCH color mixer containing Lightness ($L$), Chroma ($C$), Hue ($H$), and Alpha ($A$) sliders.
* Coordinate the sliders dynamically to prevent out-of-gamut selections:
  * When $L$ or $H$ changes, calculate the maximum displayable Chroma under sRGB/P3 limits and set it as the upper bound of the Chroma slider.
  * When $L$ and $C$ are set, compute the valid, in-gamut intervals of Hue ($0.0 \dots 360.0$).
  * Set these intervals as the `allowed_intervals` on the Hue slider (which will be a `RangeSlider`), so that out-of-gamut hues are visually grayed out (gaps) and cannot be selected by the user.
  * When $C$ and $H$ are set, compute the valid range of Lightness and constrain it.
* Render custom gradients on the tracks that represent the true OKLCH slice, showing the grey gaps where colors clip.

---

## Verification Plan

### Automated Tests
* Run look tests to ensure parser and state resolution are correct:
  ```bash
  cargo test -p gpui-luma-look-shadcn
  ```

### Manual Verification
* Run the Gallery application:
  ```bash
  cargo run -p gpui-luma-gallery
  ```
* Open the **Color Mixer** pane.
* Verify adjusting Lightness or Chroma dynamically updates the Hue `RangeSlider` track gaps.
* Verify dragging the Hue slider snaps over the gaps and never permits selecting an unrenderable/clipped color.
