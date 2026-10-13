use crate::{data::Sample, plots::Decimation};

/// Select from an accepted contiguous source slice before allocating drawing points.
/// Only selected samples reach `emit`; discarded samples avoid Y projection and storage.
pub(super) fn reduce(run: &[Sample], column: impl Fn(f64) -> f32, policy: Decimation, mut emit: impl FnMut(Sample)) {
    if policy == Decimation::None || run.len() <= 4 || run.windows(2).any(|pair| pair[1].x < pair[0].x) {
        for sample in run {
            emit(*sample);
        }
        return;
    }
    let mut start = 0;
    while start < run.len() {
        let bucket = column(run[start].x);
        let mut end = start + 1;
        let mut minimum = start;
        let mut maximum = start;
        while end < run.len() && column(run[end].x) == bucket {
            if run[end].y < run[minimum].y {
                minimum = end;
            }
            if run[end].y > run[maximum].y {
                maximum = end;
            }
            end += 1;
        }
        let mut indices = [start, minimum, maximum, end - 1];
        indices.sort_unstable();
        let mut previous = None;
        for index in indices {
            if previous != Some(index) {
                emit(run[index]);
                previous = Some(index);
            }
        }
        start = end;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn samples(values: &[f64]) -> Vec<Sample> {
        values.iter().enumerate().map(|(index, y)| Sample::new(index as f64 / 10.0, *y)).collect()
    }
    fn selected(run: &[Sample], width: f32, policy: Decimation) -> Vec<Sample> {
        let mut result = Vec::new();
        reduce(run, |x| (x as f32 * width).floor(), policy, |sample| result.push(sample));
        result
    }
    #[test]
    fn retains_endpoints_and_extrema_in_input_order() {
        let run = samples(&[0.0, 1.0, 8.0, 2.0, -7.0, 3.0, 4.0]);
        assert_eq!(selected(&run, 1.0, Decimation::MinMax), [run[0], run[2], run[4], run[6]]);
    }
    #[test]
    fn disabled_short_and_reversing_runs_are_unchanged() {
        let run = samples(&[0.0; 20]);
        assert_eq!(selected(&run, 1.0, Decimation::None), run);
        let short = samples(&[1.0, 2.0, 3.0]);
        assert_eq!(selected(&short, 1.0, Decimation::MinMax), short);
        let mut reversing = run;
        reversing[10].x = 0.0;
        assert_eq!(selected(&reversing, 1.0, Decimation::MinMax), reversing);
    }
    #[test]
    fn only_retained_samples_reach_projection_and_width_changes_resolution() {
        let run = samples(&[0.0; 100]);
        let mut projection_calls = 0;
        reduce(&run, |x| (x as f32).floor(), Decimation::MinMax, |_| projection_calls += 1);
        assert_eq!(projection_calls, 20);
        assert_eq!(selected(&run, 2.0, Decimation::MinMax).len(), 40);
    }
    #[test]
    fn repeated_x_and_constant_columns_preserve_extrema_without_duplicate_indices() {
        let run: Vec<_> = [0.0, -5.0, 2.0, 10.0, 0.0, 1.0].into_iter().map(|y| Sample::new(1.0, y)).collect();
        assert_eq!(selected(&run, 0.0, Decimation::MinMax), [run[0], run[1], run[3], run[5]]);
    }
}
