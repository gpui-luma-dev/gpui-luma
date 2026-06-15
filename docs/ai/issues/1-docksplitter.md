# Specification: Resizable DockSplitter Layout (Seamless Seams)

This document describes the functionality, API, and implementation details for `DockSplitter`—a layout utility that adds drag-to-resize handles between sidebars and fill regions inside a `DockPanel`.

---

## 1. Layout Architecture: The Seamless Seam Pattern (Scenario B)

To avoid creating an $8\text{px}$ visual gap (gutter) between docked panels in the flexbox document flow, the `DockSplitter` decouples its **layout size** from its **interactive hitbox size**:

1. **Layout Size ($1\text{px}$):** The splitter root container is allocated exactly $1\text{px}$ (`visible_line_px`) in the flexbox flow. Adjacent panels meet the splitter line seamlessly with zero layout spacing.
2. **Interactive Hitbox ($8\text{px}$):** An absolute-positioned invisible hitbox is centered on top of the $1\text{px}$ line (offsetting it symmetrically by $-3.5\text{px}$ using `-half_inset`). This overlay captures hover states, cursors, and drag-and-drop gestures.
3. **Grip Thumb:** A rounded vertical/horizontal grab handle (pill shape) is rendered inside the absolute hitbox, matching the exact sizing and style of `ResizeHandleSize::Sm` in `ResizablePanels`.

```
       [Left Panel]          [Splitter]          [Right Panel]
       Bg: #1c1917        Layout: 1px width       Bg: #262626
  ====================>           |           <====================
                      |           |           |
                      |    [8px Hitbox]       |
                      |  <......[Grip]......> |
                      |    -3.5px   +3.5px    |
```

---

## 2. Implementation Specifications

### A. The State & Render Model
Stores the parameters of the splitter and exposes them during rendering:
```rust
pub enum SplitterOrientation {
    Horizontal, // Resizes vertical height (Top/Bottom)
    Vertical,   // Resizes horizontal width (Left/Right)
}

pub struct DockSplitterRenderModel<'a> {
    pub id: &'a SharedString,
    pub orientation: SplitterOrientation,
    pub enabled: bool,
    pub hovered: bool,
    pub dragging: bool,
}
```

### B. Theme Resolution (SDK/Look Layer)
Theme colors and appearances are resolved dynamically via the look stylesheet provider:
```rust
pub struct DockSplitterAppearance {
    pub line_color: Hsla,
    pub hover_color: Hsla,
    pub hit_target_px: f32,      // Default: 8.0px
    pub visible_line_px: f32,    // Default: 1.0px
}

pub trait DockSplitterTheme: Send + Sync {
    fn resolve(&self, enabled: bool) -> DockSplitterAppearance;
}
```

### C. Default SDK Seamless Template (Unopinionated)
The default SDK template in [template.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/dock_splitter/template.rs) is lookless and unopinionated, rendering only the seamless line and hitbox:

```rust
pub struct ThemedDockSplitterTemplate;

impl DockSplitterTemplate for ThemedDockSplitterTemplate {
    fn render(
        &self,
        model: &DockSplitterRenderModel<'_>,
        appearance: &DockSplitterAppearance,
        handlers: DockSplitterTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let DockSplitterTemplateHandlers { hover, mouse_down, mouse_up, mouse_up_out } = handlers;
        let line_color = if model.dragging || model.hovered {
            appearance.hover_color
        } else {
            appearance.line_color
        };
        let half_inset = ((appearance.hit_target_px - appearance.visible_line_px) * 0.5).max(0.0);

        // 1. Root element in flow is only 1px wide/high
        let mut root = div().id(format!("{}-layout", model.id)).relative().flex_shrink_0();
        root = match model.orientation {
            SplitterOrientation::Vertical => root.w(px(appearance.visible_line_px)).h_full(),
            SplitterOrientation::Horizontal => root.h(px(appearance.visible_line_px)).w_full(),
        };

        // 2. Absolute hitbox overlay centered symmetrically (-3.5px offset)
        let hit_target = match model.orientation {
            SplitterOrientation::Vertical => div()
                .id(model.id.clone())
                .absolute()
                .left(px(-half_inset))
                .top(px(0.0))
                .bottom(px(0.0))
                .w(px(appearance.hit_target_px))
                .on_hover(hover)
                .on_mouse_down(MouseButton::Left, mouse_down)
                .on_mouse_up(MouseButton::Left, mouse_up)
                .on_mouse_up_out(MouseButton::Left, mouse_up_out)
                .when(model.enabled, |this| this.cursor_col_resize())
                .child(
                    div()
                        .absolute()
                        .left(px(half_inset))
                        .top(px(0.0))
                        .bottom(px(0.0))
                        .w(px(appearance.visible_line_px))
                        .bg(line_color),
                ),
            SplitterOrientation::Horizontal => div()
                .id(model.id.clone())
                .absolute()
                .top(px(-half_inset))
                .left(px(0.0))
                .right(px(0.0))
                .h(px(appearance.hit_target_px))
                .on_hover(hover)
                .on_mouse_down(MouseButton::Left, mouse_down)
                .on_mouse_up(MouseButton::Left, mouse_up)
                .on_mouse_up_out(MouseButton::Left, mouse_up_out)
                .when(model.enabled, |this| this.cursor_row_resize())
                .child(
                    div()
                        .absolute()
                        .top(px(half_inset))
                        .left(px(0.0))
                        .right(px(0.0))
                        .h(px(appearance.visible_line_px))
                        .bg(line_color),
                ),
        };

        root.child(hit_target)
    }
}
```

