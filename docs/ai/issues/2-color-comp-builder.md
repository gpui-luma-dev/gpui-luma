# Color Composition Builder & Coordinator

This document outlines the architecture for a new declarative `ColorCompositionBuilder` and event coordinator pattern to simplify, unify, and robustly test complex multi-control color picker layouts (such as the Pixagram-style split-ring picker).

---

## 1. Background: The Problem with Hand-Tuned Compositions

During the development and testing of radial color pickers and sliders, several critical issues emerged:
* **Circular Sync Feedback Loops**: When multiple controls represent overlapping fields of a single color model (e.g. saturation changing HSV value, which updates lightness), programmatic synchronizations trigger cascading event listeners. Developers were forced to use manual state enums (e.g., `SyncSource`) or boolean guard flags (e.g., `programmatic_update_in_progress`) to prevent stack overflows.
* **Scattered Delegate/Renderer Lifetimes**: Each color component (Hue Ring, Saturation Arc, Lightness Arc) requires holding reference entities for:
  1. The `Entity<SliderControl>` (logical interaction, value tracking)
  2. The `Arc<DomainRenderer>` (custom background rasterization)
  3. The `TrackContext` (physical dimensions, radius, sweep angles, thickness)
  
  Keeping all three components synchronized during an update loop required calling `update_*_delegate(...)` followed by `refresh_*(...)` in specific sequences. A single missed call caused the thumb position to drift away from the background gradient coordinates.
* **Floating Point Drift**: Converting continuously between degrees ($0..360$), turns ($0..1$), and normalized percentages ($0..1$) caused minor float fluctuations. When synced back to controls, these tiny drifts triggered continuous change events.
* **Coordinate Mapping Gotchas**: For elements that placed color controls at non-standard rotations (e.g., mirrored saturation arcs or radial controllers rotated $180^{\circ}$), the logic mapping saturation to physical positions was manually calculated inside the composition, hiding bugs that should have been managed at the engine level.

---

## 2. The Improved Architecture

To eliminate these problems, we introduce the **Composition Coordinator** pattern. The coordinator owns the shared color state, acts as the single source of truth, manages all subscriptions, and handles recursive update suppression.

### Proposed Coordinator & Builder Design (Pseudo-code)

