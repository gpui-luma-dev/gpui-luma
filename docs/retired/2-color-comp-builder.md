# Color Composition Sync Helper

This note should stay narrow.

The problem is real: complex color compositions still require too much hand-wired sync code. But the answer is not a large declarative `ColorCompositionBuilder` framework. The right fix is a small SDK helper that centralizes shared-state sync suppression, sibling control updates, and dependent renderer refresh ordering.

---

## 1. What Is Still Wrong

Today, compositions like the split ring and HSV plane repeat the same pattern:

- own canonical color state in the pane entity
- subscribe to each control manually
- write one changed value into shared state
- push sibling values back into other controls
- rebuild dependent track delegates
- call `refresh_*` helpers in the right order

That is the real duplication.

The note's original motivation is still valid for:

- `apps/gallery/.../hue_ring_sl_arcs.rs`
- `apps/gallery/.../hsv_photoshop.rs`

---

## 2. What This Should Become

Add a small SDK color-composition sync helper.

It should own only:

- re-entrancy suppression for programmatic updates
- shared-state mutation from control events
- pushing sibling control values from shared state
- `update_*_delegate(...)` + `refresh_*` ordering

It should not own:

- layout composition (delegated to GPUI layouts and resolved sizing metrics)
- theme concerns
- generic "product color" abstractions
- `color-viz` yet

---

## 3. Minimal API Shape

Keep the surface small and concrete.

```rust
pub struct ColorCompositionSync<S> {
    state: S,
    is_syncing: bool,
    epsilon: f32,
}

impl<S> ColorCompositionSync<S> {
    pub fn new(state: S) -> Self { /* ... */ }

    pub fn handle_slider(
        &mut self,
        value: f32,
        write_to_state: impl FnOnce(&mut S, f32),
        sync_controls: impl FnOnce(&S),
    ) {
        if self.is_syncing {
            return;
        }

        self.is_syncing = true;
        write_to_state(&mut self.state, value);
        sync_controls(&self.state);
        self.is_syncing = false;
    }

    pub fn state(&self) -> &S { &self.state }
    pub fn state_mut(&mut self) -> &mut S { &mut self.state }
}
```

The important part is the responsibility split, not the exact type signature.

The helper is just a small sync coordinator around existing controls.

---

## 4. Helper-Level Refresh Wrappers

The helper should hide the repeated renderer refresh sequence.

```rust
pub fn sync_ring(
    slider: &Entity<SliderControl>,
    renderer: &ColorRingDomainRenderer,
    delegate: Arc<dyn ColorRingTrackDelegate>,
    context: ColorRingTrackContext,
    value: f32,
    cx: &mut App,
) {
    slider.update(cx, |slider, cx| {
        if (slider.value() - value).abs() > 0.001 {
            slider.set_value(value, cx);
        }
    });
    update_ring_delegate(renderer, delegate, context);
    refresh_color_ring(slider, cx);
}
```

Same idea for arcs and sliders.

This is the actual seam we keep repeating manually today.

---

## 5. First Target: Split Ring

Use the helper first on the split ring.

Success means:

- no app-local `SyncSource` enum
- no app-local "if source is external" sync branching
- no scattered `update_*_delegate(...)` + `refresh_*` calls in the pane

Sketch:

