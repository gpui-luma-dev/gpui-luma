# Specification & Upfill Plan: Focus & Keyboard for DockSplitter

This document specifies the keyboard accessibility, focus states, and alternate templates for the `DockSplitter` component in the Luma SDK, and outlines how these are integrated into the Gallery application's `dock_panel` demo pane.

---

## 1. SDK Specifications: DockSplitter Focus & Key Handling

We will enhance the SDK `DockSplitter` to act as a focusable element that handles keyboard events for resizing panels.

### A. Focus Registration & State Models (`crates/sdk/src/controls/dock_splitter/`)

#### 1. [control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/dock_splitter/control.rs)
- Instantiate and store a `FocusHandle` in `DockSplitter`.
- Expose the focus state to the template via `DockSplitterRenderModel`.
- Add a keyboard resize key down listener (`handle_key_down`).
- To avoid modifying parent application subscription handlers (which store drag start dimensions on `ResizeStart` and expect absolute deltas on `Resize`), keyboard resizes will emit a transactional sequence: `ResizeStart`, `Resize { total_delta }`, and `ResizeEnd`.

```rust
pub struct DockSplitter {
    model: DockSplitterModel,
    focus_handle: gpui::FocusHandle, // NEW: Focus state tracker
    hovered: bool,
    dragging: bool,
    drag_start_axis_px: f32,
}

impl DockSplitter {
    pub(crate) fn from_builder(builder: DockSplitterBuilder, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle(); // NEW: Create FocusHandle
        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self {
            model: builder.model,
            focus_handle,
            hovered: false,
            dragging: false,
            drag_start_axis_px: 0.0,
        }
    }

    pub fn focus_handle(&self) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui::Focusable for DockSplitter {
    fn focus_handle(&self, _cx: &gpui::AppContext) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl DockSplitter {
    fn render_model(&self, window: &mut Window) -> DockSplitterRenderModel<'_> {
        DockSplitterRenderModel {
            id: &self.model.id,
            orientation: self.model.orientation,
            enabled: self.model.enabled,
            hovered: self.hovered,
            dragging: self.dragging,
            focused: self.focus_handle.is_focused(window), // NEW: Track active focus
            focus_handle: &self.focus_handle,             // NEW: Pass handle to template
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> DockSplitterTemplateHandlers {
        DockSplitterTemplateHandlers {
            hover: Box::new(cx.listener(Self::handle_hover)),
            mouse_down: Box::new(cx.listener(Self::handle_mouse_down)),
            mouse_up: Box::new(cx.listener(Self::handle_mouse_up)),
            mouse_up_out: Box::new(cx.listener(Self::handle_mouse_up)),
            key_down: Box::new(cx.listener(Self::handle_key_down)), // NEW
        }
    }

    fn handle_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window, // Changed from _window
        cx: &mut Context<Self>,
    ) {
        if !self.model.enabled || event.button != MouseButton::Left {
            return;
        }

        self.focus_handle.focus(window, cx); // NEW: Request focus on click

        self.dragging = true;
        self.drag_start_axis_px = self.axis_position(event.position);
        cx.emit(DockSplitterEvent::ResizeStart);
        cx.notify();
    }

    fn handle_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.model.enabled {
            return;
        }

        let step = if event.keystroke.modifiers.shift {
            self.model.keyboard_shift_step
        } else {
            self.model.keyboard_step
        };

        let delta = match (self.model.orientation, event.keystroke.key.as_str()) {
            (SplitterOrientation::Vertical, "left" | "up") => -step,
            (SplitterOrientation::Vertical, "right" | "down") => step,
            (SplitterOrientation::Horizontal, "up" | "left") => -step,
            (SplitterOrientation::Horizontal, "down" | "right") => step,
            _ => return, // Ignore irrelevant keys
        };

        // Emit transaction sequence to keep parent subscriptions clean
        cx.emit(DockSplitterEvent::ResizeStart);
        cx.emit(DockSplitterEvent::Resize { total_delta: delta });
        cx.emit(DockSplitterEvent::ResizeEnd);

        window.prevent_default();
        cx.stop_propagation();
    }
}
```

#### 2. [model.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/dock_splitter/model.rs)
- Update `DockSplitterRenderModel` to carry `focused` and `focus_handle`.
- Add customizable keyboard step sizes with default values (`2.0` pixels normal, `10.0` pixels with Shift) to the model and builder.

