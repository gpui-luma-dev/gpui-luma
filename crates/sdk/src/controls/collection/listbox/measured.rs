//! Content-sized geometry shared by eager and windowed vertical composition.
//! GPUI measures templates; this index only stores extents and resolves positions.
use std::{collections::HashMap, hash::Hash, ops::Range, sync::Arc};

use super::{ListBoxState, ListBoxVirtualWindow};

/// Prefix sums with logarithmic updates and offset lookup. Includes one gap per
/// entry internally; the final gap is excluded from the collection extent.
#[derive(Default)]
struct HeightIndex {
    values: Vec<f64>,
    sums: Vec<f64>,
}

impl HeightIndex {
    fn new(values: Vec<f64>) -> Self {
        let mut sums = vec![0.0; values.len() + 1];
        for (index, value) in values.iter().enumerate() {
            let i = index + 1;
            sums[i] += value;
            let parent = i + i.isolate_lowest_one();
            if parent < sums.len() {
                sums[parent] += sums[i];
            }
        }
        Self { values, sums }
    }

    fn prefix(&self, mut count: usize) -> f64 {
        let mut sum = 0.0;
        while count > 0 {
            sum += self.sums[count];
            count &= count - 1;
        }
        sum
    }

    fn set(&mut self, index: usize, value: f64) {
        let delta = value - self.values[index];
        self.values[index] = value;
        let mut i = index + 1;
        while i < self.sums.len() {
            self.sums[i] += delta;
            i += i.isolate_lowest_one();
        }
    }

    fn at_offset(&self, offset: f64) -> usize {
        let mut index = 0;
        let mut sum = 0.0;
        let mut step = self.values.len().checked_next_power_of_two().unwrap_or(0);
        while step > 0 {
            let next = index + step;
            if next < self.sums.len() && sum + self.sums[next] <= offset {
                sum += self.sums[next];
                index = next;
            }
            step >>= 1;
        }
        index.min(self.values.len().saturating_sub(1))
    }
}

pub(super) struct MeasuredGeometry<K> {
    identity: Option<Arc<()>>,
    projection_identity: Option<Arc<()>>,
    keys: Vec<K>,
    indices: HashMap<K, usize>,
    heights: HeightIndex,
    valid: Vec<bool>,
    measured: usize,
    gap: f32,
    estimate: f32,
    width: Option<f32>,
    pub overscan: Option<usize>,
    pub correction: f32,
    pub reveal: Option<K>,
    pub reveal_center: bool,
    pub reveal_unfocused: bool,
}

impl<K> Default for MeasuredGeometry<K> {
    fn default() -> Self {
        Self {
            identity: None,
            projection_identity: None,
            keys: Vec::new(),
            indices: HashMap::new(),
            heights: HeightIndex::default(),
            valid: Vec::new(),
            measured: 0,
            gap: 0.0,
            estimate: 48.0,
            width: None,
            overscan: None,
            correction: 0.0,
            reveal: None,
            reveal_center: false,
            reveal_unfocused: false,
        }
    }
}

impl<K: Clone + Eq + Hash> MeasuredGeometry<K> {
    pub fn prepare<T>(
        &mut self,
        state: &ListBoxState<T, K>,
        estimate: f32,
        gap: f32,
        overscan: Option<usize>,
        offset: f32,
    ) -> f32 {
        let estimate = if estimate.is_finite() && estimate > 0.0 {
            estimate.max(1.0)
        } else {
            48.0
        };
        let gap = if gap.is_finite() { gap.max(0.0) } else { 0.0 };
        self.overscan = overscan;
        let identity = &state.snapshot().identity;
        if self.identity.as_ref().is_some_and(|old| Arc::ptr_eq(old, identity))
            && self.projection_identity.as_ref().is_some_and(|old| Arc::ptr_eq(old, state.projection_identity()))
            && self.gap == gap
            && self.estimate == estimate
        {
            return offset;
        }
        let anchor = self.anchor(offset);
        let old: HashMap<_, _> = self.keys.iter().enumerate().map(|(i, key)| (key.clone(), self.height(i))).collect();
        self.keys = state.visible_items().map(|item| item.key).collect();
        self.indices = self.keys.iter().enumerate().map(|(i, key)| (key.clone(), i)).collect();
        self.heights = HeightIndex::new(
            self.keys
                .iter()
                .map(|key| f64::from(old.get(key).copied().unwrap_or(estimate)) + f64::from(gap))
                .collect(),
        );
        // Replacements may change content under the same key. Retain old heights
        // only as estimates; mounted rows will be measured again this frame.
        self.valid = vec![false; self.keys.len()];
        self.measured = 0;
        self.identity = Some(identity.clone());
        self.projection_identity = Some(state.projection_identity().clone());
        self.gap = gap;
        self.estimate = estimate;
        if self.reveal.as_ref().is_some_and(|key| !self.indices.contains_key(key)) {
            self.reveal = None;
        }
        self.restore(anchor).unwrap_or(offset)
    }

