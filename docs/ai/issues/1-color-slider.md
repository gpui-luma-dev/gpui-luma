# Issue #1: Upgrade Color Slider to the New Control Model

## Description
The workspace currently has two slider families with overlapping responsibilities:

- `crates/sdk/src/controls/slider/*`
  - newer LMTP-style control family
  - look-integrated via `ShadcnLookControlExt`
  - used by app surfaces like theme studio
- `crates/sdk/src/controls/color/color_slider/*`
  - stronger color-spectrum behavior
  - better event semantics for interactive color work
  - older, separate control architecture

We should rationalize this by upgrading the main SDK slider model so it can support both:

- normal semantic sliders
- color-spectrum / gradient sliders

The immediate validation target is:

1. gallery app coverage for the upgraded control
2. replacing the two `HS MIXER` sliders in theme studio with color-spectrum sliders

Related mixer module:

- `apps/theme-studio/src/studio/hs_mixer.rs`

That file already makes OKLCH-based hue/chroma palette generation first-class for the theme studio mixer. The slider upgrade should treat OKLCH as a first-class color model for color controls as well.

## Goals
- Keep `controls::slider` as the canonical SDK slider family.
- Port the useful behavior from `controls::color::color_slider` into the main slider architecture.
- Make OKLCH a first-class option in color slider tracks and delegates.
- Avoid leaving the product with two unrelated slider stacks long-term.
- Preserve the current SDK architecture principles from `docs/architecture.md`.

## Non-Goals
- Do not rewrite all color controls in one pass.
- Do not remove `color_slider` immediately.
- Do not block the gallery or theme-studio work on perfect final deprecation.

## Current State

### Main slider strengths
Files:
- `crates/sdk/src/controls/slider/model.rs`
- `crates/sdk/src/controls/slider/control.rs`
- `crates/sdk/src/controls/slider/template.rs`
- `crates/sdk/src/controls/slider/theme.rs`

Pros:
- follows LMTP split
- look-themed
- already used in apps cleanly
- good default product control foundation

Weaknesses:
- event model is too simple for color editing
- no built-in gradient / spectrum track model
- no first-class color interpolation / track delegate abstraction

### Color slider strengths
Files:
- `crates/sdk/src/controls/color/color_slider/model.rs`
- `crates/sdk/src/controls/color/color_slider/slider.rs`
- `crates/sdk/src/controls/color/color_slider/delegates.rs`
- `crates/sdk/src/controls/color/color_slider/visual.rs`

Pros:
- `Change` and `Release` events
- track delegates
- hue spectrum support
- arbitrary gradient support
- interpolation modes: RGB / HSL / Lab

Weaknesses:
- separate control family
- not aligned with the newer standard slider architecture
- not naturally look-integrated the same way the main slider is
- OKLCH is not first-class in the slider API today

## Proposed Direction: Layered Composition

To prevent polluting the generic `Slider` with color-specific logic (e.g. color interpolation, spaces like OKLCH/Lab, checkerboard rendering, etc.), we adopt a **layered composition architecture**.

1. **Keep `Slider` color-blind**: The core `Slider` will remain completely unaware of colors. We will upgrade it with the minimum generic capabilities needed to support advanced template overrides:
   - Add `Release` events.
   - Accept arbitrary custom template configurations (or hooks for custom track rendering).
2. **Re-implement `ColorSlider` as a facade/custom template**: Keep `ColorSlider` as a distinct control API under `crates/sdk/src/controls/color/color_slider`. Under the hood, it wraps or configures a generic `Slider` but sets a specialized `ColorSliderTemplate` to handle the rendering of spectrums, checkerboards, gradients, and custom colored thumbs.
3. **OKLCH and Color Math encapsulation**: Keep all color interpolation algorithms, color specification models, and delegates isolated in the color sub-module.

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

### 2. Custom Track/Thumb Hooks in `Slider`
To allow color spectrum rendering without making `Slider` color-aware, the slider's layout & templates should allow custom track backgrounds and thumb customizations.