```rust
pub struct DockSplitterModel {
    pub(crate) id: SharedString,
    pub(crate) orientation: SplitterOrientation,
    pub(crate) enabled: bool,
    pub(crate) on_resize: Option<DockSplitterResizeHandler>,
    pub(crate) template: Arc<dyn DockSplitterTemplate>,
    pub(crate) theme: Arc<dyn DockSplitterTheme>,
    pub(crate) keyboard_step: f32,       // Default: 2.0
    pub(crate) keyboard_shift_step: f32, // Default: 10.0
}

pub struct DockSplitterRenderModel<'a> {
    pub id: &'a SharedString,
    pub orientation: SplitterOrientation,
    pub enabled: bool,
    pub hovered: bool,
    pub dragging: bool,
    pub focused: bool,                      // NEW
    pub focus_handle: &'a gpui::FocusHandle, // NEW
}

impl DockSplitterBuilder {
    // Modify constructor to set default step values
    pub fn new(id: impl Into<SharedString>, orientation: SplitterOrientation) -> Self {
        Self {
            model: DockSplitterModel {
                id: id.into(),
                orientation,
                enabled: true,
                on_resize: None,
                template: default_dock_splitter_template(),
                theme: default_dock_splitter_theme(),
                keyboard_step: 2.0,       // Default: 2.0px
                keyboard_shift_step: 10.0, // Default: 10.0px
            },
        }
    }

    pub fn keyboard_step(mut self, step: f32) -> Self {
        self.model.keyboard_step = step;
        self
    }

    pub fn keyboard_shift_step(mut self, step: f32) -> Self {
        self.model.keyboard_shift_step = step;
        self
    }
}
```

#### 3. [template.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/dock_splitter/template.rs)
- Update `DockSplitterTemplateHandlers` and the `DockSplitterTemplate` trait's signature.
- Apply `.track_focus()`, `.tab_index()`, and `.on_key_down()` hooks to the splitter hit target.
- When `focused`, render the default visible line using the active highlight theme (`appearance.hover_color`).

```rust
pub type DockSplitterKeyDownHandler = Box<dyn Fn(&gpui::KeyDownEvent, &mut Window, &mut App) + 'static>;

pub struct DockSplitterTemplateHandlers {
    pub hover: DockSplitterHoverHandler,
    pub mouse_down: DockSplitterMouseDownHandler,
    pub mouse_up: DockSplitterMouseUpHandler,
    pub mouse_up_out: DockSplitterMouseUpHandler,
    pub key_down: DockSplitterKeyDownHandler, // NEW
}

// In ThemedDockSplitterTemplate::render:
let hit_target = match model.orientation {
    SplitterOrientation::Vertical => div()
        .id(model.id.clone())
        .absolute()
        .left(px(-half_inset))
        .top(px(0.0))
        .bottom(px(0.0))
        .w(px(appearance.hit_target_px))
        .track_focus(model.focus_handle) // NEW: register for focus ring tabbing
        .tab_index(if model.enabled { 0 } else { -1 }) // NEW
        .on_key_down(model.focus_handle.clone(), handlers.key_down) // NEW: wire keys
        .on_hover(hover)
        .on_mouse_down(MouseButton::Left, mouse_down)
        ...
```

---

## 2. Unifying into a Single ThemedDockSplitterTemplate with Configurable Grip

Instead of creating two parallel templates, we unify them into a single `ThemedDockSplitterTemplate` containing `thumb_color: Option<Hsla>`. This eliminates duplicate boilerplate while maintaining complete customization of the grip's color.

