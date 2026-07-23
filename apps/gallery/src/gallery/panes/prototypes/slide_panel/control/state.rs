use std::time::Instant;

use gpui::{Context, FocusHandle, KeyDownEvent, Point, Pixels, Window, px};

const PANEL_ANIMATION_SECONDS: f32 = 0.22;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::gallery) enum SlidePanelEdge {
    Left,
    Right,
    Top,
    Bottom,
}

impl SlidePanelEdge {
    pub(in crate::gallery) fn label(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Top => "Top",
            Self::Bottom => "Bottom",
        }
    }

    pub(in crate::gallery) fn description(self) -> &'static str {
        match self {
            Self::Left => "Navigation drawer pinned to the leading edge.",
            Self::Right => "Detail panel suited for inspectors and settings.",
            Self::Top => "Command sheet for short-lived announcements or filters.",
            Self::Bottom => "Mobile-style tray for actions and summaries.",
        }
    }

    pub(in crate::gallery) fn is_horizontal_main_axis(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::gallery) enum SlidePanelTopAnchor {
    WindowEdge,
    BelowTopBar,
}

pub(super) const SUPPORTED_TOP_ANCHORS: [SlidePanelTopAnchor; 2] =
    [SlidePanelTopAnchor::WindowEdge, SlidePanelTopAnchor::BelowTopBar];

