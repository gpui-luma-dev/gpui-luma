# Specification: Resizable DockSplitter Layout

This document describes the functionality, API, and implementation details for `DockSplitter`—a lightweight layout utility that adds drag-to-resize handles between sidebars and fill regions inside a `DockPanel`.

---

## 1. Functionality Overview

The `DockSplitter` is a stateless visual handle that intercepts mouse drag gestures. It translates mouse movement deltas on the main axis into updates for the adjacent panel’s size.

Unlike complex multi-split containers, `DockSplitter` delegates state storage back to the parent container. The sidebar size is simply a local state variable (e.g. `self.left_width: f32`), which makes saving, loading, and adjusting the panels simple and clean.

### Core Features:
* **Hit Target Scaling:** Provides a narrow visible separator line (`1px`) wrapped inside a wider invisible hit target (`8px`) to ensure hover and drag interactions feel generous.
* **Window-Level Drag Tracking:** Once clicked, the splitter registers window-level mouse-move and mouse-up listeners to track movement outside the boundary limits, automatically releasing when the mouse button is let go.
* **Directional Cursors:** Automatically changes the system cursor to `ColResize` (for vertical splitters) or `RowResize` (for horizontal splitters).

---

## 2. Implementation Specifications (Theme/Template Pattern)

To ensure consistency with the rest of the SDK, the `DockSplitter` is split into four parts: a State/Render Model, a Theme interface, a rendering Template, and the parent Control.

This decouples the visual style (like colors, margins, or grip icons) from the core mouse drag logic.

### A. The State & Render Model
Stores the parameters of the splitter and exposes them during render phase:
```rust
pub enum SplitterOrientation {
    Horizontal, // Resizes vertical height (Top/Bottom)
    Vertical,   // Resizes horizontal width (Left/Right)
}

pub struct DockSplitterRenderModel<'a> {
    pub id: &'a SharedString,
    pub orientation: SplitterOrientation,
    pub enabled: bool,
}
```

### B. The Theme & Look
Defines style tokens resolved from the theme (such as `ShadcnLook` border colors and hover metrics):
```rust
pub struct DockSplitterLook {
    pub line_color: Hsla,
    pub hover_color: Hsla,
    pub hit_target_px: f32,      // Default hit zone (e.g., 8.0px)
    pub visible_line_px: f32,    // Visible separator line (e.g., 1.0px)
}

pub trait DockSplitterTheme: Send + Sync {
    fn resolve(&self, enabled: bool) -> DockSplitterLook;
}
```

### C. The Rendering Template
Builds the visual GPUI element structure. Looks can swap this template to add custom visual splits (like dot grips in the center):
```rust
pub trait DockSplitterTemplate: Send + Sync {
    fn render(&self, model: &DockSplitterRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<Div>;
}
```

### D. The Control Builder API
The parent control is created via the SDK builder pattern:
```rust
pub struct DockSplitter {
    id: SharedString,
    orientation: SplitterOrientation,
    on_resize: Option<Rc<dyn Fn(f32, &mut Window, &mut AppContext)>>,
    template: Arc<dyn DockSplitterTemplate>,
}

impl DockSplitter {
    pub fn new(orientation: SplitterOrientation) -> Self;
    pub fn on_resize(self, callback: impl Fn(f32, &mut Window, &mut AppContext) + 'static) -> Self;
    pub fn template(self, template: Arc<dyn DockSplitterTemplate>) -> Self;
}
```


---

## 3. Integration Example: `dock_panel/pane.rs`

Below is the concrete implementation showing how `DockSplitter` is integrated into the existing `DockPanelPane` gallery pane code to make all four outer boundaries resizable.

### Refactored State & Render:
```rust
#[derive(Clone)]
pub(in crate::gallery) struct DockPanelPane {
    state: Entity<DockPanelPaneState>,
}

struct DockPanelPaneState {
    left_width: f32,
    right_width: f32,
    top_height: f32,
    bottom_height: f32,
}

impl DockPanelPaneState {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            left_width: 80.0,
            right_width: 80.0,
            top_height: 50.0,
            bottom_height: 50.0,
        }
    }
}

impl gpui::Render for DockPanelPaneState {
    fn render(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();

        dock_panel! {
            // 1. LEFT SIDEBAR + SPLITTER
            left: div()
                .w(px(self.left_width))
                .bg(chrome.panel_background)
                .child("Left"),
            left: DockSplitter::new(SplitterOrientation::Vertical)
                .on_resize(cx.listener(|this, delta, cx| {
                    this.left_width = (this.left_width + delta).clamp(40.0, 300.0);
                    cx.notify();
                })),

            // 2. TOP HEADER + SPLITTER
            top: div()
                .h(px(self.top_height))
                .bg(chrome.panel_background)
                .child("Top"),
            top: DockSplitter::new(SplitterOrientation::Horizontal)
                .on_resize(cx.listener(|this, delta, cx| {
                    this.top_height = (this.top_height + delta).clamp(30.0, 150.0);
                    cx.notify();
                })),

            // 3. RIGHT SIDEBAR + SPLITTER
            right: div()
                .w(px(self.right_width))
                .bg(chrome.panel_background)
                .child("Right"),
            right: DockSplitter::new(SplitterOrientation::Vertical)
                .on_resize(cx.listener(|this, delta, cx| {
                    // Moving cursor left (negative delta) expands right panel width
                    this.right_width = (this.right_width - delta).clamp(40.0, 300.0);
                    cx.notify();
                })),

            // 4. BOTTOM FOOTER + SPLITTER
            bottom: div()
                .h(px(self.bottom_height))
                .bg(chrome.panel_background)
                .child("Bottom"),
            bottom: DockSplitter::new(SplitterOrientation::Horizontal)
                .on_resize(cx.listener(|this, delta, cx| {
                    // Moving cursor up (negative delta) expands bottom panel height
                    this.bottom_height = (this.bottom_height - delta).clamp(30.0, 150.0);
                    cx.notify();
                })),

            // 5. FILL WORKSPACE
            fill: div()
                .size_full()
                .bg(gpui::red())
                .child("Center Fill")
        }
    }
}
```

---

## 4. Key Design Patterns Highlighted in the Code
1. **Dynamic Precedence Order:** The sidebar and its splitter are docked *together* under the same edge (`left: ...` followed by `left: ...`), guaranteeing that the splitter spans the exact same height boundary as the sidebar itself.
2. **Reverse Delta Calculation:** For the right and bottom boundaries, moving the mouse in the negative direction (left/up) increases the size of the sidebar. The callback logic handles this simply by subtracting the delta: `this.right_width - delta`.
3. **No Intermediate Containers:** Because of the dynamic docking order, the splitters and boundaries are created without wrapping elements or complex layout math.