---

## 3. Gallery Integration Sandbox (Local Custom Grip Thumb)

To keep the default SDK splitter unopinionated, visual ornamentations (like the grab handle thumb) are implemented via a **local custom template** inside the application sandbox, registered **only on the left-side splitter**.

### A. Local Template Implementation
Add the custom template and imports inside [pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/dock_panel/pane.rs):

```rust
use gpui::{Div, MouseButton, Stateful, prelude::*, px};
use gpui_luma::controls::dock_splitter::{
    DockSplitterAppearance, DockSplitterRenderModel, DockSplitterTemplate,
    DockSplitterTemplateHandlers,
};

struct LeftSplitterTemplate;

impl DockSplitterTemplate for LeftSplitterTemplate {
    fn render(
        &self,
        model: &DockSplitterRenderModel<'_>,
        appearance: &DockSplitterAppearance,
        handlers: DockSplitterTemplateHandlers,
        _window: &mut Window,
        _cx: &mut gpui::App,
    ) -> Stateful<Div> {
        let DockSplitterTemplateHandlers { hover, mouse_down, mouse_up, mouse_up_out } = handlers;
        let line_color = if model.dragging || model.hovered {
            appearance.hover_color
        } else {
            appearance.line_color
        };
        let half_inset = ((appearance.hit_target_px - appearance.visible_line_px) * 0.5).max(0.0);

        // 1px flow width container
        let mut root = div().id(format!("{}-layout", model.id)).relative().flex_shrink_0();
        root = match model.orientation {
            SplitterOrientation::Vertical => root.w(px(appearance.visible_line_px)).h_full(),
            SplitterOrientation::Horizontal => root.h(px(appearance.visible_line_px)).w_full(),
        };

        // Render the local center grip thumb (Sm metrics: 4px x 36px vertical pill)
        let grip = div()
            .absolute()
            .inset_0()
            .flex()
            .justify_center()
            .items_center()
            .child(
                div()
                    .rounded(px(8.0)) // Pill shape
                    .bg(if model.hovered || model.dragging {
                        appearance.hover_color
                    } else {
                        appearance.line_color.opacity(0.4)
                    })
                    .w(px(4.0))
                    .h(px(36.0)),
            );

        // 8px absolute overlay hitbox centered symmetrically (-3.5px offset)
        let hit_target = match model.orientation {
            SplitterOrientation::Vertical => div()
                .id(model.id.clone())
                .absolute()
                .left(px(-half_inset))
                .top(px(0.0))
                .bottom(px(0.0))
                .w(px(appearance.hit_target_px))
                .on_hover(hover)
                .on_mouse_down(MouseButton::Left, mouse_down)
                .on_mouse_up(MouseButton::Left, mouse_up)
                .on_mouse_up_out(MouseButton::Left, mouse_up_out)
                .when(model.enabled, |this| this.cursor_col_resize())
                .child(
                    div()
                        .absolute()
                        .left(px(half_inset))
                        .top(px(0.0))
                        .bottom(px(0.0))
                        .w(px(appearance.visible_line_px))
                        .bg(line_color),
                )
                .child(grip), // Overlay the grip thumb in the center of the vertical hitbox
            SplitterOrientation::Horizontal => div()
                .id(model.id.clone())
                .absolute()
                .top(px(-half_inset))
                .left(px(0.0))
                .right(px(0.0))
                .h(px(appearance.hit_target_px))
                .on_hover(hover)
                .on_mouse_down(MouseButton::Left, mouse_down)
                .on_mouse_up(MouseButton::Left, mouse_up)
                .on_mouse_up_out(MouseButton::Left, mouse_up_out)
                .when(model.enabled, |this| this.cursor_row_resize())
                .child(
                    div()
                        .absolute()
                        .top(px(half_inset))
                        .left(px(0.0))
                        .right(px(0.0))
                        .h(px(appearance.visible_line_px))
                        .bg(line_color),
                ), // No grip thumb on horizontal splitters
        };

        root.child(hit_target)
    }
}
```