```rust
#[derive(Clone, Copy)]
struct SplitRingState {
    hue_degrees: f32,
    saturation: f32,
    lightness: f32,
}

fn sync_split_ring_controls(&self, state: &SplitRingState, cx: &mut Context<Self>) {
    let hsv_value = Hsv::from_hsla_ext(hsla(
        (state.hue_degrees / 360.0).rem_euclid(1.0),
        state.saturation,
        state.lightness,
        1.0,
    ))
    .v;

    sync_ring(
        &self.hue_ring,
        &self.hue_ring_renderer,
        Arc::new(HueRingDelegate {
            saturation: state.saturation,
            lightness: state.lightness,
        }),
        self.hue_ring_context.clone(),
        state.hue_degrees,
        cx,
    );

    sync_arc(
        &self.saturation_arc,
        &self.saturation_renderer,
        Arc::new(RasterArcDelegate::saturation(state.hue_degrees, hsv_value)),
        self.saturation_context.clone(),
        state.saturation,
        cx,
    );

    sync_arc(
        &self.lightness_arc,
        &self.lightness_renderer,
        Arc::new(RasterArcDelegate::lightness(state.hue_degrees, state.saturation)),
        self.lightness_context.clone(),
        state.lightness,
        cx,
    );
}
```

That is enough to prove the seam.

---

## 6. Second Target: HSV Plane

If split ring works cleanly, apply the same helper to the HSV plane.

That validates the seam for:

- slider-only compositions
- field + slider compositions
- dependent track refresh

No broader rollout is needed before those two are clean.

---

## 7. `color-viz`

Do not apply this in `color-viz` yet.

`color-viz` currently has gradient-stop editing, not the same multi-control composition problem. If it later grows coordinated per-stop color editors, it should reuse this helper instead of inventing a separate sync loop.

---

## 8. Sizing & Dimensions System (Abstract & Pixel)

Compositions (like the split ring) should support standard abstract sizes (`sm`, `md`, `lg`) as well as custom pixel-based measurements, mapping them to concrete dimensions (outer size, track thickness, margins, gaps).

### The Sizing Pattern

Use a composition sizing enum:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CompositionSize {
    Sm,
    #[default]
    Md,
    Lg,
    Custom(Pixels),
}
```

A composition pane/view resolves these using a layout helper struct (e.g. `SplitRingMetrics`) to obtain derived dimensions:

```rust
pub struct SplitRingMetrics {
    pub outer_size: Pixels,
    pub track_width: Pixels,
    pub arc_ring_gap: Pixels,
    pub ring_swatch_gap: Pixels,
    pub outer_padding: Pixels,
    pub border_gap: Pixels,
}

impl SplitRingMetrics {
    pub fn resolve(size: CompositionSize) -> Self {
        match size {
            CompositionSize::Sm => Self {
                outer_size: px(180.0),
                track_width: px(14.0),
                arc_ring_gap: px(4.0),
                ring_swatch_gap: px(8.0),
                outer_padding: px(8.0),
                border_gap: px(10.0),
            },
            CompositionSize::Md => Self {
                outer_size: px(240.0),
                track_width: px(16.0),
                arc_ring_gap: px(5.0),
                ring_swatch_gap: px(12.0),
                outer_padding: px(10.0),
                border_gap: px(12.0),
            },
            CompositionSize::Lg => Self {
                outer_size: px(300.0),
                track_width: px(20.0),
                arc_ring_gap: px(5.0),
                ring_swatch_gap: px(14.0),
                outer_padding: px(12.0),
                border_gap: px(14.0),
            },
            CompositionSize::Custom(custom_outer_size) => {
                // Scale derived dimensions proportionally relative to a 300px reference size
                let factor = custom_outer_size.0 / 300.0;
                Self {
                    outer_size: custom_outer_size,
                    track_width: px(20.0 * factor),
                    arc_ring_gap: px(5.0 * factor),
                    ring_swatch_gap: px(14.0 * factor),
                    outer_padding: px(12.0 * factor),
                    border_gap: px(14.0 * factor),
                }
            }
        }
    }
}
```

This ensures we get pixel-perfect curated layouts for standard sizes, and robust proportional scaling for arbitrary user-defined pixel sizes.

---

## 9. Success Criteria

This change is good if:

- split ring loses most of its manual sync plumbing
- HSV plane can use the same seam
- no new control framework is introduced
- existing color controls remain the real primitives
- compositions support both abstract sizes and custom pixel-based measurements via resolved metric helpers

