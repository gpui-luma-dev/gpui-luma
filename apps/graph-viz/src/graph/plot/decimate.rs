/// Min-max binning decimation for line charts.
///
/// Divides the X span into `target_bins` buckets and retains the minimum and
/// maximum Y sample in each bucket to preserve spikes when downsampling.
pub fn minmax_decimate(samples: &[(f32, f32)], target_bins: usize) -> Vec<(f32, f32)> {
    if samples.len() <= 2 || target_bins == 0 {
        return samples.to_vec();
    }

    let max_points = target_bins.saturating_mul(2);
    if samples.len() <= max_points {
        return samples.to_vec();
    }

    let x_min = samples.first().map(|sample| sample.0).unwrap_or(0.0);
    let x_max = samples.last().map(|sample| sample.0).unwrap_or(x_min);
    let x_span = (x_max - x_min).max(f32::EPSILON);
    let mut bins: Vec<Vec<(f32, f32)>> = vec![Vec::new(); target_bins];

    for &(x, y) in samples {
        let fraction = ((x - x_min) / x_span).clamp(0.0, 0.999_999);
        let index = (fraction * target_bins as f32) as usize;
        bins[index.min(target_bins - 1)].push((x, y));
    }

    let mut decimated = Vec::with_capacity(max_points);
    for mut bucket in bins {
        if bucket.is_empty() {
            continue;
        }
        bucket.sort_by(|left, right| left.0.total_cmp(&right.0));
        let min = bucket.iter().min_by(|left, right| left.1.total_cmp(&right.1)).copied().unwrap();
        let max = bucket.iter().max_by(|left, right| left.1.total_cmp(&right.1)).copied().unwrap();
        if min.0 <= max.0 {
            decimated.push(min);
            if max != min {
                decimated.push(max);
            }
        } else {
            decimated.push(max);
            decimated.push(min);
        }
    }

    decimated.sort_by(|left, right| left.0.total_cmp(&right.0));
    decimated
}

/// Uniformly subsample scatter points while preserving temporal order.
pub fn uniform_subsample(samples: &[(f32, f32)], max_points: usize) -> Vec<(f32, f32)> {
    if samples.len() <= max_points || max_points == 0 {
        return samples.to_vec();
    }

    let stride = samples.len() as f32 / max_points as f32;
    let mut subsampled = Vec::with_capacity(max_points);
    let mut cursor = 0.0_f32;
    while subsampled.len() < max_points {
        let index = cursor.floor() as usize;
        if index >= samples.len() {
            break;
        }
        subsampled.push(samples[index]);
        cursor += stride;
    }
    subsampled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_subsample_reduces_large_series() {
        let samples: Vec<(f32, f32)> = (0..1000).map(|index| (index as f32, index as f32)).collect();
        let subsampled = uniform_subsample(&samples, 100);
        assert_eq!(subsampled.len(), 100);
        assert_eq!(subsampled.first(), Some(&(0.0, 0.0)));
    }

    #[test]
    fn minmax_decimate_preserves_spike() {
        let samples: Vec<(f32, f32)> = (0..100)
            .map(|index| {
                let x = index as f32;
                let y = if index == 50 { 100.0 } else { 1.0 };
                (x, y)
            })
            .collect();
        let decimated = minmax_decimate(&samples, 10);
        assert!(decimated.iter().any(|(_, y)| (*y - 100.0).abs() < f32::EPSILON));
    }

    #[test]
    fn minmax_decimate_leaves_small_series_untouched() {
        let samples = vec![(0.0, 1.0), (1.0, 2.0), (2.0, 3.0)];
        assert_eq!(minmax_decimate(&samples, 4), samples);
    }
}
