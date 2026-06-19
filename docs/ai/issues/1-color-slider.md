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

## Proposed Direction
Upgrade `crates/sdk/src/controls/slider/*` so it absorbs the capabilities needed for color-spectrum sliders.

### Principle
Do **not** make `color_slider` the new universal base control.

Instead:
- keep `controls::slider` as the canonical slider primitive
- port over the missing capabilities from `color_slider`
- later migrate app usage and possibly deprecate the old color slider family

## Proposed API Additions

### 1. Richer slider events
Current:
- `SliderEvent::Change { value }`

Target:
- add release / commit semantics

Possible shape:
```rust
pub enum SliderEvent {
    Change { value: f32 },
    Release { value: f32 },
}
```

Or a phased event shape if preferred.

Why:
- color editing often wants live preview on drag
- some sync/export work should happen on release only
- this matches the current color slider behavior

### 2. Track delegate / track renderer model
Add a track abstraction to the main slider.

Possible concepts:
- `SliderTrackDelegate`
- `SliderTrackVisual`
- `SliderTrackMode`

Required capabilities:
- plain themed rail/fill
- arbitrary gradient track
- hue spectrum track
- computed color-at-position

This should live in the main slider control family rather than in ad-hoc app code.

### 3. First-class color interpolation
Current color slider supports:
- RGB
- HSL
- Lab

Upgrade target should add:
- OKLCH

Possible enum:
```rust
pub enum SliderColorInterpolation {
    Rgb,
    Hsl,
    Lab,
    Oklch,
}
```

Reason:
- `hs_mixer.rs` already derives palette behavior from OKLCH-like hue/chroma thinking
- future color controls should not treat OKLCH as a second-class conversion step

### 4. Gradient stop model
The main slider should support explicit color stops.

Possible shape:
```rust
pub struct SliderColorStop {
    pub position: f32,
    pub color: Hsla,
}
```

Needed for:
- warm ↔ cool tracks
- neutral ↔ vivid tracks
- arbitrary palette ramps
- color controls beyond hue sliders

### 5. Optional built-in spectrum modes
Convenience helpers on the main slider would reduce app wiring.

Examples:
- hue spectrum
- custom gradient
- alpha-over-checkerboard

These should still compile down to the same delegate/track system.

## OKLCH First-Class Work
This should be added now while upgrading the slider.

### Requirements
- track interpolation mode supports OKLCH
- helper constructors can derive gradient stops in OKLCH
- any future color-spec system should be able to expose OKLCH channels directly
- avoid baking HSL-only assumptions into the upgraded slider API

### Minimum OKLCH scope for this issue
- add OKLCH interpolation support in upgraded slider track logic
- allow `hs_mixer.rs`-driven sliders to generate visually meaningful spectrum tracks
- ensure gallery has at least one OKLCH-based example or comparison pane

## Migration Plan

### Phase 1: Upgrade main slider internals
Files likely touched:
- `crates/sdk/src/controls/slider/model.rs`
- `crates/sdk/src/controls/slider/control.rs`
- `crates/sdk/src/controls/slider/template.rs`
- `crates/sdk/src/controls/slider/theme.rs`
- possibly new helper files under `crates/sdk/src/controls/slider/`

Tasks:
- [ ] add `Release` event support
- [ ] add track delegate / gradient support
- [ ] add color interpolation enum with OKLCH
- [ ] add gradient stop model
- [ ] keep default themed slider behavior unchanged for non-color consumers

### Phase 2: Gallery validation
Use the gallery as the primary proving ground.

Tasks:
- [ ] add gallery examples showing:
  - [ ] default slider behavior still works
  - [ ] hue spectrum slider in upgraded main slider
  - [ ] arbitrary gradient slider in upgraded main slider
  - [ ] OKLCH interpolation example
- [ ] compare old `color_slider` behavior versus upgraded `slider` where useful
- [ ] verify drag, release, keyboard, and focus behavior

Good existing references:
- `apps/gallery/src/gallery/panes/color/hsv_plane_pane.rs`
- `apps/gallery/src/gallery/panes/color/multi_mixer_pane.rs`
- `apps/gallery/src/gallery/panes/slider/*`

### Phase 3: Replace theme studio HS mixer sliders
Target files:
- `apps/theme-studio/src/studio/theme_sidebar/model.rs`
- `apps/theme-studio/src/studio/theme_sidebar/mod.rs`
- `apps/theme-studio/src/studio/theme_sidebar/subscriptions.rs`
- `apps/theme-studio/src/studio/theme_sidebar/sync.rs`
- `apps/theme-studio/src/studio/theme_sidebar/panels/other.rs`
- `apps/theme-studio/src/studio/hs_mixer.rs`

Tasks:
- [ ] replace the two `HS MIXER` generic sliders with upgraded spectrum sliders
- [ ] temperature slider should show warm ↔ cool color track
- [ ] vividness slider should show neutral ↔ vivid track
- [ ] track generation should be driven from `hs_mixer.rs`
- [ ] preserve the current layout and width behavior already tuned in the sidebar

### Phase 4: Evaluate deprecation path
Tasks:
- [ ] decide whether `controls/color/color_slider` remains for advanced picker workflows only
- [ ] or start migrating it behind the main slider model
- [ ] document the intended long-term ownership boundary

## Design Constraints
- Maintain LMTP split.
- Do not push app-specific palette math into the base slider control.
- `hs_mixer.rs` should remain the theme-studio-specific source of hue/chroma spectrum generation.
- The main slider should expose generic track/delegate primitives, not hardcode theme-studio logic.
- Keep app usage on SDK controls; do not invent raw div-based sliders in apps.

## Risks
- Event compatibility changes may affect existing slider consumers.
- Rendering custom gradients in the main slider template may complicate the simple themed fill rail path.
- OKLCH interpolation may need careful gamut handling to avoid ugly transitions.
- There may be overlap with existing `color_slider` delegate concepts that should be unified deliberately, not duplicated.

## Acceptance Criteria
- Upgraded `controls::slider` supports release/commit-style events.
- Upgraded `controls::slider` supports spectrum / gradient tracks.
- OKLCH is a first-class interpolation option in the upgraded slider path.
- Gallery demonstrates the upgraded control clearly.
- Theme studio `HS MIXER` uses the upgraded color-spectrum slider for its two rows.
- Existing non-color slider behavior remains intact.
- `cargo fmt`, `cargo clippy`, and relevant gallery/theme-studio validation pass.
