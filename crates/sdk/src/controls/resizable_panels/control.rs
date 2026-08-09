use gpui::{
    Context, DragMoveEvent, EventEmitter, FocusHandle, FocusOutEvent, Hsla, IntoElement, KeyDownEvent, MouseButton,
    MouseDownEvent, MouseUpEvent, ParentElement, Pixels, Render, SharedString, Subscription, Window, div, prelude::*,
};

use super::{
    math::{apply_pair_collapse_px, apply_pair_delta_px, content_axis_size, solve_layout_px_with_min_overrides},
    model::{
        PanelHideMode, PanelLayoutState, PanelSize, ResizeCollapseDirection, ResizeCollapseMode,
        ResizeHandleVisibility, ResizablePanelsBuilder, ResizablePanelsModel, ResizablePanelsOrientation,
        ResizablePanelsRenderModel,
    },
};
use crate::animation::{DEFAULT_TRANSITION_DURATION, VisualTransition};
use crate::theme::InteractionState;

#[derive(Clone, Debug)]
pub struct ResizablePanelsHandleDrag {
    pub(crate) id: SharedString,
    pub(crate) handle_index: usize,
}

impl Render for ResizablePanelsHandleDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ResizablePanelsEvent {
    ResizeStart,
    SizesChanged { sizes_px: Vec<f32> },
    ResizeEnd { sizes_px: Vec<f32> },
    PanelHiddenChanged { panel_index: usize, hidden: bool },
    HandleFocusChanged { handle_index: usize, focused: bool },
    HandleHoverChanged { handle_index: usize, hovered: bool },
    EnabledChanged { enabled: bool },
}

#[derive(Clone, Debug)]
struct CollapseRestoreState {
    handle_index: usize,
    target_index: usize,
    left_state: PanelLayoutState,
    right_state: PanelLayoutState,
    left_px: f32,
    right_px: f32,
    left_min_override: bool,
    right_min_override: bool,
    content_axis_px: f32,
}

impl CollapseRestoreState {
    fn intersects_pair(&self, handle_index: usize) -> bool {
        let left = handle_index;
        let right = handle_index + 1;
        let restore_left = self.handle_index;
        let restore_right = self.handle_index + 1;

        restore_left == left || restore_left == right || restore_right == left || restore_right == right
    }
}

pub struct ResizablePanels {
    model: ResizablePanelsModel,
    layout_states: Vec<PanelLayoutState>,
    collapsed_min_overrides: Vec<bool>,
    collapse_restore: Vec<Option<CollapseRestoreState>>,
    panel_hide_restore: Vec<Option<CollapseRestoreState>>,
    panel_sizes_px: Vec<f32>,
    handle_focuses: Vec<FocusHandle>,
    handle_focus_subscriptions: Vec<Subscription>,
    emitted_focused_handles: Vec<bool>,
    hovered_handle: Option<usize>,
    dragging_handle: Option<usize>,
    drag_start_axis_px: f32,
    drag_start_states: Vec<PanelLayoutState>,
    /// Content-axis size at drag start; stabilizes weight panes during handle moves.
    drag_content_axis_px: Option<f32>,
    measured_size: Option<gpui::Size<Pixels>>,
    transitions: Vec<VisualTransition>,
}

impl EventEmitter<ResizablePanelsEvent> for ResizablePanels {}

impl ResizablePanels {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ResizablePanelsBuilder {
        ResizablePanelsBuilder::new(id)
    }

    pub fn horizontal(id: impl Into<SharedString>) -> ResizablePanelsBuilder {
        ResizablePanelsBuilder::horizontal(id)
    }

    pub fn vertical(id: impl Into<SharedString>) -> ResizablePanelsBuilder {
        ResizablePanelsBuilder::vertical(id)
    }

