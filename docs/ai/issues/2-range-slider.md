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

#### Inside Allowed Ranges (Active Segments)
* **Interaction**: The thumb moves continuously and smoothly along the track, following the mouse cursor or stepping by the configured `step` interval.
* **Value & State**: Standard active state. The value updates continuously, and semantic `Change` events are emitted normally.
* **Visuals**: The track is rendered with full color, gradients, or the standard active background.

#### Outside Allowed Ranges (Gaps / Dead Zones)
* **Visuals**: The gap segments are styled as inactive or disabled (e.g., a muted grey or pattern), showing they are unselectable.
* **Mouse Clicks/Dragging**:
  * If a user clicks or drags the cursor over a gap, the thumb **snaps instantly** to the nearest boundary of the closest valid segment.
  * The thumb never visually rests or stops inside a gap.
* **Keyboard Steps**:
  * If the thumb is at the edge of a valid range (e.g., `120.0`) and you press the right arrow key, it **jumps over the gap** directly to the start of the next valid range (e.g., `180.0`).
  * The step size is dynamically adjusted to bridge the gap.
* **Initial/Programmatic Value**:
  * If the slider is initialized or programmatically set to a value inside a gap, it is automatically snapped to the nearest valid interval edge upon model configuration.
* **Dynamic Range Changes**:
  * If the allowed intervals are updated dynamically (e.g., when another channel like Lightness changes in an OKLCH color mixer) and the current value falls into a newly formed gap, the control must instantly migrate/snap its value to the closest available valid interval boundary, even if the control is not active or currently being interacted with.

### 3. Track Visual Template
* The track is rendered (via Canvas or sub-segmented divs) to show:
  * **Active segments**: rendered with the normal track fill, gradient, or color spectrum.
  * **Gap segments**: rendered with a disabled/inactive style (e.g., dark grey background, no gradient).

---

## Value Snapping & Constraint Strategy

To prevent ad-hoc snapping, stepping, and gap-jumping math from cluttering `RangeSliderControl` and event handlers, all value-constraint logic is encapsulated in a dedicated **Constraint Strategy**:

```rust
pub trait SliderConstraintStrategy: Send + Sync {
    /// Constrains a raw target value (e.g. from mouse click/drag or setter)
    /// to the allowed intervals, snapping it to the nearest valid point.
    fn clamp_and_snap(
        &self,
        value: f32,
        allowed_intervals: &[std::ops::RangeInclusive<f32>],
        range: std::ops::RangeInclusive<f32>,
        step: Option<f32>,
    ) -> f32;

    /// Computes the new value when stepping (e.g. via keyboard arrow keys),
    /// skipping entirely over gaps to land on the next valid interval boundary.
    fn step_value(
        &self,
        current: f32,
        step_delta: f32,
        allowed_intervals: &[std::ops::RangeInclusive<f32>],
        range: std::ops::RangeInclusive<f32>,
        step: Option<f32>,
    ) -> f32;
}
```

### Strategy Implementations
* **`IntervalConstraintStrategy`**: The default strategy for `RangeSlider`. It implements interval snapping (finding the closest range, clamping bounds, and stepping across dead zones).
* **`ContinuousConstraintStrategy`**: A fallback strategy used when `allowed_intervals` is empty, behaving like a standard continuous slider.

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
