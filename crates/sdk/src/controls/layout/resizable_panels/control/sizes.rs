use gpui::{Context, Pixels};

use super::super::math::{apply_pair_delta_px, content_axis_size, solve_layout_px_with_min_overrides};
use super::super::model::{PanelId, ResizablePanelSpec, ResizablePanelsOrientation};
use super::{ResizablePanels, ResizablePanelsEvent};

impl ResizablePanels {
    pub(super) fn axis_position(&self, position: gpui::Point<Pixels>) -> f32 {
        match self.model.orientation {
            ResizablePanelsOrientation::Horizontal => position.x.as_f32(),
            ResizablePanelsOrientation::Vertical => position.y.as_f32(),
        }
    }

    pub(super) fn main_axis_px(&self) -> f32 {
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

    pub(super) fn content_axis_size_px(&self) -> f32 {
        let panel_count = self.layout_states.len();
        content_axis_size(self.main_axis_px(), 0.0, panel_count)
    }

    pub(super) fn drag_content_axis_size_px(&self) -> f32 {
        self.drag_content_axis_px.unwrap_or_else(|| self.content_axis_size_px())
    }

    pub(super) fn apply_pair_delta(&mut self, index: usize, delta_px: f32, cx: &mut Context<Self>) -> bool {
        if index + 1 >= self.layout_states.len() {
            return false;
        }

        self.clear_pair_restore_state(index, cx);
        // Direct dragging takes over from any complete-collapse transition.
        self.transitions[index].snap_to(1.0);
        self.transitions[index + 1].snap_to(1.0);
        let content_main_px = self.drag_content_axis_size_px();
        apply_pair_delta_px(&self.model.panels, &mut self.layout_states, index, delta_px, content_main_px)
    }
    pub(super) fn emit_sizes_changed_if_needed(&mut self, changed: bool, cx: &mut Context<Self>) {
        if !changed {
            return;
        }
        cx.emit(ResizablePanelsEvent::SizesChanged { sizes_px: self.panel_sizes_px.clone() });
        cx.notify();
    }
    pub(super) fn refresh_panel_sizes_px(&mut self) {
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
                let full_w0 = self.panel_hide_restore[0]
                    .as_ref()
                    .or_else(|| self.collapse_restore.iter().flatten().find(|restore| restore.target_index == 0))
                    .map(|restore| restore.left_px)
                    .unwrap_or(raw_sizes[0]);
                let w0 = full_w0 * p0;
                animated_sizes[0] = w0;
                animated_sizes[1] = (total_axis - w0).max(0.0);
            } else if p1 < 1.0 || self.transitions[1].is_animating() {
                let full_w1 = self.panel_hide_restore[1]
                    .as_ref()
                    .or_else(|| self.collapse_restore.iter().flatten().find(|restore| restore.target_index == 1))
                    .map(|restore| restore.right_px)
                    .unwrap_or(raw_sizes[1]);
                let w1 = full_w1 * p1;
                animated_sizes[1] = w1;
                animated_sizes[0] = (total_axis - w1).max(0.0);
            }
        }

        self.panel_sizes_px = animated_sizes;
    }
}

pub(super) fn swap_region_ids(panels: &mut [ResizablePanelSpec], first: &PanelId, second: &PanelId) -> bool {
    let Some(first_index) = panels.iter().position(|panel| panel.panel_id.as_ref() == Some(first)) else {
        return false;
    };
    let Some(second_index) = panels.iter().position(|panel| panel.panel_id.as_ref() == Some(second)) else {
        return false;
    };
    if first_index == second_index {
        return false;
    }
    panels[first_index].panel_id = Some(second.clone());
    panels[second_index].panel_id = Some(first.clone());
    true
}

#[cfg(test)]
mod tests {
    use super::swap_region_ids;
    use crate::controls::resizable_panels::{PanelId, ResizablePanelSpec};

    #[test]
    fn swap_region_ids_preserves_the_physical_panel_slots() {
        let first = PanelId::new("first");
        let second = PanelId::new("second");
        let mut panels =
            vec![ResizablePanelSpec::new_render(|| gpui::div()), ResizablePanelSpec::new_render(|| gpui::div())];
        panels[0].panel_id = Some(first.clone());
        panels[1].panel_id = Some(second.clone());

        assert!(swap_region_ids(&mut panels, &first, &second));
        assert_eq!(panels[0].panel_id, Some(second));
        assert_eq!(panels[1].panel_id, Some(first));
    }
}