    pub fn invalidate(&mut self, key: Option<&K>) {
        if let Some(key) = key {
            if let Some(&i) = self.indices.get(key)
                && self.valid[i]
            {
                self.valid[i] = false;
                self.measured -= 1;
            }
        } else {
            self.valid.fill(false);
            self.measured = 0;
        }
    }

    fn height(&self, index: usize) -> f32 {
        (self.heights.values[index] - f64::from(self.gap)) as f32
    }

    fn start(&self, index: usize) -> f32 {
        self.heights.prefix(index) as f32
    }

    pub fn total(&self) -> f32 {
        (self.heights.prefix(self.keys.len()) - if self.keys.is_empty() { 0.0 } else { f64::from(self.gap) }) as f32
    }

    pub fn clamp(&self, offset: f32, viewport: f32) -> f32 {
        offset.clamp(0.0, (self.total() - viewport).max(0.0))
    }

    fn anchor(&self, offset: f32) -> Option<(K, usize, f32)> {
        let index = self.heights.at_offset(f64::from(offset.max(0.0)));
        self.keys.get(index).map(|key| (key.clone(), index, offset - self.start(index)))
    }

    fn restore(&self, anchor: Option<(K, usize, f32)>) -> Option<f32> {
        let (key, index, within) = anchor?;
        if self.keys.is_empty() {
            return Some(0.0);
        }
        let index = self.indices.get(&key).copied().unwrap_or(index.min(self.keys.len() - 1));
        Some(self.start(index) + within.min(self.height(index) + self.gap))
    }

    pub fn position_for_key(&self, key: &K, offset: f32, viewport: f32, center: bool) -> Option<f32> {
        let index = *self.indices.get(key)?;
        Some(self.clamp(
            super::scroll::position_offset(offset, viewport, self.start(index), self.height(index), center),
            viewport,
        ))
    }

    pub fn reveal_offset(&self, offset: f32, viewport: f32) -> f32 {
        let mut offset = self.clamp(offset, viewport);
        if viewport > 0.0
            && let Some(index) = self.reveal.as_ref().and_then(|key| self.indices.get(key)).copied()
        {
            let top = self.start(index);
            let height = self.height(index);
            offset = super::scroll::position_offset(offset, viewport, top, height, self.reveal_center);
        }
        self.clamp(offset, viewport)
    }

    pub fn window(&self, offset: f32, viewport: f32) -> ListBoxVirtualWindow {
        let count = self.keys.len();
        let offset = self.clamp(offset, viewport);
        let first = self.heights.at_offset(f64::from(offset));
        let visible_start = if count > 0 && offset >= self.start(first) + self.height(first) {
            first + 1
        } else {
            first
        };
        let mut visible_end = visible_start;
        if viewport > 0.0 && count > 0 {
            let last = self.heights.at_offset(f64::from(offset + viewport));
            visible_end = (last + usize::from(self.start(last) < offset + viewport)).min(count).max(visible_start);
        }
        let range = self.overscan.map_or(0..count, |buffer| {
            first.saturating_sub(buffer)..visible_end.max(first.saturating_add(1)).saturating_add(buffer).min(count)
        });
        ListBoxVirtualWindow {
            leading_space: (range.start > 0).then(|| (self.start(range.start) - self.gap).max(0.0)),
            trailing_space: (range.end < count).then(|| (self.total() - self.start(range.end)).max(0.0)),
            range,
            visible_range: visible_start..visible_end,
            measured_items: Some(self.measured),
            metrics: None,
        }
    }

    /// Record one layout batch. Preserve the top key/pixel offset, or finish a
    /// pending keyboard reveal using the target's real height. No input listeners.
    pub fn measure(&mut self, rows: impl Iterator<Item = (K, f32)>, width: f32, offset: f32, viewport: f32) -> bool {
        let anchor = self.anchor(offset);
        let mut changed = false;
        if self.width != Some(width) {
            self.width = Some(width);
            self.invalidate(None);
            changed = true;
        }
        let mut revealed = false;
        for (key, height) in rows {
            if !height.is_finite() || height < 0.0 {
                continue;
            }
            let Some(&i) = self.indices.get(&key) else { continue };
            // A tiny positive extent ensures progress even for empty templates.
            let height = height.max(1.0);
            if !self.valid[i] {
                self.valid[i] = true;
                self.measured += 1;
                changed = true;
            }
            if (height - self.height(i)).abs() > 0.01 {
                self.heights.set(i, f64::from(height) + f64::from(self.gap));
                changed = true;
            }
            revealed |= self.reveal.as_ref() == Some(&key);
        }
        let anchored = self.restore(anchor).unwrap_or(offset);
        let next = self.reveal_offset(anchored, viewport);
        self.correction = next - offset;
        if revealed {
            self.reveal = None;
        }
        changed || (next - offset).abs() > 0.01
    }

