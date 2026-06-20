# Issue #1: Upgrade Color Slider to the New Control Model

## Description
The workspace currently has two slider families with overlapping responsibilities:

- `crates/sdk/src/controls/slider/*`
  - newer LMTP-style control family
  - look-integrated via `ShadcnLookControlExt`
- `crates/sdk/src/controls/color/color_slider/*`
  - stronger color-spectrum behavior
  - better event semantics for interactive color work
  - older, separate control architecture

We should rationalize this by aligning their control interfaces and builder APIs to be as parallel as possible, unifying their sizing and styling systems, while keeping their underlying implementations decoupled.

## Goals
- Align the public API and builder interfaces of both `Slider` and `ColorSlider` for SDK consistency.
- Port the useful event behavior (like `Release` events) to the main `Slider` architecture.
- Share a unified semantic sizing system (`ControlSize`) and a custom corner radius override system across both semantic sliders and color sliders.
- Keep their internal control structures independent to avoid complex wrapping/orchestration boilerplate.

## Non-Goals
- Do not add color interpolation or color-space specific features directly to the base `Slider` control.
- Do not wrap `SliderControl` inside `ColorSlider`, and do not share templates between them. Keep them as separate, parallel controls.
- Do not add thumb shape enums (like Circle, Square, Bar) to the core `SliderModel`.
- Do not migrate or replace the Theme Studio HS mixer sliders in this pass; validation in the gallery is sufficient.
- Do not remove `color_slider` immediately.

---

## Current Sizing & Styling Analysis

Currently, the two slider families have disconnected sizing and styling systems:

### 1. Sizing Systems
* **Semantic `Slider`**: Has no size options. It hardcodes standard dimensions in `DefaultSliderTheme` (width: 260px, height: 32px, track: 8px, thumb: 18px).
* **`ColorSlider`**: Defines a custom size enum (`XSmall`, `Small`, `Medium`, `Large`, `Size(Pixels)`) and maps them to custom track thicknesses and thumb sizes.

### 2. Corner Radii Systems
* **Semantic `Slider`**: Resolves a single static corner radius (`look.radius`) from the theme (usually `radius.pill`). Callers cannot override specific corners (e.g. to make a square-edged track).
* **`ColorSlider`**: Supports setting explicit corner radii overrides on the control and state:
  ```rust
  pub fn set_corner_radius(&mut self, radius: AbsoluteLength, cx: &mut Context<Self>);
  pub fn clear_corner_radius(&mut self, cx: &mut Context<Self>);
  ```
  This is useful for creating sharp block-style gradients or custom edge designs.

### Harmonization Plan
We will align both sliders to share `ControlSize` and custom corner radius overrides:

1. **Unify Sizing under `ControlSize`**:
   * Add `.size(ControlSize)` to `SliderBuilder`.
   * Update `SliderTheme` to resolve size-specific dimensions:
     * `Sm`: height: `24px`, track_height: `4px`, thumb_size: `12px`
     * `Md` (Default): height: `32px`, track_height: `6px`, thumb_size: `16px`
     * `Lg`: height: `40px`, track_height: `8px`, thumb_size: `20px`
   * Harmonize `ColorSlider` to use `ControlSize` but preserve its thicker tracks designed to display gradients:
     * `Sm`: track: `12px`, thumb: `16px`
     * `Md` (Default): track: `20px`, thumb: `24px`
     * `Lg`: track: `30px`, thumb: `32px`

2. **Unify Corner Radii**:
   * Port the corner radius override properties into `SliderModel` and `SliderBuilder`.
   * Expose `.corner_radius(...)` and `.clear_corner_radius()` on both `SliderBuilder` / `SliderControl` and `ColorSliderModel` / `ColorSliderState`.
   * Update the standard `ThemedSliderTemplate` to apply these overrides when drawing the track.

---

## Proposed Direction: Parallel Independent Controls

Rather than wrapping the generic slider or sharing templates, `ColorSlider` and `Slider` will remain separate control architectures. This avoids wrapping overhead while ensuring they present a unified user-facing API:

1. **Parallel Builders**: Both controls will follow the standard builder-to-spawn pattern, exposing parallel sizing and corner styling methods.
2. **Parallel Event Models**: Both controls will emit comparable semantic events (`Change` and `Release`), allowing downstream apps to consume them interchangeably.

---

## Proposed API Additions

### 1. Richer slider events
Add release / commit semantics to the core `Slider` to support live preview on drag and state-sync/commit on release.

```rust
pub enum SliderEvent {
    Change { value: f32 },
    Release { value: f32 },
}
```

### 2. Sizing configuration
Expose standard sizing builders on both controls:
```rust
// On SliderBuilder and ColorSliderModel
pub fn size(mut self, size: ControlSize) -> Self;
```

### 3. Corner Radii overrides
Expose corner radius overrides on both controls:
```rust
// On SliderBuilder and ColorSliderModel
pub fn corner_radius(mut self, radius: gpui::CornerRadius) -> Self;

// On SliderControl and ColorSliderState
pub fn set_corner_radius(&mut self, radius: gpui::CornerRadius, cx: &mut Context<Self>);
pub fn clear_corner_radius(&mut self, cx: &mut Context<Self>);
```

---

## Migration Plan

### Phase 1: Upgrade main slider internals (Color-Blind)
Files touched:
- `crates/sdk/src/controls/slider/model.rs`
- `crates/sdk/src/controls/slider/control.rs`
- `crates/sdk/src/controls/slider/template.rs`
- `crates/sdk/src/controls/slider/theme.rs`

Tasks:
- [ ] Add `Release` event support to `SliderEvent`.
- [ ] Implement `ControlSize` handling and update `SliderTheme` to resolve size-specific metrics.
- [ ] Add custom track corner radii override properties and methods (`corner_radius`, `clear_corner_radius`).
- [ ] Verify that default themed slider behavior is unchanged for existing consumers.

### Phase 2: Refactor `ColorSlider`
Files touched:
- `crates/sdk/src/controls/color/color_slider/*`

Tasks:
- [ ] Replace `color::style::Size` with `ControlSize` and map sizes accordingly.
- [ ] Implement the unified corner radius override interfaces in `ColorSliderModel` and `ColorSliderState`.
- [ ] Align helper builder methods to match the SDK conventions used in the generic slider.

### Phase 3: Gallery Validation
Tasks:
- [ ] Update the current slider pane ([pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/slider/pane.rs)) to display coordinated size comparisons:
  - [ ] Render both standard `Slider` and `ColorSlider` side-by-side (or stacked) in `Sm`, `Md`, and `Lg` sizes, demonstrating coordinated visual alignments.
  - [ ] Show custom corner radius overrides (e.g. rounded vs sharp) on both controls.
  - [ ] Show a checkerboard alpha color slider.

---

## Design Constraints
- **Separation of concerns**: Keep the two control codebases fully decoupled to avoid wrapping/orchestration boilerplate.
- **Unified API conventions**: Ensure the control interfaces and methods are named and typed identically.

---

## Acceptance Criteria
- Core `Slider` supports `Release` events, `ControlSize` sizing, and custom corner radius overrides.
- `ColorSlider` supports `ControlSize` sizing and custom corner radius overrides.
- Sizing and corner radius configuration interfaces match across both controls.
- Gallery validates both normal and color sliders across all size and radius variants.
- All code formats and clippy checks pass.
