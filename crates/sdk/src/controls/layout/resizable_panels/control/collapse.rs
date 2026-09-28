use gpui::Context;

use super::super::math::{apply_pair_collapse_px, solve_layout_px_with_min_overrides};
use super::super::model::{PanelLayoutState, ResizeCollapseDirection, ResizeCollapseMode, ResizablePanelsOrientation};
use super::{ResizablePanels, ResizablePanelsEvent};

#[derive(Clone, Debug)]
pub(super) struct CollapseRestoreState {
    pub(super) handle_index: usize,
    pub(super) target_index: usize,
    pub(super) left_state: PanelLayoutState,
    pub(super) right_state: PanelLayoutState,
    pub(super) left_px: f32,
    pub(super) right_px: f32,
    pub(super) left_min_override: bool,
    pub(super) right_min_override: bool,
    pub(super) content_axis_px: f32,
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

impl ResizablePanels {
    pub(super) fn emit_panel_hidden_changed(&self, panel_index: usize, hidden: bool, cx: &mut Context<Self>) {
        cx.emit(ResizablePanelsEvent::PanelHiddenChanged { panel_index, hidden });
        if let Some(region) = self.region_for_panel_index(panel_index) {
            cx.emit(ResizablePanelsEvent::PanelRegionHiddenChanged { region, panel_index, hidden });
        }
    }

    pub(super) fn clear_all_restore_state(&mut self, cx: &mut Context<Self>) {
        self.collapsed_min_overrides.fill(false);
        self.collapse_restore.fill(None);
        let restored_panels = self
            .panel_hide_restore
            .iter_mut()
            .enumerate()
            .filter_map(|(panel_index, restore)| restore.take().map(|_| panel_index))
            .collect::<Vec<_>>();
        for panel_index in restored_panels {
            self.emit_panel_hidden_changed(panel_index, false, cx);
        }
    }

    pub(super) fn clear_pair_restore_state(&mut self, index: usize, cx: &mut Context<Self>) {
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
                self.emit_panel_hidden_changed(panel_index, false, cx);
            }
        }
    }

    pub(super) fn clear_collapse_restore_for_pair(&mut self, pair_handle_index: usize) {
        for restore in &mut self.collapse_restore {
            if restore.as_ref().is_some_and(|restore| restore.intersects_pair(pair_handle_index)) {
                *restore = None;
            }
        }
    }

    pub(super) fn hide_handle_index(&self, panel_index: usize) -> Option<usize> {
        if self.layout_states.len() < 2 {
            return None;
        }
        if panel_index == 0 {
            Some(0)
        } else {
            Some(panel_index - 1)
        }
    }

    pub(super) fn collapse_target_index(
        &self,
        handle_index: usize,
        direction: ResizeCollapseDirection,
    ) -> Option<usize> {
        match (self.model.orientation, direction) {
            (ResizablePanelsOrientation::Horizontal, ResizeCollapseDirection::Left)
            | (ResizablePanelsOrientation::Vertical, ResizeCollapseDirection::Top) => Some(handle_index),
            (ResizablePanelsOrientation::Horizontal, ResizeCollapseDirection::Right)
            | (ResizablePanelsOrientation::Vertical, ResizeCollapseDirection::Bottom) => Some(handle_index + 1),
            _ => None,
        }
    }

    pub(super) fn collapse_target_px(&self, target_index: usize, mode: ResizeCollapseMode) -> f32 {
        match mode {
            ResizeCollapseMode::ToMinSize => self.model.panels[target_index].min_px.unwrap_or(0.0),
            ResizeCollapseMode::Completely => 0.0,
        }
    }

    pub(super) fn current_restore_state(&self, handle_index: usize, target_index: usize) -> CollapseRestoreState {
        // Save the full layout, not an in-flight animated width, so reversing a
        // transition does not multiply its current progress into the width twice.
        let sizes = solve_layout_px_with_min_overrides(
            &self.model.panels,
            &self.layout_states,
            &self.collapsed_min_overrides,
            self.content_axis_size_px(),
        );
        CollapseRestoreState {
            handle_index,
            target_index,
            left_state: self.layout_states[handle_index],
            right_state: self.layout_states[handle_index + 1],
            left_px: sizes.get(handle_index).copied().unwrap_or(0.0),
            right_px: sizes.get(handle_index + 1).copied().unwrap_or(0.0),
            left_min_override: self.collapsed_min_overrides.get(handle_index).copied().unwrap_or(false),
            right_min_override: self.collapsed_min_overrides.get(handle_index + 1).copied().unwrap_or(false),
            content_axis_px: self.content_axis_size_px(),
        }
    }

    pub(super) fn apply_restore_state(
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

    pub(super) fn scale_restored_pair_to_current_axis(&mut self, restore: &CollapseRestoreState) {
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

    pub(super) fn apply_double_click_collapse(&mut self, index: usize, cx: &mut Context<Self>) -> bool {
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
            if self.model.animated
                && (self.transitions[target_index].is_animating() || self.transitions[target_index].progress() < 1.0)
            {
                self.show_panel(target_index, cx);
                return true;
            }
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
        let animated =
            self.model.animated && behavior.mode == ResizeCollapseMode::Completely && self.model.panels.len() == 2;
        if animated {
            self.transitions[target_index].set_target(0.0);
        }
        self.refresh_panel_sizes_px();
        let sizes_px = self.panel_sizes_px.clone();
        cx.emit(ResizablePanelsEvent::ResizeStart);
        if !animated {
            cx.emit(ResizablePanelsEvent::SizesChanged { sizes_px: sizes_px.clone() });
            cx.emit(ResizablePanelsEvent::ResizeEnd { sizes_px });
        }
        cx.notify();
        true
    }
}