    pub fn keys_in(&self, range: Range<usize>) -> Vec<K> {
        self.keys[range].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::listbox::{ListBoxSnapshot, SelectionMode};

    #[test]
    fn prefix_index_updates_and_finds_variable_extents() {
        let mut index = HeightIndex::new(vec![24.0, 44.0, 84.0, 14.0]);
        assert_eq!(index.prefix(3), 152.0);
        for (offset, expected) in [(0.0, 0), (23.0, 0), (24.0, 1), (68.0, 2), (900.0, 3)] {
            assert_eq!(index.at_offset(offset), expected);
        }
        index.set(1, 104.0);
        assert_eq!(index.prefix(3), 212.0);
        assert_eq!(index.at_offset(100.0), 1);
    }

    #[test]
    fn measurement_preserves_anchor_and_replacement_invalidates_by_key() {
        let mut state = ListBoxState::try_new(1..=100_000, |n| *n, SelectionMode::Extended).unwrap();
        let mut geometry = MeasuredGeometry::default();
        geometry.prepare(&state, 20.0, 4.0, Some(2), 0.0);
        // Item 101 stays 7px above the viewport after a preceding row grows.
        let offset = 2_407.0;
        assert!(geometry.measure([(99, 70.0), (101, 40.0)].into_iter(), 250.0, offset, 196.0));
        assert_eq!(geometry.correction, 50.0);
        let window = geometry.window(offset + geometry.correction, 196.0);
        assert_eq!(window.visible_range.start, 100);
        assert!(window.range.len() < 20);
        let gaps = window.range.len()
            + usize::from(window.leading_space.is_some())
            + usize::from(window.trailing_space.is_some())
            - 1;
        let extent = window.range.clone().map(|i| geometry.height(i)).sum::<f32>()
            + window.leading_space.unwrap_or(0.0)
            + window.trailing_space.unwrap_or(0.0)
            + gaps as f32 * 4.0;
        assert_eq!(extent, geometry.total());
        assert_eq!(window.measured_items, Some(2));
        geometry.invalidate(Some(&101));
        assert_eq!(geometry.measured, 1);
        state.replace_snapshot(ListBoxSnapshot::try_new([101, 99], |n| *n).unwrap());
        geometry.prepare(&state, 20.0, 4.0, Some(2), offset + 50.0);
        assert_eq!(geometry.height(0), 40.0);
        assert_eq!(geometry.height(1), 70.0);
        assert_eq!(geometry.measured, 0);
        geometry.measure([(101, 80.0), (99, 50.0)].into_iter(), 250.0, 0.0, 196.0);
        assert_eq!(geometry.total(), 134.0);
        assert_eq!(geometry.window(0.0, 196.0).visible_range, 0..2);
    }

    #[test]
    fn measured_windows_match_actual_intersections_including_gaps_and_empty_lists() {
        for count in [0, 1, 50] {
            for gap in [0.0, 4.0] {
                let state = ListBoxState::try_new(0..count, |n| *n, SelectionMode::Extended).unwrap();
                let mut geometry = MeasuredGeometry::default();
                geometry.prepare(&state, 48.0, gap, Some(usize::MAX), 0.0);
                geometry.measure((0..count).map(|i| (i, (i % 5 + 1) as f32 * 10.0)), 250.0, 0.0, 40.0);
                for offset in (0..2_000).step_by(3) {
                    let offset = geometry.clamp(offset as f32, 40.0);
                    let window = geometry.window(offset, 40.0);
                    let actual: Vec<_> = (0..count)
                        .filter(|&i| {
                            geometry.start(i) < offset + 40.0 && geometry.start(i) + geometry.height(i) > offset
                        })
                        .collect();
                    assert_eq!(window.visible_range.clone().collect::<Vec<_>>(), actual);
                    assert_eq!(window.range, 0..count);
                }
            }
        }
        let state = ListBoxState::try_new(0..20, |n| *n, SelectionMode::Extended).unwrap();
        let mut geometry = MeasuredGeometry::default();
        geometry.prepare(&state, 20.0, 4.0, Some(1), 0.0);
        // Repainting with the viewport edge inside a gap must not move scrolling.
        geometry.measure((0..20).map(|i| (i, 20.0)), 250.0, 22.0, 40.0);
        assert_eq!(geometry.correction, 0.0);
    }
}
