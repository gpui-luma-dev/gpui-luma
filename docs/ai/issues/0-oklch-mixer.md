# Issue #0-oklch-mixer: Gamut-Clamped OKLCH Color Mixer Specification

## Goal
Implement an interactive, gamut-clamped OKLCH color mixer pane in the multi-mixer demo section of the gallery. This mixer will serve as the stress-test for the new unified `Slider` engine's advanced features, specifically:
- Non-contiguous `allowed_intervals` (blocked track gaps).
- Dynamic track rendering delegates (`DomainTrackRenderer`) that paint real-time color slices and grayed-out out-of-gamut segments.
- Standard slider thumbs matching the styling of the other color mixers.

---

## UI Layout & Interactive Behavior
The mixer will consist of four stacked sliders matching the reference design:

```
+-------------------------------------------------------------+
| Lightness                                                   |
| [=== Blocked ===][============ Active Gradient ========o====]  (0.0..=1.0)
|                                                             |
| Chroma                                                      |
| [============= Active Gradient =======o][===== Blocked =====]  (0.0..=0.4)
|                                                             |
| Hue                                                         |
| [= Active =][== Blocked ==][= Active =][== Blocked ==]  o   ]  (0.0..=360.0)
|                                                             |
| Alpha                                                       |
| [=================== Alpha Transparency Gradient ======o====]  (0.0..=1.0)
+-------------------------------------------------------------+
```

### 1. Slider Parameters & Bounds

| Parameter | Range | Visual Track | Behavior & Gamut Constraints |
| :--- | :--- | :--- | :--- |
| **Lightness ($L$)** | `0.0..=1.0` | In-gamut gradient slice to black/white. | Capped dynamically at the bottom/top where $L$ cannot support the current Chroma ($C$) within the sRGB gamut. Out-of-gamut ranges are grayed out and blocked. |
| **Chroma ($C$)** | `0.0..=0.4` | Grey-to-chroma gradient. | Dynamically caps the upper bounds. Any Chroma value higher than the maximum displayable Chroma at the current $L$ and $H$ is marked as a blocked segment. |
| **Hue ($H$)** | `0.0..=360.0` | Full spectrum slice. | Uses non-contiguous `allowed_intervals` representing the disjointed hue angles that remain in-gamut for the current $L$ and $C$. Blocked sections are colored neutral grey and skipped by pointer/keyboard. |
| **Alpha ($A$)** | `0.0..=1.0` | Checkerboard + alpha gradient. | Standard transparency control. |

### 2. Thumb Styling
- All four sliders will configure the standard thumb shapes matching the other color mixers in the gallery (e.g. circle thumb with a border and filled color preview center).

---

## Gamut Coordination Logic

We will leverage the `palette` crate for sRGB boundary checks:

1. **Gamut Check Helper**:
   Convert the candidate `Oklch` color to `Srgb` and check if it is within bounds:
   ```rust
   fn is_in_gamut(l: f32, c: f32, h: f32) -> bool {
       use palette::{FromColor, Oklch, Srgb};
       let oklch = Oklch::new(l, c, h);
       let srgb = Srgb::from_color(oklch);
       srgb.is_within_bounds()
   }
   ```

2. **Hue Interval Solver**:
   Scan Hue from `0.0` to `360.0` (at `1.0`-degree steps) for the current $L$ and $C$. Group contiguous true results into a `Vec<RangeInclusive<f32>>` to set as `allowed_intervals` on the Hue slider:
   ```rust
   fn calculate_in_gamut_hues(l: f32, c: f32) -> Vec<RangeInclusive<f32>> {
       let mut intervals = Vec::new();
       let mut start: Option<f32> = None;

       for h in 0..=360 {
           let h_f32 = h as f32;
           let in_gamut = is_in_gamut(l, c, h_f32);
           
           match (in_gamut, start) {
               (true, None) => start = Some(h_f32),
               (false, Some(s)) => {
                   intervals.push(s..=(h_f32 - 1.0));
                   start = None;
               }
               _ => {}
           }
       }
       
       if let Some(s) = start {
           intervals.push(s..=360.0);
       }
       intervals
   }
   ```

3. **Lightness Interval Solver**:
   Similarly compute contiguous in-gamut intervals of Lightness ($0.0 \dots 1.0$) for the current $C$ and $H$.

4. **Maximum Chroma Solver**:
   Binary search or step-scan to find the maximum Chroma at the current $L$ and $H$ where the color is still in gamut, then cap the Chroma slider's active region there.

---

## Proposed Tasks

### 1. Implement Track Renderers
Define custom `DomainTrackRenderer` delegates in the color controls:
- **`OklchLightnessTrackRenderer`**: Paints the lightness spectrum slice, applying a grey fill on out-of-gamut segments.
- **`OklchChromaTrackRenderer`**: Paints grey to fully saturated chroma gradient, graying out above the maximum in-gamut chroma limit.
- **`OklchHueTrackRenderer`**: Paints the circular hue spectrum, using the slider's `track_segments` to paint the active/blocked partitions differently (blocked partitions colored neutral grey).

### 2. Scaffold the Demo Pane
Create `apps/gallery/src/gallery/panes/color_compositions/compositions/oklch_mixer.rs`:
- Instantiate the four sliders via `ColorSliderBuilder` or generic `SliderBuilder` with custom look templates.
- Define a state manager holding the active `Oklch { l, c, h, a }` state.
- Subscribe to `SliderEvent::Change` and `SliderEvent::Release` events on all four sliders.
- In the event handlers, update the state, recalculate gamut boundaries/intervals, and invoke `set_allowed_intervals` and `set_domain_track` to refresh the sister sliders.

### 3. Add to Gallery Menu
Register the new OKLCH Mixer pane inside the gallery compositions list so users can inspect and test the interactive clipping behavior.
