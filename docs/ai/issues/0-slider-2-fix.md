# Slider 2 Starting State Fix: ColorSliderBuilder Channel Ranges

This document details the coordination breakdown between the `ColorRing` and the linked HSV color sliders in the interactive sample, along with the precise fix.

---

## 1. Problem Diagnosis
* **Unified Slider Defaults**: The base `Slider` control defaults to `range: 0.0..100.0` with `step: 1.0`.
* **Builder Range Omission**: While specific builders like `.hue()` configure correct ranges, `.channel()` (used for `.saturation()` and `.lightness()`) only calls `Self::new(...)` without setting the range.
* **Range Mismatch**: The sliders remain at `0.0..100.0` but the HSV state / `ColorRing` expects `0.0..1.0`.
  * **Slider -> Ring Sync**: Setting a slider to $50\%$ sends `50.0` to the ring, which clamps it to `1.0`.
  * **Ring -> Slider Sync**: Setting a ring value to `0.5` sets the slider to `0.5`, placing the thumb at the far left (`0.5%` of `100.0`).

---

## 2. Solution

Configure [ColorSliderBuilder::channel](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/color/color_slider/builder.rs#L63-L71) to automatically retrieve the min, max, and step fields from the specification's `ColorChannel` metadata.

### Step-by-Step Code Fix

In [builder.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/color/color_slider/builder.rs#L63-L71):

```rust
    pub fn channel<S: ColorSpecification>(
        id: impl Into<SharedString>,
        value: f32,
        spec: S,
        channel_name: impl Into<SharedString>,
    ) -> Result<Self, String> {
        let channel_name = channel_name.into();
        let delegate = ChannelDelegate::new(spec, channel_name.clone())?;
        
        let channel = spec
            .channels()
            .iter()
            .find(|c| c.name == channel_name.as_ref())
            .ok_or_else(|| format!("Channel '{}' not found", channel_name))?;
        let step = super::color_spec::slider_step_for_channel(channel);

        Ok(Self::new(id, value, Arc::new(delegate))
            .range(channel.min..channel.max)
            .step(step))
    }
```
