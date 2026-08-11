# Issue #15: Reusable LayerStack Layout Primitive

## Description
In GPU-accelerated rendering frameworks like GPUI, clipping child elements to a parent's rounded boundaries (`overflow_hidden()`) can be expensive or is not fully supported on some backend primitives (especially when children are absolute-positioned layers). 

This leads to a recurring visual bug: child layers (like solid background colors, hover overlays, or image checkers) with sharp square corners bleed past the parent's rounded border, showing color leaks in the corners.

We will solve this systematically by creating a reusable **`LayerStack` layout primitive** (both as a component and a macro) that automatically synchronizes corner radii across all stacked layers.

---

## Technical Problem: Rounded Corner Bleeding
The bleeding happens because of the mismatch between the parent's boundary path and the child's bounding box:
```text
Parent (Rounded Corner Path):
   ╭────────────────────────╮
   │   Child (Square Box):  │
   │  ┌──────────────────┐  │
   │  │                  │  │
   │  │  Color overlay   │  │
   │  │                  │  │
   │  └──────────────────┘  │
   ╰────────────────────────╯
         ▲ Leak here (sharp corners peek out)
```

By explicitly applying the exact same corner radius to each stacked absolute child layer, we guarantee that no pixels are drawn outside the rounded path, rendering clean rounded corners across all platforms.

---

## Proposed Solutions

### Solution 1: `LayerStack` Utility Component
We can implement a lookless layout component in `crates/sdk/src/layouts/layer_stack.rs` (or similar shared layout module).

```rust
use gpui::{AnyElement, Corners, IntoElement, ParentElement, Pixels, RenderOnce, div, prelude::*};

#[derive(IntoElement)]
pub struct LayerStack {
    children: Vec<AnyElement>,
    radius: Corners<Pixels>,
}

impl LayerStack {
    /// Create a new LayerStack with the specified corner radius
    pub fn new(radius: impl Into<Corners<Pixels>>) -> Self {
        Self {
            children: Vec::new(),
            radius: radius.into(),
        }
    }
}

impl ParentElement for LayerStack {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for LayerStack {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let radius = self.radius;

        div()
            .relative()
            .rounded(radius)
            // Absolute position and apply the matching corner radius to every child
            .children(self.children.into_iter().map(|child| {
                div()
                    .absolute()
                    .inset_0()
                    .rounded(radius)
                    .child(child)
            }))
    }
}
```

### Solution 2: `layer_stack!` Macro
For lightweight inline usage where you want to construct the elements directly, we can define a macro in `crates/sdk/src/macros.rs`:

```rust
#[macro_export]
macro_rules! layer_stack {
    ($radius:expr; $($child:expr),* $(,)?) => {
        div()
            .relative()
            .rounded($radius)
            $(
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .rounded($radius)
                        .child($child)
                )
            )*
    };
}
```

---

## Example Usage

### 1. Color Swatch
Instead of writing a custom color swatch component with complex absolute layer matching:
```rust
fn render_swatch(color: Hsla, is_dark: bool) -> impl IntoElement {
    LayerStack::new(px(12.0))
        .child(Checkerboard::new(is_dark))
        .child(div().bg(color))
}
```

### 2. Badge Overlay with Skeletons
```rust
fn render_badge_with_loader(loading: bool) -> impl IntoElement {
    layer_stack!(px(6.0);
        div().bg(hsla(0.0, 0.0, 0.2, 1.0)), // Base badge background
        if loading { Skeleton::new() } else { BadgeContent::new() }
    )
}
```

---

## Migration & Implementation Plan

1. **Implement Core Primitives**:
   * Add the `LayerStack` component under `crates/sdk/src/layouts/layer_stack.rs` (and export in `crates/sdk/src/layouts/mod.rs`).
   * Add the `layer_stack!` macro inside `crates/sdk/src/macros.rs`.
2. **Refactor Existing Bleed-Prone Swatches**:
   * Update [multi_mixer_pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/color/multi_mixer_pane.rs) swatches to use the new primitive.
   * Review other layout widgets in luma studio (like preview chips, selection badges) where absolute overlays are used and convert them.

---

## Verification Plan

### Automated Tests
* Add a layout test verifying `LayerStack` renders without panicking and correctly delegates `Corners<Pixels>` to all inner absolute child containers.

### Manual Verification
* Add a card inside the gallery's layout pane to showcase different `LayerStack` variations:
  * A swatch showing a gradient over a checkerboard.
  * A text title layered on top of an image background.
* Verify visually that all card designs display perfect rounded boundaries on screens, without corner leaking.
