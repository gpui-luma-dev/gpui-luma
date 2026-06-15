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
