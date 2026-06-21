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
- Support independent thumb sizing with a semantic size vocabulary parallel to `ControlSize`, so callers can tune track size and thumb size separately when needed.
- Keep their internal control structures independent to avoid complex wrapping/orchestration boilerplate.

## Non-Goals
- Do not add color interpolation or color-space specific features directly to the base `Slider` control.
- Do not wrap `SliderControl` inside `ColorSlider`, and do not share templates between them. Keep them as separate, parallel controls.
- Do not add thumb shape enums (like Circle, Square, Bar) to the core `SliderModel`.
- Do not migrate or replace the Theme Studio HS mixer sliders in this pass; validation in the gallery is sufficient.
- Do not remove `color_slider` immediately.
- Do not mix `ColorSlider` demos into the semantic slider pane at `apps/gallery/src/gallery/panes/slider/pane.rs`. Validate semantic `Slider` sizing there, and validate `ColorSlider` separately in its own color/gallery surfaces.

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

### Important sizing implementation note: `style.toml` must be updated
Adding `ControlSize` to the Rust-side `Slider` APIs is **not sufficient** to make the gallery show different slider sizes under the Shadcn look.

The look layer resolves slider metrics from `crates/look-shadcn/assets/style.toml`. If that stylesheet only defines `[slider.metrics.md]`, then `Sm` and `Lg` requests fall back to default metrics and will render the same as medium. This was observed directly during implementation: the gallery pane showed identical `Sm`, `Md`, and `Lg` sliders even after the SDK size plumbing was added, because the look stylesheet did not yet define `sm` and `lg` rules.

That means this issue requires **both**:
1. SDK/control plumbing for `ControlSize`
2. Look stylesheet metric definitions for each slider size

Required Shadcn stylesheet work:
```toml
[slider.metrics.sm]
width = 260.0
height = 24.0
track_height = 4.0
thumb_size = 12.0
radius = "radius.pill"

[slider.metrics.md]
width = 260.0
height = 32.0
track_height = 6.0
thumb_size = 16.0
radius = "radius.pill"

[slider.metrics.lg]
width = 260.0
height = 40.0
track_height = 8.0
thumb_size = 20.0
radius = "radius.pill"
```

Without those `style.toml` entries, the semantic slider gallery validation is misleading because the three sizes will appear identical even if the Rust control APIs are implemented correctly.

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
   * Preserve an **independent thumb sizing control** on both slider families, using a semantic size vocabulary parallel to the main control size (for example `Sm` / `Md` / `Lg` thumb constants), so callers can intentionally choose combinations like a small track with a larger thumb.

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

Also expose independent semantic thumb sizing on both controls:
```rust
pub enum ThumbSize {
    Sm,
    Md,
    Lg,
}

// On SliderBuilder and ColorSliderModel
pub fn thumb_size(mut self, size: ThumbSize) -> Self;
```

This should use size constants parallel to the control size system rather than ad-hoc pixel-only tuning in normal call sites.

### 3. Corner Radii overrides
Expose corner radius overrides on both controls using the same style already used by existing color controls:
```rust
// On SliderBuilder and ColorSliderModel
pub fn corner_radius(mut self, radius: gpui::AbsoluteLength) -> Self;

// On SliderControl and ColorSliderState
pub fn set_corner_radius(&mut self, radius: gpui::AbsoluteLength, cx: &mut Context<Self>);
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
- `crates/look-shadcn/src/controls/slider.rs`
- `crates/look-shadcn/src/controls/templates.rs`
- `crates/look-shadcn/assets/style.toml`

Tasks:
- [ ] Add `Release` event support to `SliderEvent`.
- [ ] Implement `ControlSize` handling and update `SliderTheme` to resolve size-specific metrics.
- [ ] Thread `ControlSize` through the Shadcn look path (`slider_look`, `RadixSliderTheme`, etc.) so the requested size is not discarded.
- [ ] Add `slider.metrics.sm`, `slider.metrics.md`, and `slider.metrics.lg` to `crates/look-shadcn/assets/style.toml` with the intended `24/4/12`, `32/6/16`, and `40/8/20` dimensions.
- [ ] Add custom track corner radii override properties and methods (`corner_radius`, `clear_corner_radius`).
- [ ] Verify that default themed slider behavior is unchanged for existing consumers.

### Phase 2: Refactor `ColorSlider`
Files touched:
- `crates/sdk/src/controls/color/color_slider/*`

Tasks:
- [ ] Replace `color::style::Size` with `ControlSize` and map sizes accordingly.
- [ ] Add independent semantic thumb sizing APIs/constants to both slider families and keep them parallel.
- [ ] Implement the unified corner radius override interfaces in `ColorSliderModel` and `ColorSliderState`.
- [ ] Align helper builder methods to match the SDK conventions used in the generic slider.

### Phase 3: Gallery Validation
Tasks:
- [ ] Update the current slider pane ([pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/slider/pane.rs)) to validate the semantic `Slider` only:
  - [ ] Render standard `Slider` in `Sm`, `Md`, and `Lg` sizes.
  - [ ] Show semantic slider corner radius overrides (e.g. rounded vs sharp).
  - [ ] Confirm the Shadcn look stylesheet metrics actually differ visually after `style.toml` updates.
- [ ] Validate `ColorSlider` sizing and radius behavior in its own color/gallery panes. Do **not** insert `ColorSlider` demos into `apps/gallery/src/gallery/panes/slider/pane.rs`.

---

## Design Constraints
- **Separation of concerns**: Keep the two control codebases fully decoupled to avoid wrapping/orchestration boilerplate.
- **Unified API conventions**: Ensure the control interfaces and methods are named and typed identically.

---

## Acceptance Criteria
- Core `Slider` supports `Release` events, `ControlSize` sizing, independent semantic thumb sizing, and custom corner radius overrides.
- `ColorSlider` supports `ControlSize` sizing, independent semantic thumb sizing, and custom corner radius overrides.
- Sizing, thumb sizing, and corner radius configuration interfaces match across both controls.
- Gallery validates semantic `Slider` sizing in the slider pane and validates `ColorSlider` sizing in separate color/gallery panes without mixing the two control families into one pane.
- All code formats and clippy checks pass.
