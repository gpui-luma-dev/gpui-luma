# Issue #2: Range-Constrained Slider Control

## Description
In perceptually uniform color spaces like OKLCH, the gamut space is non-linear and irregular. Shifting one channel (like Lightness or Chroma) dramatically alters the valid range of the other channels (especially Hue). 

Under certain configurations (e.g., high Chroma), some sections of the Hue spectrum become entirely out-of-gamut (unrenderable in sRGB/P3) resulting in non-contiguous "gaps" on the slider track.

To prevent the user from selecting unrenderable colors and to visually communicate the gamut constraints, we need a **Range-Constrained Slider** (or `RangeSlider`). This control restricts the thumb's value and movement to a set of valid, non-contiguous intervals (segments), visually greying out and skipping over the "gaps."

---

## Core Features & Requirements

### 1. Interval-Based Constraints
* The slider value must be constrained to a list of allowed ranges:
  ```rust
  pub struct RangeInterval {
      pub start: f32,
      pub end: f32,
  }
  ```
* The control holds a `allowed_intervals: Vec<RangeInterval>` list.
* If `allowed_intervals` is empty, it behaves as a standard slider with a single continuous range.

### 2. Interaction & Snapping Logic
* **Dragging/Clicking**: When the user drags or clicks the mouse, the computed track percentage is mapped to the slider's coordinate space. If the target value falls into a gap:
  * Snap the value to the closest boundary (`start` or `end`) of the nearest valid interval.
* **Keyboard Navigation**: Arrow keys skip over the gaps, jumping directly from the end of one valid interval to the start of the next.

### 3. Track Visual Template
* The track is rendered (via Canvas or sub-segmented divs) to show:
  * **Active segments**: rendered with the normal track fill, gradient, or color spectrum.
  * **Gap segments**: rendered with a disabled/inactive style (e.g., dark grey background, no gradient).

---

## Proposed API & Control Model

### Model & Builder
```rust
pub struct RangeSliderModel {
    pub(crate) id: SharedString,
    pub(crate) value: f32,
    pub(crate) range: std::ops::RangeInclusive<f32>,
    pub(crate) allowed_intervals: Vec<std::ops::RangeInclusive<f32>>,
    pub(crate) size: ControlSize,
    pub(crate) orientation: SliderOrientation,
    pub(crate) template: Arc<dyn RangeSliderTemplate>,
}

pub struct RangeSliderBuilder {
    model: RangeSliderModel,
}

impl RangeSliderBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self;
    pub fn range(mut self, range: std::ops::RangeInclusive<f32>) -> Self;
    pub fn allowed_intervals(mut self, intervals: Vec<std::ops::RangeInclusive<f32>>) -> Self;
    pub fn value(mut self, value: f32) -> Self;
    pub fn size(mut self, size: ControlSize) -> Self;
    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<RangeSliderControl>;
}
```

### Control Entity
```rust
pub struct RangeSliderControl {
    model: RangeSliderModel,
    // coordinates interaction, focus, and pointer bounds...
}

impl RangeSliderControl {
    pub fn value(&self) -> f32;
    pub fn set_value(&mut self, value: f32, cx: &mut Context<Self>);
    pub fn set_allowed_intervals(&mut self, intervals: Vec<std::ops::RangeInclusive<f32>>, cx: &mut Context<Self>);
}
```

---

## Gallery Demo Pane: Range Slider Showcase

To validate the implementation, we will add a new pane in the gallery app: `apps/gallery/src/gallery/panes/slider/range_slider_pane.rs`.

### Showcase Features
1. **Mock Gamut Hue Slider**:
   * A simulated Hue slider ($0.0 \dots 360.0$) configured with two gaps:
     * Allowed intervals: `[0.0..=120.0, 180.0..=240.0, 300.0..=360.0]`
     * Track rendered as a custom color spectrum only in the allowed sections, with dark grey blocks in the gaps.
     * Demonstrates that dragging the thumb jumps/skips over the gaps cleanly.
2. **Interactive Gaps Customizer**:
   * Controls below the slider allow the user to dynamically adjust the gaps and watch the track rerender and snap the thumb to new valid bounds in real time.
3. **Keyboard focus validation**:
   * Pressing arrow keys steps the slider and showcases jumping over the dead zones.

---

## Acceptance Criteria
- `RangeSlider` constraints restrict values strictly to the configured `allowed_intervals`.
- Mouse dragging and clicking snaps to the nearest valid interval edge rather than landing in gaps.
- Keyboard navigation skips gaps.
- Gallery range slider pane successfully demonstrates Hue spectrum gaps and snapping behavior.
- Clean separation from standard `Slider` (reusing common sub-components where appropriate).
