use super::model::{PanelLayoutState, PanelSize, ResizeHandleMetrics, ResizablePanelSpec};

/// Minimum hit-target width/height for drag interaction (legacy floor; presets define their own).
pub const MIN_HANDLE_LANE_PX: f32 = 12.0;

/// Main-axis visual lane from preset metrics.
pub fn handle_layout_main_axis_px(metrics: &ResizeHandleMetrics) -> f32 {
    metrics.lane_px.max(1.0)
}

/// Hit target width/height from preset metrics.
pub fn handle_hit_target_main_axis_px(metrics: &ResizeHandleMetrics) -> f32 {
    metrics.hit_target_px.max(1.0)
}

/// Back-compat alias for hit target.
pub fn effective_handle_lane_px(metrics: &ResizeHandleMetrics) -> f32 {
    handle_hit_target_main_axis_px(metrics)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanelPxBounds {
    pub min_px: f32,
    pub max_px: f32,
}

pub fn resolve_panel_px_bounds(spec: &ResizablePanelSpec, content_main_px: f32) -> PanelPxBounds {
    resolve_panel_px_bounds_with_min_override(spec, content_main_px, false)
}

fn resolve_panel_px_bounds_with_min_override(
    spec: &ResizablePanelSpec,
    content_main_px: f32,
    allow_zero_min: bool,
) -> PanelPxBounds {
    let content_main_px = content_main_px.max(1.0);

    match spec.size {
        PanelSize::Absolute(_) => {
            let min_px = if allow_zero_min {
                0.0
            } else {
                spec.min_px.unwrap_or(0.0)
            };
            let max_px = spec.max_px.unwrap_or(content_main_px);
            PanelPxBounds { min_px, max_px: max_px.max(min_px) }
        }
        PanelSize::Weight(_) => {
            let min_px = if allow_zero_min {
                0.0
            } else {
                spec.min_px.unwrap_or(0.0)
            };
            let max_px = spec.max_px.unwrap_or(content_main_px);
            PanelPxBounds { min_px, max_px: max_px.max(min_px) }
        }
    }
}

/// Two-pass layout: absolutes first, then distribute remainder by weight.
pub fn solve_layout_px(specs: &[ResizablePanelSpec], states: &[PanelLayoutState], content_main_px: f32) -> Vec<f32> {
    solve_layout_px_with_min_overrides(specs, states, &[], content_main_px)
}

pub fn solve_layout_px_with_min_overrides(
    specs: &[ResizablePanelSpec],
    states: &[PanelLayoutState],
    min_overrides: &[bool],
    content_main_px: f32,
) -> Vec<f32> {
    let panel_count = specs.len();
    if panel_count == 0 {
        return Vec::new();
    }

    let content_main_px = content_main_px.max(1.0);
    let mut sizes_px = vec![0.0; panel_count];
    let mut absolute_total = 0.0f32;
    let mut weight_indices = Vec::new();
    let mut weight_sum = 0.0f32;

    for (index, state) in states.iter().enumerate().take(panel_count) {
        match state {
            PanelLayoutState::Absolute(px) => {
                let bounds = resolve_panel_px_bounds_with_min_override(
                    &specs[index],
                    content_main_px,
                    min_overrides.get(index).copied().unwrap_or(false),
                );
                let clamped = px.clamp(bounds.min_px, bounds.max_px);
                sizes_px[index] = clamped;
                absolute_total += clamped;
            }
            PanelLayoutState::Weight(weight) => {
                let weight = weight.max(0.0);
                weight_indices.push(index);
                weight_sum += weight;
            }
        }
    }

    if absolute_total > content_main_px && absolute_total > f32::EPSILON {
        let scale = content_main_px / absolute_total;
        absolute_total = content_main_px;
        for (index, state) in states.iter().enumerate().take(panel_count) {
            if matches!(state, PanelLayoutState::Absolute(_)) {
                sizes_px[index] *= scale;
            }
        }
    }

    let remainder = (content_main_px - absolute_total).max(0.0);

    if weight_indices.is_empty() {
        return sizes_px;
    }

    if weight_sum <= f32::EPSILON {
        let even = remainder / weight_indices.len() as f32;
        for index in &weight_indices {
            sizes_px[*index] = even;
        }
    } else {
        for index in &weight_indices {
            let PanelLayoutState::Weight(weight) = states[*index] else {
                continue;
            };
            sizes_px[*index] = remainder * (weight / weight_sum);
        }
    }

    clamp_weight_panels_px(&mut sizes_px, specs, states, min_overrides, &weight_indices, content_main_px, remainder);
    sizes_px
}

fn clamp_weight_panels_px(
    sizes_px: &mut [f32],
    specs: &[ResizablePanelSpec],
    states: &[PanelLayoutState],
    min_overrides: &[bool],
    weight_indices: &[usize],
    content_main_px: f32,
    remainder: f32,
) {
    if weight_indices.is_empty() || remainder <= f32::EPSILON {
        return;
    }

    for _ in 0..weight_indices.len() {
        let mut clamped_any = false;
        let mut fixed_total = 0.0f32;
        let mut flexible = Vec::new();
        let mut flexible_weight_sum = 0.0f32;

        for &index in weight_indices {
            let PanelLayoutState::Weight(weight) = states[index] else {
                continue;
            };
            let bounds = resolve_panel_px_bounds_with_min_override(
                &specs[index],
                content_main_px,
                min_overrides.get(index).copied().unwrap_or(false),
            );
            let size = sizes_px[index];
            if size < bounds.min_px - f32::EPSILON {
                sizes_px[index] = bounds.min_px;
                fixed_total += bounds.min_px;
                clamped_any = true;
            } else if size > bounds.max_px + f32::EPSILON {
                sizes_px[index] = bounds.max_px;
                fixed_total += bounds.max_px;
                clamped_any = true;
            } else {
                flexible.push(index);
                flexible_weight_sum += weight.max(0.0);
            }
        }

        if !clamped_any {
            break;
        }

        let flex_remainder = (remainder - fixed_total).max(0.0);
        if flexible.is_empty() {
            break;
        }

        if flexible_weight_sum <= f32::EPSILON {
            let even = flex_remainder / flexible.len() as f32;
            for index in flexible {
                sizes_px[index] = even;
            }
        } else {
            for index in flexible {
                let PanelLayoutState::Weight(weight) = states[index] else {
                    continue;
                };
                sizes_px[index] = flex_remainder * (weight.max(0.0) / flexible_weight_sum);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn apply_pair_collapse_px(
    specs: &[ResizablePanelSpec],
    states: &mut [PanelLayoutState],
    min_overrides: &mut [bool],
    handle_index: usize,
    target_index: usize,
    target_px: f32,
    ignore_target_min: bool,
    content_main_px: f32,
) -> bool {
    if handle_index + 1 >= states.len() || target_index >= states.len() {
        return false;
    }
    if target_index != handle_index && target_index != handle_index + 1 {
        return false;
    }

    let neighbor_index = if target_index == handle_index {
        handle_index + 1
    } else {
        handle_index
    };
    let start_px = solve_layout_px_with_min_overrides(specs, states, min_overrides, content_main_px);
    let pair_total = start_px[handle_index] + start_px[handle_index + 1];

    let target_bounds =
        resolve_panel_px_bounds_with_min_override(&specs[target_index], content_main_px, ignore_target_min);
    let neighbor_bounds = resolve_panel_px_bounds_with_min_override(
        &specs[neighbor_index],
        content_main_px,
        min_overrides.get(neighbor_index).copied().unwrap_or(false),
    );

    let min_target = target_bounds.min_px.max(pair_total - neighbor_bounds.max_px);
    let max_target = target_bounds.max_px.min(pair_total - neighbor_bounds.min_px);
    if min_target > max_target {
        return false;
    }

    let new_target = target_px.clamp(min_target, max_target);
    let new_neighbor = pair_total - new_target;
    if (new_target - start_px[target_index]).abs() < f32::EPSILON
        && (new_neighbor - start_px[neighbor_index]).abs() < f32::EPSILON
    {
        return false;
    }

    let neighbor_was_weight = matches!(states[neighbor_index], PanelLayoutState::Weight(_));
    states[target_index] = PanelLayoutState::Absolute(new_target);
    if !(ignore_target_min && new_target <= f32::EPSILON && neighbor_was_weight) {
        states[neighbor_index] = PanelLayoutState::Absolute(new_neighbor);
    }

    if let Some(override_min) = min_overrides.get_mut(target_index) {
        *override_min = ignore_target_min && new_target < specs[target_index].min_px.unwrap_or(0.0);
    }
    if let Some(override_min) = min_overrides.get_mut(neighbor_index) {
        *override_min = false;
    }

    true
}

/// Adjusts adjacent panels at `index` / `index + 1` by `delta_px` on the main axis.
pub fn apply_pair_delta_px(
    specs: &[ResizablePanelSpec],
    states: &mut [PanelLayoutState],
    index: usize,
    delta_px: f32,
    content_main_px: f32,
) -> bool {
    if index + 1 >= states.len() {
        return false;
    }

    let start_states = states.to_vec();
    let start_px = solve_layout_px(specs, &start_states, content_main_px);
    let left_px = start_px[index];
    let right_px = start_px[index + 1];
    let pair_total = left_px + right_px;

    let left_bounds = resolve_panel_px_bounds(&specs[index], content_main_px);
    let right_bounds = resolve_panel_px_bounds(&specs[index + 1], content_main_px);

    match (states[index], states[index + 1]) {
        (PanelLayoutState::Absolute(left), PanelLayoutState::Weight(_)) => {
            let min_left = left_bounds.min_px.max(pair_total - right_bounds.max_px);
            let max_left = left_bounds.max_px.min(pair_total - right_bounds.min_px);
            if min_left > max_left {
                return false;
            }
            let new_left = (left + delta_px).clamp(min_left, max_left);
            if (new_left - left).abs() < f32::EPSILON {
                return false;
            }
            states[index] = PanelLayoutState::Absolute(new_left);
            true
        }
        (PanelLayoutState::Weight(_), PanelLayoutState::Absolute(right)) => {
            let min_right = right_bounds.min_px.max(pair_total - left_bounds.max_px);
            let max_right = right_bounds.max_px.min(pair_total - left_bounds.min_px);
            if min_right > max_right {
                return false;
            }
            let new_right = (right - delta_px).clamp(min_right, max_right);
            if (new_right - right).abs() < f32::EPSILON {
                return false;
            }
            states[index + 1] = PanelLayoutState::Absolute(new_right);
            true
        }
        (PanelLayoutState::Weight(left_weight), PanelLayoutState::Weight(right_weight)) => {
            let min_left = left_bounds.min_px.max(pair_total - right_bounds.max_px);
            let max_left = left_bounds.max_px.min(pair_total - right_bounds.min_px);
            if min_left > max_left {
                return false;
            }
            let new_left_px = (left_px + delta_px).clamp(min_left, max_left);
            let weight_sum = (left_weight + right_weight).max(f32::EPSILON);
            let left_ratio = new_left_px / pair_total.max(f32::EPSILON);
            let new_left_weight = weight_sum * left_ratio;
            let new_right_weight = weight_sum - new_left_weight;
            if (new_left_weight - left_weight).abs() < f32::EPSILON
                && (new_right_weight - right_weight).abs() < f32::EPSILON
            {
                return false;
            }
            states[index] = PanelLayoutState::Weight(new_left_weight.max(0.0));
            states[index + 1] = PanelLayoutState::Weight(new_right_weight.max(0.0));
            true
        }
        (PanelLayoutState::Absolute(left), PanelLayoutState::Absolute(right)) => {
            let min_left = left_bounds.min_px.max(pair_total - right_bounds.max_px);
            let max_left = left_bounds.max_px.min(pair_total - right_bounds.min_px);
            if min_left > max_left {
                return false;
            }
            let new_left = (left + delta_px).clamp(min_left, max_left);
            let new_right = pair_total - new_left;
            if (new_left - left).abs() < f32::EPSILON && (new_right - right).abs() < f32::EPSILON {
                return false;
            }
            states[index] = PanelLayoutState::Absolute(new_left);
            states[index + 1] = PanelLayoutState::Absolute(new_right);
            true
        }
    }
}

/// Main-axis pixels available to panels. Overlay handles do not consume layout space.
pub fn content_axis_size(main_axis_px: f32, _handle_size_px: f32, _panel_count: usize) -> f32 {
    main_axis_px.max(1.0)
}

/// Overlay handle geometry centered on `split_px` (pick'em snapping for even widths).
pub fn handle_overlay_geometry(split_px: f32, handle_width_px: f32) -> (f32, f32, f32) {
    let width = handle_width_px.max(1.0).round();
    let width_i = width as i32;
    let half = width_i / 2;
    let origin = split_px - half as f32;
    let divider_local = half as f32;
    (origin, width, divider_local)
}

pub fn split_positions_px(panel_sizes_px: &[f32]) -> Vec<f32> {
    let mut positions = Vec::with_capacity(panel_sizes_px.len().saturating_sub(1));
    let mut cumulative = 0.0f32;
    for (index, size) in panel_sizes_px.iter().enumerate() {
        cumulative += *size;
        if index + 1 < panel_sizes_px.len() {
            positions.push(cumulative);
        }
    }
    positions
}

#[cfg(test)]
mod tests {
    use super::{
        PanelLayoutState, apply_pair_collapse_px, apply_pair_delta_px, content_axis_size, handle_overlay_geometry,
        resolve_panel_px_bounds, solve_layout_px, solve_layout_px_with_min_overrides,
    };
    use crate::controls::resizable_panels::model::ResizablePanelSpec;

    fn weight_spec(weight: f32) -> ResizablePanelSpec {
        ResizablePanelSpec::new_render(|| gpui::Empty).weight(weight)
    }

    fn absolute_spec(width_px: f32) -> ResizablePanelSpec {
        ResizablePanelSpec::new_render(|| gpui::Empty).size(gpui::px(width_px))
    }

    fn min_weight_spec(weight: f32, min_px: f32) -> ResizablePanelSpec {
        ResizablePanelSpec::new_render(|| gpui::Empty).weight(weight).min(gpui::px(min_px))
    }

    #[test]
    fn solve_mixed_absolute_and_weight() {
        let specs = [absolute_spec(280.0), weight_spec(1.0)];
        let states = [PanelLayoutState::Absolute(280.0), PanelLayoutState::Weight(1.0)];
        let sizes = solve_layout_px(&specs, &states, 1000.0);
        assert!((sizes[0] - 280.0).abs() < f32::EPSILON);
        assert!((sizes[1] - 720.0).abs() < f32::EPSILON);
    }

    #[test]
    fn solve_scales_absolute_panels_on_overflow() {
        let specs = [absolute_spec(400.0), absolute_spec(400.0)];
        let states = [PanelLayoutState::Absolute(400.0), PanelLayoutState::Absolute(400.0)];
        let sizes = solve_layout_px(&specs, &states, 500.0);
        assert!((sizes[0] - 250.0).abs() < f32::EPSILON);
        assert!((sizes[1] - 250.0).abs() < f32::EPSILON);
    }

    #[test]
    fn apply_pair_delta_absolute_weight() {
        let specs = [absolute_spec(200.0), weight_spec(1.0)];
        let mut states = [PanelLayoutState::Absolute(200.0), PanelLayoutState::Weight(1.0)];
        assert!(apply_pair_delta_px(&specs, &mut states, 0, 40.0, 1000.0));
        let PanelLayoutState::Absolute(left) = states[0] else {
            panic!("expected absolute state");
        };
        assert!((left - 240.0).abs() < f32::EPSILON);
    }

    #[test]
    fn collapse_pair_to_min_size_respects_target_min() {
        let specs = [min_weight_spec(1.0, 120.0), weight_spec(1.0)];
        let mut states = [PanelLayoutState::Weight(1.0), PanelLayoutState::Weight(1.0)];
        let mut overrides = [false, false];

        let changed = apply_pair_collapse_px(&specs, &mut states, &mut overrides, 0, 0, 120.0, false, 500.0);

        assert!(changed);
        assert_eq!(overrides, [false, false]);
        let sizes = solve_layout_px_with_min_overrides(&specs, &states, &overrides, 500.0);
        assert!((sizes[0] - 120.0).abs() < f32::EPSILON);
        assert!((sizes[1] - 380.0).abs() < f32::EPSILON);
    }

    #[test]
    fn collapse_pair_completely_can_ignore_target_min() {
        let specs = [min_weight_spec(1.0, 120.0), weight_spec(1.0)];
        let mut states = [PanelLayoutState::Weight(1.0), PanelLayoutState::Weight(1.0)];
        let mut overrides = [false, false];

        let changed = apply_pair_collapse_px(&specs, &mut states, &mut overrides, 0, 0, 0.0, true, 500.0);

        assert!(changed);
        assert_eq!(overrides, [true, false]);
        let sizes = solve_layout_px_with_min_overrides(&specs, &states, &overrides, 500.0);
        assert!((sizes[0] - 0.0).abs() < f32::EPSILON);
        assert!((sizes[1] - 500.0).abs() < f32::EPSILON);
    }

    #[test]
    fn complete_collapse_preserves_weight_neighbor_across_resize() {
        let specs = [
            absolute_spec(360.0).min(gpui::px(240.0)),
            weight_spec(1.0),
            absolute_spec(360.0).min(gpui::px(240.0)),
        ];
        let mut states =
            [PanelLayoutState::Absolute(360.0), PanelLayoutState::Weight(1.0), PanelLayoutState::Absolute(360.0)];
        let mut overrides = [false, false, false];

        let changed = apply_pair_collapse_px(&specs, &mut states, &mut overrides, 1, 2, 0.0, true, 1000.0);

        assert!(changed);
        assert_eq!(overrides, [false, false, true]);
        assert!(matches!(states[1], PanelLayoutState::Weight(_)));
        let sizes = solve_layout_px_with_min_overrides(&specs, &states, &overrides, 1400.0);
        assert!((sizes[0] - 360.0).abs() < f32::EPSILON);
        assert!((sizes[1] - 1040.0).abs() < f32::EPSILON);
        assert!((sizes[2] - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn content_axis_size_ignores_overlay_handles() {
        assert_eq!(content_axis_size(540.0, 1.0, 2), 540.0);
        assert_eq!(content_axis_size(540.0, 18.0, 2), 540.0);
    }

    #[test]
    fn handle_overlay_geometry_even_width() {
        let (origin, width, divider_local) = handle_overlay_geometry(280.0, 8.0);
        assert!((origin - 276.0).abs() < f32::EPSILON);
        assert!((width - 8.0).abs() < f32::EPSILON);
        assert!((divider_local - 4.0).abs() < f32::EPSILON);
        assert!((origin + divider_local - 280.0).abs() < f32::EPSILON);
    }

    #[test]
    fn handle_overlay_geometry_odd_width() {
        let (origin, width, divider_local) = handle_overlay_geometry(280.0, 5.0);
        assert!((origin - 278.0).abs() < f32::EPSILON);
        assert!((width - 5.0).abs() < f32::EPSILON);
        assert!((divider_local - 2.0).abs() < f32::EPSILON);
        assert!((origin + divider_local - 280.0).abs() < f32::EPSILON);
    }

    #[test]
    fn resolve_weight_pixel_bounds_from_min_max_px() {
        let spec = weight_spec(1.0).min(gpui::px(200.0)).max(gpui::px(800.0));
        let bounds = resolve_panel_px_bounds(&spec, 1000.0);
        assert!((bounds.min_px - 200.0).abs() < f32::EPSILON);
        assert!((bounds.max_px - 800.0).abs() < f32::EPSILON);
    }
}
