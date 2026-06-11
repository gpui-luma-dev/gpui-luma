# SDK Slider Generalization Proposal (1D Range Control)

This document outlines the proposed changes to the SDK `Slider` component to generalize it into a reusable **1D Range Control**. These changes will allow the `Slider` core engine to support different orientations (vertical/horizontal sliders), rotary controls (dials/knobs), and scrollbars while sharing the same interaction state, keyboard shortcuts, and mouse gesture state machine.

---

## 1. Problem Statement

Currently, the SDK slider is locked to a horizontal layout:
1. **Horizontal Coordinate Math**: The [SliderControl::set_value_from_position](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slider/control.rs#L127-L144) function only tracks the X-coordinate relative to the track bounds:
   ```rust
   let percentage = ((position.x - bounds.left()) / bounds.size.width).clamp(0.0, 1.0);
   ```
2. **Coupled Styling Schema**: The default styling schema, [SliderAppearance](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slider/theme.rs#L8-L20), is hardcoded to linear parameters (e.g., `track_height`, `thumb_size`) which do not map to the geometry of custom rotary controls.
3. **Template Event Limitations**: The [SliderTemplate](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slider/template.rs#L28-L36) has no mutable access to the underlying control context. It relies on [SliderTemplateHandlers](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slider/template.rs#L19-L26) that are pre-bound to the horizontal mouse tracking handlers in [SliderControl](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slider/control.rs#L36-L40).

As a result, specialized range inputs like the rotary [dial.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/neumorphic-demo/src/components/dial.rs) or scrollbars are forced to write their own mouse-tracking, state management, and key-binding systems from scratch.

---

## 2. Proposed Architecture

To solve these problems, the SDK slider can be refactored into a general-purpose state machine by decoupling the **coordinate projection** and allowing **custom templates** to ignore default linear styling parameters.

### A. Input Strategy Abstraction

We introduce a `RangeInputStrategy` configuration enum (or strategy trait) to govern coordinate math:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RangeInputStrategy {
    /// Standard horizontal left-to-right slider/scrollbar
    Horizontal,
    /// Vertical bottom-to-top slider/scrollbar
    Vertical,
    /// Rotational dial mapped to angular mouse interaction
    Angular {
        min_angle: f32,
        max_angle: f32,
    },
}
```

The [SliderControl::set_value_from_position](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slider/control.rs#L127-L144) helper is updated to switch on the selected strategy:

```rust
fn set_value_from_position(&mut self, position: Point<Pixels>, emit: bool, cx: &mut Context<Self>) -> bool {
    if !self.model.enabled {
        return false;
    }
    let Some(bounds) = self.track_bounds else {
        return false;
    };

    let percentage = match self.model.strategy {
        RangeInputStrategy::Horizontal => {
            if bounds.size.width <= px(0.0) { 0.0 } else {
                ((position.x - bounds.left()) / bounds.size.width).clamp(0.0, 1.0)
            }
        }
        RangeInputStrategy::Vertical => {
            if bounds.size.height <= px(0.0) { 0.0 } else {
                // Invert because GPUI screen coordinates run top-to-bottom
                (1.0 - (position.y - bounds.top()) / bounds.size.height).clamp(0.0, 1.0)
            }
        }
        RangeInputStrategy::Angular { min_angle, max_angle } => {
            let center = bounds.center();
            let dy = (position.y - center.y).as_f32();
            let dx = (position.x - center.x).as_f32();
            let angle = dy.atan2(dx);
            
            // Map the calculated angle to [0.0, 1.0] range
            let span = max_angle - min_angle;
            let mut normalized_angle = angle;
            if normalized_angle > max_angle {
                normalized_angle -= std::f32::consts::TAU;
            }
            let clamped_angle = normalized_angle.clamp(min_angle, max_angle);
            ((clamped_angle - min_angle) / span).clamp(0.0, 1.0)
        }
    };

    let value = self.model.range.value_at(percentage);
    self.set_value_internal(value, emit, cx)
}
```

### B. Trackpad-Friendly Drag Deltas

For rotary dials, dragging in a circle is notoriously frustrating with trackpads and mice. The industry standard in professional design and audio software is to translate **vertical relative mouse dragging** directly into value adjustments.

To support relative adjustments seamlessly:
1. Update `SliderTemplateHandlers` to support delta-based updates, or:
2. Update the drag tracking handler in [control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slider/control.rs):

```rust
fn handle_drag_move(&mut self, event: &DragMoveEvent<SliderDrag>, _window: &mut Window, cx: &mut Context<Self>) {
    if event.drag(cx).id != self.model.id {
        return;
    }

    if let RangeInputStrategy::Angular { .. } = self.model.strategy {
        // Translate vertical drag delta to relative value change
        let dy = event.event.velocity.y.as_f32();
        let sensitivity = 150.0; // Number of pixels for a full 0-1 sweep
        let delta = -dy / sensitivity;
        self.set_value_internal(self.model.value + delta, true, cx);
    } else {
        // Standard absolute track-bounds tracking
        self.track_bounds = Some(event.bounds);
        self.set_value_from_position(event.event.position, true, cx);
    }
}
```

### C. Bypassing SliderAppearance

Custom fader, scrollbar, or dial templates can completely ignore [SliderAppearance](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slider/theme.rs#L8-L20) and resolve styling parameters (e.g. colors, font size, margins) directly from the application context (`cx`).

For example, a custom `DialSliderTemplate` implements [SliderTemplate](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slider/template.rs#L28-L36) like this:

```rust
pub struct DialSliderTemplate {
    pub dial_size: DialSize,
}

impl SliderTemplate for DialSliderTemplate {
    fn render(
        &self,
        model: &SliderRenderModel<'_>,
        handlers: SliderTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let SliderTemplateHandlers { track_bounds, hover, mouse_down, mouse_up, mouse_up_out, drag_move } = handlers;
        let value = model.value;
        let dial_size = self.dial_size;
        
        // Draw the circular interface via canvas elements using dial_size 
        // and current model.value, completely ignoring standard linear track heights/thumb sizes.
        div()
            .id(model.id.clone())
            .relative()
            .on_hover(hover)
            .on_mouse_down(MouseButton::Left, mouse_down)
            .on_mouse_up(MouseButton::Left, mouse_up)
            .on_drag(SliderDrag::new(model.id.clone()), |drag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .on_drag_move(drag_move)
            .child(
                canvas(
                    move |bounds, window, cx| track_bounds(&bounds, window, cx),
                    move |bounds, _, window, _| paint_dial(bounds, value, dial_size, window),
                )
                .size_full(),
            )
    }
}
```

---

## 3. Advantages of This Design

*   **Behavior Reuse**: All accessibility features, keyboard controls (up/down arrow increments, page adjustments, home/end keys), hover states, pressed states, and drag lifecycles are written once in [SliderControl](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slider/control.rs#L36-L40).
*   **Decoupled Drawing**: Rendering dial tick marks, custom shadows, and rotating pointer elements is isolated in the template, keeping the core control file clean.
*   **Trackpad Optimization**: Allows rotary knobs to behave elegantly on trackpads by translating vertical swipes into clean values, while still supporting absolute click-to-angle actions.
*   **Scrollbars Built Free**: With vertical and horizontal projection support, a scrollbar can be built simply as a custom-templated vertical/horizontal slider.

---

## 4. Refactoring the Neumorphic Demo Dial

To migrate the home-brewed `Dial` component in [dial.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/neumorphic-demo/src/components/dial.rs) to use this unified SDK approach, follow these steps:

### Step 1: Remove Custom State & Input Boilerplate
Remove the local dragging definitions (`DialDrag`, `capture_bounds`, and custom mouse listener logic) from [dial.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/neumorphic-demo/src/components/dial.rs). All input state tracking, keyboard focus, and bounds registration are offloaded to [SliderControl](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slider/control.rs#L36-L40).

### Step 2: Extract the Custom Dial Template
Convert the rendering function into a struct implementing the [SliderTemplate](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slider/template.rs#L28-L36) trait:

```rust
pub struct NeumorphicDialTemplate {
    pub size: DialSize,
    pub label: SharedString,
}

impl SliderTemplate for NeumorphicDialTemplate {
    fn render(
        &self,
        model: &SliderRenderModel<'_>,
        handlers: SliderTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let SliderTemplateHandlers { track_bounds, hover, mouse_down, mouse_up, mouse_up_out, drag_move } = handlers;
        let value = model.value;
        let dial_size = self.size;
        let label = self.label.clone();
        
        div()
            .flex_col()
            .items_center()
            .gap(px(12.0))
            .child(
                div()
                    .id(model.id.clone())
                    .relative()
                    .size(dial_size.frame_diameter())
                    .cursor_pointer()
                    .on_hover(hover)
                    .on_mouse_down(MouseButton::Left, mouse_down)
                    .on_mouse_up(MouseButton::Left, mouse_up)
                    .on_mouse_up_out(MouseButton::Left, mouse_up_out)
                    .on_drag(SliderDrag::new(model.id.clone()), |drag, _, _, cx| {
                        cx.stop_propagation();
                        cx.new(|_| drag.clone())
                    })
                    .on_drag_move(drag_move)
                    .child(
                        canvas(
                            move |bounds, window, cx| track_bounds(&bounds, window, cx),
                            move |bounds, _, window, _| paint_dial(bounds, value, dial_size, window),
                        )
                        .size_full(),
                    ),
            )
            .child(
                div()
                    .w(dial_size.frame_diameter())
                    .flex()
                    .justify_center()
                    .text_size(dial_size.label_size())
                    .line_height(dial_size.label_line_height())
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(0x596273))
                    .child(label),
            )
    }
}
```

### Step 3: Instantiate Dials via the SDK Slider Builder
When creating dials in your panels or dashboard decks, use the SDK's slider builder to supply the dial template and configured strategy:

```rust
use std::f32::consts::PI;

let volume_dial = Slider::new("volume-knob")
    .range(0.0..=1.0)
    .value(0.5)
    .strategy(RangeInputStrategy::Angular {
        min_angle: -1.25 * PI,
        max_angle: 0.25 * PI,
    })
    .template(Arc::new(NeumorphicDialTemplate {
        size: DialSize::Large,
        label: "Volume".into(),
    }))
    .spawn(cx);
```

### Step 4: Handle Events Cleanly
Instead of custom callbacks, subscribe to the slider control's standardized change events:

```rust
cx.subscribe(&volume_dial, |_parent, _control, event, cx| match event {
    SliderEvent::Change { value } => {
        // Update volume level / DSP node
    }
}).detach();
```
