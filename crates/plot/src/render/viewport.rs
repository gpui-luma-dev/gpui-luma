use crate::data::Sample;

/// Emit contiguous slices containing visible points and endpoints of crossing segments.
/// Call separately for each accepted run so missing data can never be bridged.
pub(super) fn select(run: &[Sample], min: f64, max: f64, context: usize, mut emit: impl FnMut(&[Sample])) {
    let crosses = |a: f64, b: f64| a.min(b) <= max && a.max(b) >= min;
    let mut start = None;
    for index in 0..run.len() {
        // Cubic derivatives need one more neighbor beyond the crossing endpoints.
        let keep = (index.saturating_sub(context)..(index + 1 + context).min(run.len())).any(|index| {
            let sample = run[index];
            (min..=max).contains(&sample.x)
                || (index > 0 && crosses(run[index - 1].x, sample.x))
                || run.get(index + 1).is_some_and(|next| crosses(sample.x, next.x))
        });
        if keep {
            start.get_or_insert(index);
        } else if let Some(start) = start.take() {
            emit(&run[start..index]);
        }
    }
    if let Some(start) = start {
        emit(&run[start..]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slices(xs: &[f64], min: f64, max: f64) -> Vec<Vec<f64>> {
        let samples: Vec<_> = xs.iter().map(|x| Sample::new(*x, 0.0)).collect();
        let mut result = Vec::new();
        select(&samples, min, max, 0, |run| result.push(run.iter().map(|sample| sample.x).collect()));
        result
    }

    #[test]
    fn keeps_one_neighbor_at_each_edge_and_crossing_without_inside_sample() {
        assert_eq!(slices(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 2.5, 3.5), [vec![2.0, 3.0, 4.0]]);
        assert_eq!(slices(&[0.0, 10.0], 4.0, 6.0), [vec![0.0, 10.0]]);
        assert!(slices(&[0.0, 1.0], 4.0, 6.0).is_empty());
    }

    #[test]
    fn preserves_order_for_reversing_lines_and_splits_offscreen_excursions() {
        assert_eq!(
            slices(&[1.0, 2.0, 10.0, 11.0, 12.0, 2.0, 1.0], 1.0, 2.0),
            [vec![1.0, 2.0, 10.0], vec![12.0, 2.0, 1.0]]
        );
        assert_eq!(slices(&[5.0, 4.0, 3.0, 2.0, 1.0], 2.5, 3.5), [vec![4.0, 3.0, 2.0]]);
    }
}
