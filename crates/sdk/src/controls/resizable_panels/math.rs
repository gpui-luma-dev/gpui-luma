/// Percent sizes for `n` panels that sum to 100.
pub fn normalize_sizes(sizes: &mut [f32]) {
    if sizes.is_empty() {
        return;
    }

    for size in sizes.iter_mut() {
        if !size.is_finite() || *size < 0.0 {
            *size = 0.0;
        }
    }

    let sum: f32 = sizes.iter().sum();
    if sum <= f32::EPSILON {
        let even = 100.0 / sizes.len() as f32;
        for size in sizes.iter_mut() {
            *size = even;
        }
        return;
    }

    for size in sizes.iter_mut() {
        *size = (*size / sum) * 100.0;
    }
}

pub struct PanelSizeBounds {
    pub min_size: f32,
    pub max_size: f32,
}

/// Adjusts adjacent panels `index` and `index + 1` by `delta_percent`, honoring per-panel bounds.
pub fn apply_pair_delta(
    sizes: &mut [f32],
    index: usize,
    delta_percent: f32,
    left_bounds: PanelSizeBounds,
    right_bounds: PanelSizeBounds,
) -> bool {
    if index + 1 >= sizes.len() {
        return false;
    }

    let left = sizes[index];
    let right = sizes[index + 1];
    let pair_total = left + right;

    let min_left = left_bounds.min_size.max(pair_total - right_bounds.max_size);
    let max_left = left_bounds.max_size.min(pair_total - right_bounds.min_size);
    if min_left > max_left {
        return false;
    }

    let new_left = (left + delta_percent).clamp(min_left, max_left);
    let new_right = pair_total - new_left;

    if (new_left - left).abs() < f32::EPSILON && (new_right - right).abs() < f32::EPSILON {
        return false;
    }

    sizes[index] = new_left;
    sizes[index + 1] = new_right;
    true
}

/// Main-axis pixels available to panels after subtracting handle thickness.
pub fn content_axis_size(main_axis_px: f32, handle_size_px: f32, panel_count: usize) -> f32 {
    let main_px = main_axis_px.max(1.0);
    let handle_total = handle_size_px.max(1.0) * panel_count.saturating_sub(1) as f32;
    (main_px - handle_total).max(1.0)
}

#[cfg(test)]
mod tests {
    use super::{apply_pair_delta, content_axis_size, normalize_sizes, PanelSizeBounds};

    #[test]
    fn normalize_sizes_even_split_when_zero_sum() {
        let mut sizes = [0.0, 0.0];
        normalize_sizes(&mut sizes);
        assert!((sizes[0] - 50.0).abs() < f32::EPSILON);
        assert!((sizes[1] - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn normalize_sizes_scales_to_hundred() {
        let mut sizes = [30.0, 70.0];
        normalize_sizes(&mut sizes);
        assert!((sizes.iter().sum::<f32>() - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn apply_pair_delta_respects_bounds() {
        let mut sizes = [30.0, 70.0];
        let changed = apply_pair_delta(
            &mut sizes,
            0,
            50.0,
            PanelSizeBounds { min_size: 20.0, max_size: 70.0 },
            PanelSizeBounds { min_size: 30.0, max_size: 80.0 },
        );
        assert!(changed);
        assert!((sizes[0] - 70.0).abs() < f32::EPSILON);
        assert!((sizes[1] - 30.0).abs() < f32::EPSILON);
    }

    #[test]
    fn content_axis_size_subtracts_handles() {
        assert_eq!(content_axis_size(540.0, 1.0, 2), 539.0);
        assert_eq!(content_axis_size(540.0, 18.0, 2), 522.0);
    }
}