```rust
use std::sync::Arc;
use gpui::{AppContext, Entity, EntityId, ViewContext};
use gpui_luma::controls::slider::{SliderControl, SliderEvent, primary_slider_value, DomainTrackRenderer};

/// Trait representing a shared color model (e.g. HSLA, OKLCH, HSV)
pub trait ColorState: Clone + PartialEq + 'static {
    /// The final mixed color product output (e.g., gpui::Hsla or a custom swatch color type)
    type Product: Clone + PartialEq + 'static;

    /// Computes the final product color from the current state values
    fn product(&self) -> Self::Product;
}

/// Holds the configuration and callbacks for a single slider component in the composition
struct ControlBinding<S: ColorState> {
    control: Entity<SliderControl>,
    // Extracts the logical value from the state to update the slider
    getter: Box<dyn Fn(&S) -> f32 + Send + Sync>,
    // Commits the slider value back to the state
    setter: Box<dyn Fn(&mut S, f32) + Send + Sync>,
    // Dynamically reconstructs the track delegate when the state changes
    track_factory: Option<Box<dyn Fn(&S) -> Arc<dyn ColorTrackDelegate> + Send + Sync>>,
    renderer: Option<Arc<dyn DomainTrackRenderer>>,
    context: Option<TrackContext>,
}

/// The Coordinator managing all synchronized sliders
pub struct ColorCompositionCoordinator<S: ColorState> {
    state: S,
    bindings: Vec<ControlBinding<S>>,
    is_updating: bool, // Update lock guard to prevent recursive event loop cycles
    // Fired whenever the resolved product changes
    on_product_change: Option<Box<dyn Fn(&S::Product, &mut ViewContext<Self>) + Send + Sync>>,
}

impl<S: ColorState> ColorCompositionCoordinator<S> {
    pub fn new(initial_state: S) -> Self {
        Self {
            state: initial_state,
            bindings: Vec::new(),
            is_updating: false,
            on_product_change: None,
        }
    }

    /// Sets a callback to be run whenever the mixed product color changes
    pub fn on_product_change<F>(&mut self, callback: F)
    where
        F: Fn(&S::Product, &mut ViewContext<Self>) + Send + Sync + 'static,
    {
        self.on_product_change = Some(Box::new(callback));
    }

    /// Retrieves the current mixed color product
    pub fn product(&self) -> S::Product {
        self.state.product()
    }

    /// Primary event entry point called whenever any bound control fires a change event
    pub fn handle_control_change(&mut self, control_id: EntityId, value: f32, cx: &mut ViewContext<Self>) {
        if self.is_updating {
            return;
        }

        self.is_updating = true;

        // 1. Update the central color state with the new value
        if let Some(binding) = self.bindings.iter_mut().find(|b| b.control.entity_id() == control_id) {
            (binding.setter)(&mut self.state, value);
        }

        // 2. Synchronize all other controls to match the updated state
        for binding in &self.bindings {
            let target_value = (binding.getter)(&self.state);

            // Epsilon check prevents microscopic float discrepancies from triggering updates
            if (binding.control.read(cx).value() - target_value).abs() > 0.001 {
                binding.control.update(cx, |slider, cx| {
                    slider.set_value(target_value, cx);
                });
            }

            // 3. Atomically regenerate track graphics if delegate dependencies changed
            if let Some(factory) = &binding.track_factory
                && let Some(renderer) = &binding.renderer
                && let Some(context) = &binding.context
            {
                let new_delegate = factory(&self.state);
                renderer.set_delegate(new_delegate);
                renderer.set_context(context.clone());

                // Trigger domain layout and thumb preview refreshes
                binding.control.update(cx, |slider, cx| {
                    slider.sync_domain_thumb_previews();
                    cx.notify();
                });
            }
        }

        self.is_updating = false;

        // 4. Notify about product changes if the color was modified
        if let Some(on_change) = &self.on_product_change {
            let product = self.state.product();
            (on_change)(&product, cx);
        }

        cx.notify(); // Redraws composition container
    }
}
```

---

## 3. Sample Build: Refactoring the Split Ring

Using the proposed coordinator pattern, we can implement the Pixagram-style `SplitRing` (`split_ring_pixagram.rs`) without any custom event subscriptions, recursion guards, or manual delegate updates in the view code.

### The Refactored Implementation