    pub(crate) fn from_builder(builder: ResizablePanelsBuilder, cx: &mut Context<Self>) -> Self {
        let layout_states: Vec<PanelLayoutState> =
            builder.model.panels.iter().map(PanelLayoutState::from_spec).collect();

        let layout_states = if layout_states.len() == builder.model.panels.len() {
            layout_states
        } else {
            vec![PanelLayoutState::Weight(1.0)]
        };

        let handle_count = builder.model.panels.len().saturating_sub(1);
        let handle_focuses = (0..handle_count).map(|_| cx.focus_handle()).collect();

        let collapsed_min_overrides = vec![false; builder.model.panels.len()];
        let collapse_restore = vec![None; handle_count];
        let panel_hide_restore = vec![None; builder.model.panels.len()];
        let panel_sizes_px =
            solve_layout_px_with_min_overrides(&builder.model.panels, &layout_states, &collapsed_min_overrides, 1.0);

        let duration = if builder.model.animated {
            DEFAULT_TRANSITION_DURATION
        } else {
            std::time::Duration::ZERO
        };
        let transitions = vec![VisualTransition::new(1.0, duration); builder.model.panels.len()];

        Self {
            model: builder.model,
            layout_states,
            collapsed_min_overrides,
            collapse_restore,
            panel_hide_restore,
            panel_sizes_px,
            handle_focuses,
            handle_focus_subscriptions: Vec::new(),
            emitted_focused_handles: vec![false; handle_count],
            hovered_handle: None,
            dragging_handle: None,
            drag_start_axis_px: 0.0,
            drag_start_states: Vec::new(),
            drag_content_axis_px: None,
            measured_size: None,
            transitions,
        }
    }

    /// True when the main layout axis uses an explicit frame size instead of prepaint measurement.
    fn main_axis_frame_pinned(&self) -> bool {
        match self.model.orientation {
            ResizablePanelsOrientation::Horizontal => self.model.frame_width.is_some(),
            ResizablePanelsOrientation::Vertical => self.model.frame_height.is_some(),
        }
    }

    pub fn set_measured_size(&mut self, size: gpui::Size<Pixels>, cx: &mut Context<Self>) {
        if self.main_axis_frame_pinned() {
            return;
        }
        if self.measured_size == Some(size) {
            return;
        }
        self.measured_size = Some(size);
        cx.notify();
    }