### B. Registering the Local Template
Spawn **only** the left-side sidebar with the localized `LeftSplitterTemplate` (top, right, and bottom splitters remain clean):

```rust
        // Left splitter gets the custom pill grip thumb
        let left_splitter = DockSplitter::new("dock-panel-left-splitter", SplitterOrientation::Vertical)
            .template(Arc::new(LeftSplitterTemplate))
            .spawn(cx);

        // Other splitters fall back to clean SDK default ThemedDockSplitterTemplate
        let top_splitter = DockSplitter::new("dock-panel-top-splitter", SplitterOrientation::Horizontal)
            .spawn(cx);
        let right_splitter = DockSplitter::new("dock-panel-right-splitter", SplitterOrientation::Vertical)
            .spawn(cx);
        let bottom_splitter = DockSplitter::new("dock-panel-bottom-splitter", SplitterOrientation::Horizontal)
            .spawn(cx);
```

---

## 4. Performance & Smoothness Analysis (Jerkiness vs. ResizablePanels)

There are two primary architectural differences causing `DockSplitter` to feel less smooth ("jerkier") than `ResizablePanels`:

### A. Relative Deltas (Cumulative Rounding Errors) vs. Absolute Deltas
- **The Issue (DockSplitter):** Resizing is driven by **relative deltas**: `new_width = old_width + delta`. Since GPUI layouts snap to physical device pixels on every frame, fractional values are rounded off. Over multiple frame updates, these rounding errors accumulate, causing the mouse position and the visual divider to drift. When the drift builds up, the panel suddenly snaps or stutters.
- **The Solution (ResizablePanels):** `ResizablePanels` solves layout dimensions from a pristine starting state: `new_width = start_width + (current_mouse - start_mouse)`. By using the **absolute delta** from the drag initiation coordinate, rounding errors never compound, and the mouse pointer stays perfectly locked to the divider.

### B. Raw Window Mouse Events vs. GPUI Drag Event Coalescing
- **The Issue (DockSplitter):** `DockSplitter` listens to raw window-level `MouseMoveEvent` callbacks registered inside a paint canvas. This callback runs as frequently as the operating system's raw mouse input rate (which can be 500Hz or 1000Hz). Flooding GPUI's main thread with dozens of reactive layout invalidations per frame budget (16.6ms at 60Hz) causes dropped frames and stutter.
- **The Solution (ResizablePanels):** `ResizablePanels` utilizes GPUI's built-in drag-and-drop subsystem via `.on_drag` and `.on_drag_move`. GPUI throttles and coalesces these drag moves to sync them precisely with the animation rendering loop, minimizing unnecessary layout cycles.

### Refactoring Recipe to Resolve Jerkiness

To achieve the same high performance as `ResizablePanels` without structural changes:

1. **Emit Absolute Delta instead of relative delta:**
   Update `DockSplitter` resize events to pass the absolute distance from drag start:
   ```rust
   // In control.rs:
   fn handle_mouse_down(&mut self, event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
       self.dragging = true;
       self.drag_start_axis_px = self.axis_position(event.position); // Store absolute start position
       cx.emit(DockSplitterEvent::ResizeStart);
   }

   fn handle_window_mouse_move(&mut self, event: &MouseMoveEvent, _window: &mut Window, cx: &mut Context<Self>) {
       if !self.dragging { return; }
       let absolute_delta = self.axis_position(event.position) - self.drag_start_axis_px;
       cx.emit(DockSplitterEvent::Resize { absolute_delta });
   }
   ```

2. **Save Start State on ResizeStart in the Container:**
   Update the subscription in the container (`pane.rs`) to store the starting width before applying deltas:
   ```rust
   // In pane.rs:
   cx.subscribe(&this.left_splitter, |this, _, event, cx| {
       match event {
           DockSplitterEvent::ResizeStart => {
               this.start_left_width = this.left_width; // Freeze initial width
           }
           DockSplitterEvent::Resize { absolute_delta } => {
               this.left_width = (this.start_left_width + absolute_delta).clamp(40.0, 300.0);
               cx.notify();
           }
           DockSplitterEvent::ResizeEnd => {}
       }
   }).detach();
   ```

3. **Migrate to GPUI Drag-and-Drop Subsystem:**
   Replace raw `canvas` window mouse listeners inside `DockSplitter::render` with GPUI's native drag payload:
   ```rust
   // Define payload
   struct DockSplitterDrag { id: SharedString }

   // Register on the interactive hitbox:
   hit_target
       .on_drag(DockSplitterDrag { id: model.id.clone() }, |_, _, _, cx| cx.new(|_| DockSplitterDrag { ... }))

   // Listen on the root component:
   root.on_drag_move(cx.listener(|this, event: &DragMoveEvent<DockSplitterDrag>, _, cx| {
       // Perform absolute delta calculation here
   }))
   ```