impl SlidePanelTopAnchor {
    pub(super) fn inset(self) -> gpui::Pixels {
        match self {
            Self::WindowEdge => px(0.0),
            Self::BelowTopBar => gpui_luma::shell::TITLE_BAR_HEIGHT,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct SlidePanelAnimation {
    from: f32,
    to: f32,
    started_at: Instant,
}

#[derive(Clone, Copy, Debug)]
struct SlidePanelResizeSession {
    start_pointer: f32,
    start_size: f32,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::gallery) struct SlidePanelSizeConfig {
    size: f32,
    min: f32,
    max: f32,
}

impl SlidePanelSizeConfig {
    pub(in crate::gallery) fn new(size: f32, min: f32, max: f32) -> Self {
        Self { size, min, max }
    }
}

pub(in crate::gallery) struct SlidePanelState {
    active_edge: Option<SlidePanelEdge>,
    open_progress: f32,
    animation: Option<SlidePanelAnimation>,
    restore_focus: Option<FocusHandle>,
    pending_panel_focus: bool,
    pending_focus_restore: bool,
    backdrop_click_closes: bool,
    top_anchor: SlidePanelTopAnchor,
    size_config: SlidePanelSizeConfig,
    resize_drag: Option<SlidePanelResizeSession>,
    resize_handle_hovered: bool,
}

impl SlidePanelState {
    pub(in crate::gallery) fn new(top_anchor: SlidePanelTopAnchor, size_config: SlidePanelSizeConfig) -> Self {
        Self {
            active_edge: None,
            open_progress: 0.0,
            animation: None,
            restore_focus: None,
            pending_panel_focus: false,
            pending_focus_restore: false,
            backdrop_click_closes: true,
            top_anchor,
            size_config,
            resize_drag: None,
            resize_handle_hovered: false,
        }
    }

    pub(in crate::gallery) fn set_size_config(&mut self, config: SlidePanelSizeConfig) {
        self.size_config = config;
    }

    pub(in crate::gallery) fn main_axis_size(&self, fallback: f32) -> f32 {
        if self.size_config.size > 0.0 {
            self.size_config.size
        } else {
            fallback
        }
    }

    pub(in crate::gallery) fn resize_handle_hovered(&self) -> bool {
        self.resize_handle_hovered
    }

    pub(in crate::gallery) fn set_resize_handle_hovered(&mut self, hovered: bool) -> bool {
        if self.resize_handle_hovered == hovered {
            return false;
        }
        self.resize_handle_hovered = hovered;
        true
    }

    pub(in crate::gallery) fn is_resizing(&self) -> bool {
        self.resize_drag.is_some()
    }

    pub(in crate::gallery) fn begin_resize(&mut self, pointer: Point<Pixels>, edge: SlidePanelEdge) -> bool {
        self.resize_drag = Some(SlidePanelResizeSession {
            start_pointer: main_axis_pointer(pointer, edge),
            start_size: self.size_config.size,
        });
        true
    }

    pub(in crate::gallery) fn update_resize(&mut self, pointer: Point<Pixels>, edge: SlidePanelEdge) -> bool {
        let Some(drag) = self.resize_drag else {
            return false;
        };

        let pointer_main = main_axis_pointer(pointer, edge);
        let delta = match edge {
            SlidePanelEdge::Right | SlidePanelEdge::Bottom => drag.start_pointer - pointer_main,
            SlidePanelEdge::Left | SlidePanelEdge::Top => pointer_main - drag.start_pointer,
        };
        let next = (drag.start_size + delta).clamp(self.size_config.min, self.size_config.max);
        if (next - self.size_config.size).abs() <= f32::EPSILON {
            return false;
        }
        self.size_config.size = next;
        true
    }

    pub(in crate::gallery) fn finish_resize(&mut self) -> bool {
        self.resize_drag.take().is_some()
    }

    pub(in crate::gallery) fn active_edge(&self) -> Option<SlidePanelEdge> {
        self.active_edge
    }

    pub(in crate::gallery) fn open_progress(&self) -> f32 {
        self.open_progress
    }

    pub(in crate::gallery) fn backdrop_click_closes(&self) -> bool {
        self.backdrop_click_closes
    }

    pub(in crate::gallery) fn top_anchor(&self) -> SlidePanelTopAnchor {
        self.top_anchor
    }

    pub(in crate::gallery) fn set_backdrop_click_closes(&mut self, backdrop_click_closes: bool) {
        self.backdrop_click_closes = backdrop_click_closes;
    }

    pub(in crate::gallery) fn open(&mut self, edge: SlidePanelEdge, opener: FocusHandle) {
        self.restore_focus = Some(opener);
        self.active_edge = Some(edge);
        self.pending_panel_focus = true;
        self.pending_focus_restore = false;
        self.start_animation(1.0);
    }

    pub(in crate::gallery) fn request_close(&mut self) -> bool {
        if self.active_edge.is_none() && self.open_progress <= 0.0 {
            return false;
        }

        self.pending_panel_focus = false;
        self.pending_focus_restore = self.restore_focus.is_some();
        self.start_animation(0.0);
        true
    }

    pub(in crate::gallery) fn handle_escape<T>(&mut self, window: &mut Window, cx: &mut Context<T>) -> bool
    where
        T: 'static,
    {
        if self.active_edge.is_none() {
            return false;
        }

        if !self.request_close() {
            return false;
        }

        window.prevent_default();
        cx.stop_propagation();
        true
    }

    pub(in crate::gallery) fn handle_tab_navigation<T>(
        &self,
        primary_focus: FocusHandle,
        extra_focuses: &[FocusHandle],
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<T>,
    ) -> bool
    where
        T: 'static,
    {
        if self.active_edge.is_none() {
            return false;
        }

        let mut focus_handles = Vec::with_capacity(extra_focuses.len() + 1);
        focus_handles.push(primary_focus);
        focus_handles.extend(extra_focuses.iter().cloned());

        let backwards = event.keystroke.modifiers.shift;
        let focused_index = focus_handles.iter().position(|handle| handle.is_focused(window));
        let next_index = match (focused_index, backwards) {
            (Some(0), true) => focus_handles.len() - 1,
            (Some(index), true) => index.saturating_sub(1),
            (Some(index), false) => (index + 1) % focus_handles.len(),
            (None, true) => focus_handles.len() - 1,
            (None, false) => 0,
        };

        focus_handles[next_index].focus(window, cx);
        window.prevent_default();
        cx.stop_propagation();
        true
    }

    pub(in crate::gallery) fn sync_animation(&mut self) -> bool {
        let Some(animation) = self.animation else {
            return false;
        };

        let elapsed = animation.started_at.elapsed().as_secs_f32();
        let linear = (elapsed / PANEL_ANIMATION_SECONDS).clamp(0.0, 1.0);
        let eased = ease_out_cubic(linear);
        self.open_progress = animation.from + ((animation.to - animation.from) * eased);

        if linear >= 1.0 {
            self.open_progress = animation.to;
            self.animation = None;
            if animation.to <= 0.0 {
                self.active_edge = None;
            }
        }

        true
    }

    pub(in crate::gallery) fn schedule_animation_frame<T>(&self, window: &mut Window, cx: &mut Context<T>)
    where
        T: 'static,
    {
        if self.animation.is_none() {
            return;
        }

        cx.on_next_frame(window, |this, _window, cx| {
            let _ = this;
            cx.notify();
        });
    }

    pub(in crate::gallery) fn schedule_pending_focus<T>(
        &mut self,
        window: &mut Window,
        cx: &mut Context<T>,
        primary_focus: FocusHandle,
    ) where
        T: 'static,
    {
        if self.pending_panel_focus && self.open_progress >= 0.999 {
            self.pending_panel_focus = false;
            cx.on_next_frame(window, move |_this, window, cx| {
                primary_focus.focus(window, cx);
            });
        }

        if self.pending_focus_restore && self.active_edge.is_none() && self.open_progress <= 0.001 {
            self.pending_focus_restore = false;
            if let Some(focus) = self.restore_focus.clone() {
                cx.on_next_frame(window, move |_this, window, cx| {
                    focus.focus(window, cx);
                });
            }
        }
    }

    fn start_animation(&mut self, target: f32) {
        let from = self.open_progress;
        if (from - target).abs() <= f32::EPSILON {
            if target == 0.0 {
                self.active_edge = None;
            }
            self.animation = None;
            self.open_progress = target;
            return;
        }

        self.animation = Some(SlidePanelAnimation { from, to: target, started_at: Instant::now() });
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn main_axis_pointer(pointer: Point<Pixels>, edge: SlidePanelEdge) -> f32 {
    match edge {
        SlidePanelEdge::Left | SlidePanelEdge::Right => pointer.x.as_f32(),
        SlidePanelEdge::Top | SlidePanelEdge::Bottom => pointer.y.as_f32(),
    }
}