```rust
pub struct ThemedDockSplitterTemplate {
    thumb_color: Option<Hsla>,
}

impl ThemedDockSplitterTemplate {
    pub fn new() -> Self {
        Self { thumb_color: None }
    }

    pub fn with_thumb(thumb_color: Hsla) -> Self {
        Self { thumb_color: Some(thumb_color) }
    }
}

impl Default for ThemedDockSplitterTemplate {
    fn default() -> Self {
        Self::new()
    }
}

pub fn default_dock_splitter_template() -> Arc<dyn DockSplitterTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn DockSplitterTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedDockSplitterTemplate::new())).clone()
}

impl DockSplitterTemplate for ThemedDockSplitterTemplate {
    fn render(
        &self,
        model: &DockSplitterRenderModel<'_>,
        appearance: &DockSplitterAppearance,
        handlers: DockSplitterTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        render_splitter(model, appearance, handlers, self.thumb_color)
    }
}

fn render_splitter(
    model: &DockSplitterRenderModel<'_>,
    appearance: &DockSplitterAppearance,
    handlers: DockSplitterTemplateHandlers,
    thumb_color: Option<Hsla>,
) -> Stateful<Div> {
    let DockSplitterTemplateHandlers { hover, mouse_down, mouse_up, mouse_up_out, key_down } = handlers;
    
    let line_color = if model.dragging || model.hovered || model.focused {
        appearance.hover_color
    } else {
        appearance.line_color
    };
    
    let accent = thumb_color.unwrap_or(appearance.hover_color);
    let half_inset = ((appearance.hit_target_px - appearance.visible_line_px) * 0.5).max(0.0);
    
    let mut root = div().id(format!("{}-layout", model.id)).relative().flex_shrink_0();
    root = match model.orientation {
        SplitterOrientation::Vertical => root.w(px(appearance.visible_line_px)).h_full(),
        SplitterOrientation::Horizontal => root.h(px(appearance.visible_line_px)).w_full(),
    };

    // Custom grip overlay (pill shape), rendered only if thumb_color is enabled
    let grip = if thumb_color.is_some() {
        Some(
            div().absolute().inset_0().flex().justify_center().items_center().child(
                div()
                    .rounded(px(8.0))
                    .bg(if model.hovered || model.dragging || model.focused {
                        accent
                    } else {
                        accent.opacity(0.4)
                    })
                    .when(model.orientation == SplitterOrientation::Vertical, |this| {
                        this.w(px(4.0)).h(px(36.0))
                    })
                    .when(model.orientation == SplitterOrientation::Horizontal, |this| {
                        this.w(px(36.0)).h(px(4.0))
                    })
            )
        )
    } else {
        None
    };

    // Symmetrical focus outline decoration around the splitter interaction area
    let focus_highlight = if model.focused && thumb_color.is_some() {
        Some(
            div()
                .absolute()
                .border_2()
                .border_color(accent.opacity(0.5))
                .rounded_sm()
                .when(model.orientation == SplitterOrientation::Vertical, |this| {
                    this.inset_y_0().left(px(-2.0)).right(px(-2.0))
                })
                .when(model.orientation == SplitterOrientation::Horizontal, |this| {
                    this.inset_x_0().top(px(-2.0)).bottom(px(-2.0))
                })
        )
    } else {
        None
    };

        let hit_target = match model.orientation {
            SplitterOrientation::Vertical => div()
                .id(model.id.clone())
                .absolute()
                .left(px(-half_inset))
                .top(px(0.0))
                .bottom(px(0.0))
                .w(px(appearance.hit_target_px))
                .track_focus(model.focus_handle)
                .tab_index(if model.enabled { 0 } else { -1 })
                .on_key_down(model.focus_handle.clone(), key_down)
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
                .children(grip)
                .children(focus_highlight),
            SplitterOrientation::Horizontal => div()
                .id(model.id.clone())
                .absolute()
                .top(px(-half_inset))
                .left(px(0.0))
                .right(px(0.0))
                .h(px(appearance.hit_target_px))
                .track_focus(model.focus_handle)
                .tab_index(if model.enabled { 0 } else { -1 })
                .on_key_down(model.focus_handle.clone(), key_down)
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
                )
                .children(grip)
                .children(focus_highlight),
        };

        root.child(hit_target)
    }
}
```

---

## 3. Gallery Integration Spec: `apps/gallery/src/gallery/panes/dock_panel/pane.rs`

Using these improvements, we refactor the Gallery demo page:
1. **Remove** `LeftSplitterTemplate` entirely from the local code.
2. Use the unified `ThemedDockSplitterTemplate::with_thumb(accent)` (enabling the grip/pill overlay with custom theme color) from the SDK for the splitters.
3. Make all four splitters fully keyboard focusable and adjustable with standard Arrow keys.

```rust
// Replace custom template with unified SDK template showing custom color grip
let left_splitter = DockSplitter::new("dock-panel-left-splitter", SplitterOrientation::Vertical)
    .template(Arc::new(ThemedDockSplitterTemplate::with_thumb(accent)))
    .theme(splitter_theme.clone())
    .spawn(cx);

let top_splitter = DockSplitter::new("dock-panel-top-splitter", SplitterOrientation::Horizontal)
    .template(Arc::new(ThemedDockSplitterTemplate::with_thumb(accent)))
    .theme(splitter_theme.clone())
    .spawn(cx);

let right_splitter = DockSplitter::new("dock-panel-right-splitter", SplitterOrientation::Vertical)
    .template(Arc::new(ThemedDockSplitterTemplate::with_thumb(accent)))
    .theme(splitter_theme.clone())
    .spawn(cx);

let bottom_splitter = DockSplitter::new("dock-panel-bottom-splitter", SplitterOrientation::Horizontal)
    .template(Arc::new(ThemedDockSplitterTemplate::with_thumb(accent)))
    .theme(splitter_theme)
    .spawn(cx);
```

---

## 4. Verification Plan

1. **Focus Rings / Tabbing:**
   - Compile and run the Gallery application (`just gallery`).
   - Navigate to the `DockSplitter` demo page.
   - Press `Tab` repeatedly to ensure focus cycles through the four splitter panels.
   - Verify that when a splitter is focused, its outline/symmetrical highlight indicator turns active.

2. **Keyboard Resizing:**
   - Select a focused splitter.
   - Press Arrow keys (Left/Up to shrink/decrease, Right/Down to grow/increase across both orientations) to adjust the layout size.
   - Hold `Shift` with the Arrow keys to confirm accelerated movement (10px increments).