    /// Clears cached prepaint measurement so the next layout pass can re-measure the parent.
    pub fn invalidate_measured_size(&mut self, cx: &mut Context<Self>) {
        if self.main_axis_frame_pinned() {
            return;
        }
        if self.measured_size.take().is_some() {
            self.refresh_panel_sizes_px();
            cx.notify();
        }
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }
        self.model.enabled = enabled;
        if !enabled {
            if let Some(handle_index) = self.hovered_handle.take() {
                cx.emit(ResizablePanelsEvent::HandleHoverChanged { handle_index, hovered: false });
            }
            if self.dragging_handle.take().is_some() {
                cx.emit(ResizablePanelsEvent::ResizeEnd { sizes_px: self.panel_sizes_px.clone() });
            }
            self.drag_content_axis_px = None;
            for index in 0..self.emitted_focused_handles.len() {
                self.emit_handle_focus_changed(index, false, cx);
            }
            self.clear_all_restore_state(cx);
        }
        cx.emit(ResizablePanelsEvent::EnabledChanged { enabled });
        cx.notify();
    }

    pub fn set_show_handle(&mut self, show_handle: bool, cx: &mut Context<Self>) {
        let visibility = if show_handle {
            ResizeHandleVisibility::Always
        } else {
            ResizeHandleVisibility::Hidden
        };
        self.set_handle_visibility(visibility, cx);
    }

    pub fn set_handle_visibility(&mut self, visibility: ResizeHandleVisibility, cx: &mut Context<Self>) {
        if self.model.handle_visibility == visibility {
            return;
        }
        self.model.handle_visibility = visibility;
        cx.notify();
    }

    pub fn set_double_click_collapse(
        &mut self,
        behavior: Option<super::model::ResizeCollapseBehavior>,
        cx: &mut Context<Self>,
    ) {
        if self.model.double_click_collapse == behavior {
            return;
        }
        self.model.double_click_collapse = behavior;
        self.collapse_restore.fill(None);
        cx.notify();
    }

    pub fn set_resize_handle(&mut self, size: super::model::ResizeHandleSize, cx: &mut Context<Self>) {
        if self.model.resize_handle == size {
            return;
        }
        self.model.resize_handle = size;
        cx.notify();
    }

    pub fn set_theme(&mut self, theme: std::sync::Arc<dyn super::theme::ResizablePanelsTheme>, cx: &mut Context<Self>) {
        if std::sync::Arc::ptr_eq(&self.model.theme, &theme) {
            cx.notify();
            return;
        }
        self.model.theme = theme;
        cx.notify();
    }

    pub fn set_panel_background(&mut self, panel_index: usize, background: Option<Hsla>, cx: &mut Context<Self>) {
        let Some(spec) = self.model.panels.get_mut(panel_index) else {
            return;
        };
        if spec.background == background {
            return;
        }
        spec.background = background;
        cx.notify();
    }

    pub fn set_template(
        &mut self,
        template: std::sync::Arc<dyn super::template::ResizablePanelsTemplate>,
        cx: &mut Context<Self>,
    ) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_frame_size(&mut self, width: Pixels, height: Pixels, cx: &mut Context<Self>) {
        if self.model.frame_width == Some(width) && self.model.frame_height == Some(height) {
            return;
        }
        self.model.frame_width = Some(width);
        self.model.frame_height = Some(height);
        self.refresh_panel_sizes_px();
        cx.notify();
    }

    /// Sets weight coefficients for weight-only panel strips.
    pub fn set_weights(&mut self, weights: Vec<f32>, cx: &mut Context<Self>) {
        if weights.len() != self.model.panels.len() || weights.is_empty() {
            return;
        }
        if !self.model.panels.iter().all(|spec| matches!(spec.size, PanelSize::Weight(_))) {
            return;
        }

        let changed = self.layout_states.iter().zip(weights.iter()).any(|(current, next)| match current {
            PanelLayoutState::Weight(weight) => (weight - next).abs() >= f32::EPSILON,
            PanelLayoutState::Absolute(px) => (px - next).abs() >= f32::EPSILON,
        });
        if !changed {
            return;
        }

        for (state, weight) in self.layout_states.iter_mut().zip(weights.iter()) {
            *state = PanelLayoutState::Weight(weight.max(0.0));
        }
        self.clear_all_restore_state(cx);

        self.refresh_panel_sizes_px();
        cx.emit(ResizablePanelsEvent::SizesChanged { sizes_px: self.panel_sizes_px.clone() });
        cx.notify();
    }

    /// Main-axis pixel size of each panel after the current layout solve.
    pub fn panel_sizes_px(&self) -> Vec<f32> {
        self.panel_sizes_px.clone()
    }

    pub fn layout_states(&self) -> Vec<PanelLayoutState> {
        self.layout_states.clone()
    }

    pub fn orientation(&self) -> ResizablePanelsOrientation {
        self.model.orientation
    }

    pub fn enabled(&self) -> bool {
        self.model.enabled
    }

    pub fn animated(&self) -> bool {
        self.model.animated
    }

    pub fn set_animated(&mut self, animated: bool, cx: &mut Context<Self>) {
        if self.model.animated == animated {
            return;
        }
        self.model.animated = animated;
        let duration = if animated {
            DEFAULT_TRANSITION_DURATION
        } else {
            std::time::Duration::ZERO
        };
        for transition in &mut self.transitions {
            *transition = VisualTransition::new(transition.progress(), duration);
        }
        cx.notify();
    }

    pub fn show_handle(&self) -> bool {
        self.model.handle_visibility == ResizeHandleVisibility::Always
    }

    pub fn handle_visibility(&self) -> ResizeHandleVisibility {
        self.model.handle_visibility
    }

    pub fn double_click_collapse(&self) -> Option<super::model::ResizeCollapseBehavior> {
        self.model.double_click_collapse
    }

    pub fn is_panel_hidden(&self, panel_index: usize) -> bool {
        self.panel_hide_restore.get(panel_index).is_some_and(Option::is_some)
    }

    /// Hides a panel and records its layout for a later [`Self::show_panel`] call.
    ///
    /// `PanelHideMode::ToMinSize` leaves the panel in the interactive resize model;
    /// `PanelHideMode::Completely` removes its visible extent and requires an
    /// explicit show operation for recovery.
    pub fn hide_panel(&mut self, panel_index: usize, mode: PanelHideMode, cx: &mut Context<Self>) {
        if panel_index >= self.layout_states.len() || self.is_panel_hidden(panel_index) {
            return;
        }
        let Some(handle_index) = self.hide_handle_index(panel_index) else {
            return;
        };
        self.clear_pair_restore_state(handle_index, cx);
        let target_px = match mode {
            PanelHideMode::ToMinSize => self.model.panels[panel_index].min_px.unwrap_or(0.0),
            PanelHideMode::Completely => 0.0,
        };
        let restore = self.current_restore_state(handle_index, panel_index);
        let content_axis_px = self.content_axis_size_px();
        let changed = apply_pair_collapse_px(
            &self.model.panels,
            &mut self.layout_states,
            &mut self.collapsed_min_overrides,
            handle_index,
            panel_index,
            target_px,
            mode == PanelHideMode::Completely,
            content_axis_px,
        );
        if !changed {
            return;
        }

        self.panel_hide_restore[panel_index] = Some(restore);
        self.transitions[panel_index].set_target(0.0);
        self.refresh_panel_sizes_px();
        let sizes_px = self.panel_sizes_px.clone();
        cx.emit(ResizablePanelsEvent::ResizeStart);
        if !self.model.animated {
            cx.emit(ResizablePanelsEvent::SizesChanged { sizes_px: sizes_px.clone() });
            cx.emit(ResizablePanelsEvent::ResizeEnd { sizes_px });
        }
        cx.emit(ResizablePanelsEvent::PanelHiddenChanged { panel_index, hidden: true });
        cx.notify();
    }

    /// Restores a panel previously hidden with [`Self::hide_panel`].
    pub fn show_panel(&mut self, panel_index: usize, cx: &mut Context<Self>) {
        let Some(restore) = self.panel_hide_restore.get(panel_index).and_then(Option::as_ref).cloned() else {
            return;
        };
        // When animated, suppress settle events here — render defers them when the transition ends.
        if self.apply_restore_state(restore, !self.model.animated, cx) {
            if let Some(panel_restore) = self.panel_hide_restore.get_mut(panel_index) {
                *panel_restore = None;
            }
            self.transitions[panel_index].set_target(1.0);
            self.refresh_panel_sizes_px();
            if self.model.animated {
                cx.emit(ResizablePanelsEvent::ResizeStart);
            }
            cx.emit(ResizablePanelsEvent::PanelHiddenChanged { panel_index, hidden: false });
            cx.notify();
        }
    }

    pub fn toggle_panel_hidden(&mut self, panel_index: usize, mode: PanelHideMode, cx: &mut Context<Self>) {
        if self.is_panel_hidden(panel_index) {
            self.show_panel(panel_index, cx);
        } else {
            self.hide_panel(panel_index, mode, cx);
        }
    }

    fn axis_position(&self, position: gpui::Point<Pixels>) -> f32 {
        match self.model.orientation {
            ResizablePanelsOrientation::Horizontal => position.x.as_f32(),
            ResizablePanelsOrientation::Vertical => position.y.as_f32(),
        }
    }

    fn main_axis_px(&self) -> f32 {
        match (self.model.frame_width, self.model.frame_height, self.model.orientation) {
            (Some(width), _, ResizablePanelsOrientation::Horizontal) => width.as_f32(),
            (_, Some(height), ResizablePanelsOrientation::Vertical) => height.as_f32(),
            (None, _, ResizablePanelsOrientation::Horizontal) => {
                self.measured_size.map(|s| s.width.as_f32()).unwrap_or(1.0)
            }
            (_, None, ResizablePanelsOrientation::Vertical) => {
                self.measured_size.map(|s| s.height.as_f32()).unwrap_or(1.0)
            }
        }
    }

    fn content_axis_size_px(&self) -> f32 {
        let panel_count = self.layout_states.len();
        content_axis_size(self.main_axis_px(), 0.0, panel_count)
    }

    fn drag_content_axis_size_px(&self) -> f32 {
        self.drag_content_axis_px.unwrap_or_else(|| self.content_axis_size_px())
    }

    fn apply_pair_delta(&mut self, index: usize, delta_px: f32, cx: &mut Context<Self>) -> bool {
        if index + 1 >= self.layout_states.len() {
            return false;
        }

        self.clear_pair_restore_state(index, cx);
        let content_main_px = self.drag_content_axis_size_px();
        apply_pair_delta_px(&self.model.panels, &mut self.layout_states, index, delta_px, content_main_px)
    }

    fn clear_all_restore_state(&mut self, cx: &mut Context<Self>) {
        self.collapsed_min_overrides.fill(false);
        self.collapse_restore.fill(None);
        for (panel_index, restore) in self.panel_hide_restore.iter_mut().enumerate() {
            if restore.take().is_some() {
                cx.emit(ResizablePanelsEvent::PanelHiddenChanged { panel_index, hidden: false });
            }
        }
    }

    fn clear_pair_restore_state(&mut self, index: usize, cx: &mut Context<Self>) {
        for panel_index in [index, index + 1] {
            if let Some(override_min) = self.collapsed_min_overrides.get_mut(panel_index) {
                *override_min = false;
            }
        }
        self.clear_collapse_restore_for_pair(index);
        for panel_index in [index, index + 1] {
            if let Some(restore) = self.panel_hide_restore.get_mut(panel_index)
                && restore.take().is_some()
            {
                cx.emit(ResizablePanelsEvent::PanelHiddenChanged { panel_index, hidden: false });
            }
        }
    }

    fn clear_collapse_restore_for_pair(&mut self, pair_handle_index: usize) {
        for restore in &mut self.collapse_restore {
            if restore.as_ref().is_some_and(|restore| restore.intersects_pair(pair_handle_index)) {
                *restore = None;
            }
        }
    }

    fn hide_handle_index(&self, panel_index: usize) -> Option<usize> {
        if self.layout_states.len() < 2 {
            return None;
        }
        if panel_index == 0 {
            Some(0)
        } else {
            Some(panel_index - 1)
        }
    }

    fn collapse_target_index(&self, handle_index: usize, direction: ResizeCollapseDirection) -> Option<usize> {
        match (self.model.orientation, direction) {
            (ResizablePanelsOrientation::Horizontal, ResizeCollapseDirection::Left)
            | (ResizablePanelsOrientation::Vertical, ResizeCollapseDirection::Top) => Some(handle_index),
            (ResizablePanelsOrientation::Horizontal, ResizeCollapseDirection::Right)
            | (ResizablePanelsOrientation::Vertical, ResizeCollapseDirection::Bottom) => Some(handle_index + 1),
            _ => None,
        }
    }

    fn collapse_target_px(&self, target_index: usize, mode: ResizeCollapseMode) -> f32 {
        match mode {
            ResizeCollapseMode::ToMinSize => self.model.panels[target_index].min_px.unwrap_or(0.0),
            ResizeCollapseMode::Completely => 0.0,
        }
    }

    fn current_restore_state(&self, handle_index: usize, target_index: usize) -> CollapseRestoreState {
        CollapseRestoreState {
            handle_index,
            target_index,
            left_state: self.layout_states[handle_index],
            right_state: self.layout_states[handle_index + 1],
            left_px: self.panel_sizes_px.get(handle_index).copied().unwrap_or(0.0),
            right_px: self.panel_sizes_px.get(handle_index + 1).copied().unwrap_or(0.0),
            left_min_override: self.collapsed_min_overrides.get(handle_index).copied().unwrap_or(false),
            right_min_override: self.collapsed_min_overrides.get(handle_index + 1).copied().unwrap_or(false),
            content_axis_px: self.content_axis_size_px(),
        }
    }

    fn apply_restore_state(
        &mut self,
        restore: CollapseRestoreState,
        emit_resize_events: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        if restore.handle_index + 1 >= self.layout_states.len() {
            return false;
        }

        self.layout_states[restore.handle_index] = restore.left_state;
        self.layout_states[restore.handle_index + 1] = restore.right_state;
        if let Some(override_min) = self.collapsed_min_overrides.get_mut(restore.handle_index) {
            *override_min = restore.left_min_override;
        }
        if let Some(override_min) = self.collapsed_min_overrides.get_mut(restore.handle_index + 1) {
            *override_min = restore.right_min_override;
        }
        self.scale_restored_pair_to_current_axis(&restore);
        self.clear_collapse_restore_for_pair(restore.handle_index);
        self.refresh_panel_sizes_px();
        if emit_resize_events {
            let sizes_px = self.panel_sizes_px.clone();
            cx.emit(ResizablePanelsEvent::ResizeStart);
            cx.emit(ResizablePanelsEvent::SizesChanged { sizes_px: sizes_px.clone() });
            cx.emit(ResizablePanelsEvent::ResizeEnd { sizes_px });
        }
        cx.notify();
        true
    }

    fn scale_restored_pair_to_current_axis(&mut self, restore: &CollapseRestoreState) {
        let current_axis_px = self.content_axis_size_px();
        if (current_axis_px - restore.content_axis_px).abs() < f32::EPSILON || restore.content_axis_px <= f32::EPSILON {
            return;
        }

        let scale = current_axis_px / restore.content_axis_px;
        for panel_index in [restore.handle_index, restore.handle_index + 1] {
            if let Some(PanelLayoutState::Absolute(px)) = self.layout_states.get_mut(panel_index) {
                *px *= scale;
            }
        }
    }

    fn apply_double_click_collapse(&mut self, index: usize, cx: &mut Context<Self>) -> bool {
        let Some(behavior) = self.model.double_click_collapse else {
            return false;
        };
        let Some(target_index) = self.collapse_target_index(index, behavior.direction) else {
            return false;
        };
        if target_index >= self.layout_states.len() || index + 1 >= self.layout_states.len() {
            return false;
        }

        if let Some(restore) = self
            .collapse_restore
            .get(index)
            .and_then(Option::as_ref)
            .filter(|restore| restore.handle_index == index && restore.target_index == target_index)
            .cloned()
        {
            return self.apply_restore_state(restore, true, cx);
        }

        self.clear_pair_restore_state(index, cx);
        let restore = self.current_restore_state(index, target_index);
        let target_px = self.collapse_target_px(target_index, behavior.mode);
        let content_axis_px = self.content_axis_size_px();
        let changed = apply_pair_collapse_px(
            &self.model.panels,
            &mut self.layout_states,
            &mut self.collapsed_min_overrides,
            index,
            target_index,
            target_px,
            behavior.mode == ResizeCollapseMode::Completely,
            content_axis_px,
        );
        if !changed {
            return false;
        }

        if let Some(collapse_restore) = self.collapse_restore.get_mut(index) {
            *collapse_restore = Some(restore);
        }
        self.refresh_panel_sizes_px();
        let sizes_px = self.panel_sizes_px.clone();
        cx.emit(ResizablePanelsEvent::ResizeStart);
        cx.emit(ResizablePanelsEvent::SizesChanged { sizes_px: sizes_px.clone() });
        cx.emit(ResizablePanelsEvent::ResizeEnd { sizes_px });
        cx.notify();
        true
    }

    fn emit_sizes_changed_if_needed(&mut self, changed: bool, cx: &mut Context<Self>) {
        if !changed {
            return;
        }
        cx.emit(ResizablePanelsEvent::SizesChanged { sizes_px: self.panel_sizes_px.clone() });
        cx.notify();
    }

    pub(crate) fn handle_handle_mouse_down(
        &mut self,
        index: usize,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.model.enabled || index >= self.handle_focuses.len() {
            return;
        }

        self.handle_focuses[index].focus(window, cx);
        if event.click_count >= 2 && self.apply_double_click_collapse(index, cx) {
            self.dragging_handle = None;
            self.drag_content_axis_px = None;
            window.prevent_default();
            cx.stop_propagation();
            return;
        }

        self.dragging_handle = Some(index);
        self.drag_start_axis_px = self.axis_position(event.position);
        self.drag_start_states = self.layout_states.clone();
        self.drag_content_axis_px = Some(self.content_axis_size_px());
        cx.emit(ResizablePanelsEvent::ResizeStart);
        cx.notify();
    }

    pub(crate) fn handle_handle_hover(&mut self, index: usize, hovered: &bool, cx: &mut Context<Self>) {
        if !self.model.enabled || index >= self.handle_focuses.len() {
            return;
        }

        let next = if *hovered {
            Some(index)
        } else if self.hovered_handle == Some(index) {
            None
        } else {
            return;
        };
        if self.hovered_handle == next {
            return;
        }
        self.hovered_handle = next;
        cx.emit(ResizablePanelsEvent::HandleHoverChanged { handle_index: index, hovered: *hovered });
        cx.notify();
    }

    fn emit_handle_focus_changed(&mut self, index: usize, focused: bool, cx: &mut Context<Self>) -> bool {
        let Some(current) = self.emitted_focused_handles.get_mut(index) else {
            return false;
        };
        if *current == focused {
            return false;
        }
        *current = focused;
        cx.emit(ResizablePanelsEvent::HandleFocusChanged { handle_index: index, focused });
        true
    }

    fn handle_handle_focus_in(&mut self, index: usize, _window: &mut Window, cx: &mut Context<Self>) {
        if self.emit_handle_focus_changed(index, true, cx) {
            cx.notify();
        }
    }

    fn handle_handle_focus_out(
        &mut self,
        index: usize,
        _: FocusOutEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.emit_handle_focus_changed(index, false, cx) {
            cx.notify();
        }
    }

    pub(crate) fn handle_drag_move(
        &mut self,
        event: &DragMoveEvent<ResizablePanelsHandleDrag>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.model.enabled {
            return;
        }

        let drag = event.drag(cx);
        if drag.id != self.model.id {
            return;
        }
        let Some(index) = self.dragging_handle else {
            return;
        };
        if index != drag.handle_index || index + 1 >= self.layout_states.len() {
            return;
        }

        self.layout_states.clone_from(&self.drag_start_states);
        let delta_px = self.axis_position(event.event.position) - self.drag_start_axis_px;
        let changed = self.apply_pair_delta(index, delta_px, cx);
        if changed {
            self.refresh_panel_sizes_px();
        }
        self.emit_sizes_changed_if_needed(changed, cx);
    }

    pub(crate) fn finish_drag(&mut self, event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || event.button != MouseButton::Left {
            return;
        }
        if self.dragging_handle.take().is_none() {
            return;
        }
        self.drag_content_axis_px = None;
        cx.emit(ResizablePanelsEvent::ResizeEnd { sizes_px: self.panel_sizes_px.clone() });
        cx.notify();
    }

    pub(crate) fn handle_handle_key_down(
        &mut self,
        index: usize,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.model.enabled || index + 1 >= self.layout_states.len() {
            return;
        }

        let step_px = if event.keystroke.modifiers.shift {
            self.model.keyboard_shift_step
        } else {
            self.model.keyboard_step
        };

        let delta_px = match (self.model.orientation, event.keystroke.key.as_str()) {
            (ResizablePanelsOrientation::Horizontal, "left") => -step_px,
            (ResizablePanelsOrientation::Horizontal, "right") => step_px,
            (ResizablePanelsOrientation::Vertical, "up") => -step_px,
            (ResizablePanelsOrientation::Vertical, "down") => step_px,
            _ => return,
        };

        self.handle_focuses[index].focus(window, cx);
        if self.apply_pair_delta(index, delta_px, cx) {
            self.refresh_panel_sizes_px();
            let sizes_px = self.panel_sizes_px.clone();
            cx.emit(ResizablePanelsEvent::ResizeStart);
            cx.emit(ResizablePanelsEvent::SizesChanged { sizes_px: sizes_px.clone() });
            cx.emit(ResizablePanelsEvent::ResizeEnd { sizes_px });
            window.prevent_default();
            cx.stop_propagation();
            cx.notify();
        }
    }

    fn refresh_panel_sizes_px(&mut self) {
        let raw_sizes = solve_layout_px_with_min_overrides(
            &self.model.panels,
            &self.layout_states,
            &self.collapsed_min_overrides,
            self.content_axis_size_px(),
        );

        if !self.model.animated || self.transitions.is_empty() {
            self.panel_sizes_px = raw_sizes;
            return;
        }

        let mut animated_sizes = raw_sizes.clone();
        let total_axis = self.content_axis_size_px();

        if self.model.panels.len() == 2 {
            let p0 = self.transitions[0].progress();
            let p1 = self.transitions[1].progress();

            if p0 < 1.0 || self.transitions[0].is_animating() {
                let full_w0 = if self.is_panel_hidden(0) {
                    self.panel_hide_restore
                        .get(0)
                        .and_then(|opt| opt.as_ref())
                        .map(|r| r.left_px)
                        .unwrap_or(raw_sizes[0])
                } else {
                    raw_sizes[0]
                };
                let w0 = full_w0 * p0;
                animated_sizes[0] = w0;
                animated_sizes[1] = (total_axis - w0).max(0.0);
            } else if p1 < 1.0 || self.transitions[1].is_animating() {
                let full_w1 = if self.is_panel_hidden(1) {
                    self.panel_hide_restore
                        .get(1)
                        .and_then(|opt| opt.as_ref())
                        .map(|r| r.right_px)
                        .unwrap_or(raw_sizes[1])
                } else {
                    raw_sizes[1]
                };
                let w1 = full_w1 * p1;
                animated_sizes[1] = w1;
                animated_sizes[0] = (total_axis - w1).max(0.0);
            }
        }

        self.panel_sizes_px = animated_sizes;
    }

    fn render_model(&self) -> ResizablePanelsRenderModel<'_> {
        ResizablePanelsRenderModel {
            id: &self.model.id,
            orientation: self.model.orientation,
            frame_width: self.model.frame_width,
            frame_height: self.model.frame_height,
            show_border: self.model.show_border,
            enabled: self.model.enabled,
            handle_visibility: self.model.handle_visibility,
            double_click_collapse: self.model.double_click_collapse,
            hovered_handle: self.hovered_handle,
            dragging_handle: self.dragging_handle,
            resize_handle: self.model.resize_handle,
            handle_grip: self.model.handle_grip,
            panel_sizes_px: &self.panel_sizes_px,
            panel_hidden: self.panel_hide_restore.iter().map(Option::is_some).collect(),
            panels: &self.model.panels,
            measured_size: self.measured_size,
        }
    }

    fn sync_handle_focus_subscriptions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.handle_focus_subscriptions.len() == self.handle_focuses.len() * 2 {
            return;
        }

        self.handle_focus_subscriptions.clear();
        for (index, focus_handle) in self.handle_focuses.clone().into_iter().enumerate() {
            self.handle_focus_subscriptions.push(cx.on_focus(&focus_handle, window, move |this, window, cx| {
                this.handle_handle_focus_in(index, window, cx);
            }));
            self.handle_focus_subscriptions.push(cx.on_focus_out(
                &focus_handle,
                window,
                move |this, event: FocusOutEvent, window, cx| {
                    this.handle_handle_focus_out(index, event, window, cx);
                },
            ));
        }
    }
}

impl Render for ResizablePanels {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut was_animating = false;
        let mut now_animating = false;

        for transition in &mut self.transitions {
            if transition.is_animating() {
                was_animating = true;
            }
            if transition.sync() {
                now_animating = true;
            }
            transition.schedule_frame(window, cx);
        }

        self.sync_handle_focus_subscriptions(window, cx);
        self.refresh_panel_sizes_px();

        // Never emit from render — subscribers (e.g. studio layout refresh) must not re-enter
        // layout while the element tree is being built. Defer settle events to the effect cycle.
        if was_animating && !now_animating {
            let sizes_px = self.panel_sizes_px.clone();
            let entity = cx.entity();
            cx.defer(move |cx| {
                entity.update(cx, |_, cx| {
                    cx.emit(ResizablePanelsEvent::SizesChanged { sizes_px: sizes_px.clone() });
                    cx.emit(ResizablePanelsEvent::ResizeEnd { sizes_px });
                });
            });
        }

        let look = self.model.theme.resolve(InteractionState { disabled: !self.model.enabled, ..Default::default() });
        let model = self.render_model();
        let template = self.model.template.clone();

        div().size_full().child(template.render(&model, &look, &self.handle_focuses, window, cx))
    }
}