```rust
use std::sync::Arc;
use gpui::{Context, Entity, Hsla, px, Size, hsla};
use gpui_luma::controls::color::color_arc::{ColorArcBuilder, RasterArcDelegate};
use gpui_luma::controls::color::color_ring::{ColorRingBuilder, HueRingDelegate};
use gpui_luma::controls::slider::SliderControl;

// Define our shared color state
#[derive(Clone, PartialEq)]
struct SplitRingColor {
    hue: f32,       // 0..360
    saturation: f32, // 0..1
    lightness: f32,  // 0..1
}

impl ColorState for SplitRingColor {
    type Product = Hsla;

    // The mixed output color "product"
    fn product(&self) -> Self::Product {
        hsla(self.hue / 360.0, self.saturation, self.lightness, 1.0)
    }
}

pub struct SplitRingComposition {
    coordinator: Entity<ColorCompositionCoordinator<SplitRingColor>>,
    hue_ring: Entity<SliderControl>,
    saturation_arc: Entity<SliderControl>,
    lightness_arc: Entity<SliderControl>,
}

impl SplitRingComposition {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let initial_color = SplitRingColor {
            hue: 18.0,
            saturation: 0.85,
            lightness: 0.49,
        };

        // 1. Initialize the builder with a target outer size (e.g., 300px for desktop, 150px for mobile)
        let target_size = px(150.0); // Simple single-point diameter adjustment!
        let mut builder = ColorCompositionBuilder::new(initial_color, target_size, cx);

        // Define ratio measurements relative to the target outer diameter
        let track_ratio = 0.067;       // 20px / 300px
        let ring_size_ratio = 0.733;    // 220px / 300px

        // 2. Bind Hue Ring (Middle ring - scaled proportionally)
        let hue_ring = builder.add_ring(
            "split-ring-hue",
            |color| color.hue,
            |color, val| color.hue = val,
            |color| HueRingDelegate {
                saturation: color.saturation,
                lightness: color.lightness,
            },
            |ring, layout| ring
                .size(layout.relative_size(ring_size_ratio))
                .ring_thickness_size(layout.relative_thickness(track_ratio)),
            cx,
        );

        // 3. Bind Saturation Arc (Top sweep - outer)
        let saturation_arc = builder.add_arc(
            "split-ring-saturation",
            |color| color.saturation,
            |color, val| color.saturation = val,
            |color| {
                let hsv_v = Hsv::from_hsla_ext(color.product()).v;
                RasterArcDelegate::saturation(color.hue, hsv_v)
            },
            |arc, layout| arc
                .size(layout.outer_size()) // Defaults to target_size (1.0 ratio)
                .start_degrees(184.0)
                .sweep_degrees(172.0)
                .arc_thickness_size(layout.relative_thickness(track_ratio)),
            cx,
        );

        // 4. Bind Lightness Arc (Bottom sweep - outer)
        let lightness_arc = builder.add_arc(
            "split-ring-lightness",
            |color| color.lightness,
            |color, val| color.lightness = val,
            |color| RasterArcDelegate::lightness(color.hue, color.saturation),
            |arc, layout| arc
                .size(layout.outer_size())
                .start_degrees(4.0)
                .sweep_degrees(172.0)
                .arc_thickness_size(layout.relative_thickness(track_ratio)),
            cx,
        );

        // Register action on product change
        builder.on_product_change(|product_color, cx| {
            println!("Mixed product color updated: {:?}", product_color);
        });

        Self {
            coordinator: builder.build(),
            hue_ring,
            saturation_arc,
            lightness_arc,
        }
    }
}
```

---

## 4. Proportional Layout & Size Scaling

To avoid "chasing pixel measurements" when scaling a composition up or down, the `ColorCompositionBuilder` utilizes a **proportional layout context** during child control construction.

### Layout Helper (Pseudo-code)

```rust
/// Helper passed to child configurators to scale widths, radius, and margins proportionally
pub struct CompositionLayoutContext {
    base_diameter: Pixels,
}

impl CompositionLayoutContext {
    /// Returns the target outer diameter of the entire composition container
    pub fn outer_size(&self) -> Size {
        Size::Size(self.base_diameter)
    }

    /// Computes a scaled diameter relative to the base diameter
    pub fn relative_size(&self, ratio: f32) -> Size {
        Size::Size(self.base_diameter * ratio)
    }

    /// Computes a scaled track/border/margin thickness relative to the base diameter
    pub fn relative_thickness(&self, ratio: f32) -> Size {
        Size::Size(self.base_diameter * ratio)
    }
}
```

By expressing child properties in terms of the layout context, changing the composition size requires updating only **one parameter** (e.g. changing `target_size` from `px(300.0)` to `px(150.0)`). The builder automatically scales the rings, arcs, gaps, and borders proportionally without layout overlapping or breaking.

---

## 5. Key Gotchas Resolved

1. **Synchronous Updates**: Visual track gradients now update *atomically* alongside physical thumb coordinate recalculations. There is no longer a frame lag where the thumb jumps prior to the rasterized track cache regenerating.
2. **Unified Directionality**: The builder handles translating the coordinate system (e.g. clockwise vs counter-clockwise angles, linear ranges) so that setters/getters only see normalized logical color spaces.
3. **Product Color Isolation**: Parent views can subscribe to the final product color event directly, without needing to know anything about the underlying component composition structure (e.g. rings, triangles, or sliders).
4. **Proportional Scaling**: Eliminates absolute pixel adjustments. High-resolution desktop pickers and compact mobile sliders can share the exact same builder layout definition by passing a different base diameter.
5. **Robust Unit Testing**: Since the `ColorCompositionCoordinator` contains all coordinate mapping, bounds clamping, and synchronization logic in a pure state machine, it can be tested completely in memory without requiring a UI/window layout engine.