Possible approach:
- Enhance `SliderTemplate` (or introduce a track-rendering delegate slot in `SliderModel`) so that custom visual templates (like `ColorSliderTemplate`) can take control of painting the track background, track overlays, and thumb color/shape while relying on `SliderControl` for input bounds, drag-movement, and keyboard focus.

### 3. First-class Color Interpolation in `ColorSlider`
The `ColorSlider` delegates and templates will support OKLCH alongside existing HSL, RGB, and Lab modes:

```rust
pub enum ColorInterpolation {
    Rgb,
    Hsl,
    Lab,
    Oklch,
}
```

### 4. Gradient Stop and Checkerboard Support
Implement these purely within the custom `ColorSliderTemplate` and delegates:
- Checkerboards for alpha/opacity channels.
- Multicolored stops and spectrum segments with sub-pixel overlapping drawing.

---

## OKLCH First-Class Work
- Port the OKLCH interpolation algorithm to `color_spec` in the color module.
- Allow the theme studio mixer (`hs_mixer.rs`) to drive the color slider templates with OKLCH spectrum tracks.
- Show an OKLCH-based spectrum comparison in the gallery.

---

## Migration Plan

### Phase 1: Upgrade main slider internals (Color-Blind)
Files touched:
- `crates/sdk/src/controls/slider/model.rs`
- `crates/sdk/src/controls/slider/control.rs`
- `crates/sdk/src/controls/slider/template.rs`

Tasks:
- [ ] Add `Release` event support to `SliderEvent`.
- [ ] Add custom track / custom template hooks to allow full visual control of the track and thumb.
- [ ] Verify that default themed slider behavior is unchanged for existing consumers.

### Phase 2: Refactor `ColorSlider` Internals
Files touched:
- `crates/sdk/src/controls/color/color_slider/*`

Tasks:
- [ ] Create `ColorSliderTemplate` implementing `SliderTemplate` to handle the color drawing.
- [ ] Re-implement `ColorSliderState` / `ColorSlider` to compose a nested `SliderControl` configured with the new template.
- [ ] Keep the public API of `ColorSlider` backward-compatible.
- [ ] Add `Oklch` to `ColorInterpolation` and implement its interpolation algorithm.

### Phase 3: Gallery Validation
Tasks:
- [ ] Add gallery examples showing:
  - [ ] Default slider still works perfectly.
  - [ ] Composed `ColorSlider` rendering a Hue spectrum.
  - [ ] Composed `ColorSlider` rendering a checkerboard alpha spectrum.
  - [ ] Composed `ColorSlider` using OKLCH interpolation.

### Phase 4: Replace Theme Studio HS Mixer Sliders
Target files:
- `apps/theme-studio/src/studio/hs_mixer.rs`
- `apps/theme-studio/src/studio/theme_sidebar/panels/other.rs` (and related sync code)

Tasks:
- [ ] Replace the two old mixer sliders with the composed `ColorSlider`.
- [ ] Temperature slider uses warm ↔ cool spectrum driven by `hs_mixer.rs`.
- [ ] Vividness slider uses neutral ↔ vivid spectrum driven by `hs_mixer.rs`.

---

## Design Constraints
- **Separation of concerns**: The core `Slider` must not have any dependencies on color spaces, checkerboards, or color conversion libraries.
- **Maintain LMTP split**: The templates should remain stateless GPUI renderers, resolving styles from themes/delegates.
- Keep app-level custom spectrum calculations in `hs_mixer.rs`.

---

## Risks
- Sub-pixel overlapping drawing trickiness needs to translate cleanly into the custom `ColorSliderTemplate`.
- Custom track bounds and dragging mechanics in `SliderControl` must properly align with the track insets of the color slider.

---

## Acceptance Criteria
- Core `Slider` supports `Release` events and custom templates.
- `ColorSlider` wraps `Slider` and renders checkerboards, hue spectrums, and OKLCH gradients.
- Theme studio mixer uses the composed OKLCH color sliders.
- Gallery validates both normal and color sliders.
- All code formats and clippy checks pass.
