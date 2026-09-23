//! Uniform-item windowing geometry. Selection continues to cover the full snapshot.
use std::ops::Range;

use gpui::{Pixels, ScrollHandle, Size, px};
use super::ListBoxAxis;

/// Rendering policy for fixed-size ListBox items. Eager rendering is the default.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ListBoxVirtualization {
    /// Construct every item on each render.
    #[default]
    Eager,
    /// Construct viewport items plus this many items on each side. Item extent
    /// must be positive and spacing nonnegative; invalid geometry falls back to eager.
    Uniform { overscan: usize },
}

/// One render's collection range and spacer geometry. Keep spacers as direct
/// siblings of the items, with the same inter-child gap as the eager stack.
/// Obtain this from [`super::ListBoxScrollHandle::virtual_window`].
#[derive(Clone, Debug)]
pub struct ListBoxVirtualWindow {
    /// Indices into the full collection, including the overscan buffer.
    pub range: Range<usize>,
    /// Items intersecting the list viewport, including partially visible items,
    /// excluding overscan. Empty before measurement; ignores ancestor clipping.
    pub visible_range: Range<usize>,
    /// Leading spacer extent, excluding the stack's following gap.
    pub leading_space: Option<f32>,
    /// Trailing spacer extent, excluding the stack's preceding gap.
    pub trailing_space: Option<f32>,
    pub(super) metrics: UniformMetrics,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct UniformMetrics {
    pub axis: ListBoxAxis,
    pub count: usize,
    extent: f32,
    gap: f32,
    overscan: usize,
}

impl UniformMetrics {
    pub fn new(axis: ListBoxAxis, count: usize, extent: f32, gap: f32, overscan: usize) -> Option<Self> {
        let metrics = Self { axis, count, extent, gap, overscan };
        (extent.is_finite()
            && extent > 0.0
            && gap.is_finite()
            && gap >= 0.0
            && metrics.total().is_finite()
            && (extent + gap).is_finite())
        .then_some(metrics)
    }

    fn stride(self) -> f32 {
        self.extent + self.gap
    }

    fn total(self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.count as f32 * self.extent + self.count.saturating_sub(1) as f32 * self.gap
        }
    }

    pub fn viewport_extent(self, size: Size<Pixels>) -> f32 {
        f32::from(match self.axis {
            ListBoxAxis::Vertical => size.height,
            ListBoxAxis::Horizontal => size.width,
        })
        .max(0.0)
    }

    pub fn offset(self, scroll: &ScrollHandle) -> f32 {
        let offset = scroll.offset();
        -f32::from(match self.axis {
            ListBoxAxis::Vertical => offset.y,
            ListBoxAxis::Horizontal => offset.x,
        })
    }

    fn clamped(self, offset: f32, viewport: f32) -> f32 {
        offset.clamp(0.0, (self.total() - viewport).max(0.0))
    }

    fn revealed(self, offset: f32, viewport: f32, index: Option<usize>) -> f32 {
        let mut offset = self.clamped(offset, viewport);
        if viewport > 0.0
            && let Some(index) = index.filter(|i| *i < self.count)
        {
            let start = index as f32 * self.stride();
            if start < offset || self.extent > viewport {
                offset = start;
            } else if start + self.extent > offset + viewport {
                offset = start + self.extent - viewport;
            }
        }
        self.clamped(offset, viewport)
    }

    pub fn update_scroll(self, scroll: &ScrollHandle, viewport: f32, reveal: Option<usize>) {
        let value = px(-self.revealed(self.offset(scroll), viewport, reveal));
        let mut offset = scroll.offset();
        match self.axis {
            ListBoxAxis::Vertical => offset.y = value,
            ListBoxAxis::Horizontal => offset.x = value,
        }
        scroll.set_offset(offset);
    }

    pub fn window(self, offset: f32, viewport: f32) -> ListBoxVirtualWindow {
        let offset = self.clamped(offset, viewport);
        let first = ((offset / self.stride()).floor() as usize).min(self.count.saturating_sub(1));
        // Include a boundary row even before the first viewport measurement.
        let end = (((offset + viewport) / self.stride()).ceil() as usize).max(first.saturating_add(1));
        let start = first.saturating_sub(self.overscan).min(self.count);
        let end = end.saturating_add(self.overscan).min(self.count);
        let visible_start = (first + usize::from(offset >= first as f32 * self.stride() + self.extent)).min(self.count);
        let visible_end = if viewport > 0.0 {
            (((offset + viewport) / self.stride()).ceil() as usize).min(self.count).max(visible_start)
        } else {
            visible_start
        };
        ListBoxVirtualWindow {
            range: start..end,
            visible_range: visible_start..visible_end,
            leading_space: (start > 0).then(|| start as f32 * self.stride() - self.gap),
            trailing_space: (end < self.count).then(|| (self.count - end) as f32 * self.stride() - self.gap),
            metrics: self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(count: usize, overscan: usize) -> UniformMetrics {
        UniformMetrics::new(ListBoxAxis::Vertical, count, 36.0, 4.0, overscan).unwrap()
    }

    #[test]
    fn ranges_are_bounded_and_spacers_preserve_exact_extent() {
        for count in [0, 1, 3, 100_000] {
            let metrics = metrics(count, 2);
            for offset in [-50.0, 0.0, 39.0, 20_005.0, 9_000_000.0] {
                let window = metrics.window(offset, 196.0);
                assert!(window.range.len() <= 10);
                assert!(window.range.end <= count);
                assert!(window.visible_range.start >= window.range.start);
                assert!(window.visible_range.end <= window.range.end);
                let children = window.range.len()
                    + usize::from(window.leading_space.is_some())
                    + usize::from(window.trailing_space.is_some());
                let extent = window.range.len() as f32 * 36.0
                    + window.leading_space.unwrap_or(0.0)
                    + window.trailing_space.unwrap_or(0.0)
                    + children.saturating_sub(1) as f32 * 4.0;
                assert_eq!(extent, metrics.total());
            }
        }
    }

    #[test]
    fn visible_ranges_exclude_buffers_and_gaps_but_include_partial_rows() {
        let geometry = metrics(10_000, 2);
        assert_eq!(geometry.window(0.0, 0.0).visible_range, 0..0);
        assert_eq!(geometry.window(0.0, 196.0).visible_range, 0..5);
        assert_eq!(geometry.window(8_000.0, 196.0).visible_range, 200..205);
        assert_eq!(geometry.window(8_010.0, 196.0).visible_range, 200..206);
        assert_eq!(geometry.window(36.0, 4.0).visible_range, 1..1);
        assert_eq!(geometry.window(f32::MAX, 196.0).visible_range, 9_995..10_000);
        assert_eq!(metrics(0, 2).window(0.0, 196.0).visible_range, 0..0);
    }

    #[test]
    fn reveal_and_shrink_clamp_to_the_full_collection() {
        let large = metrics(100_000, 0);
        let end = large.revealed(0.0, 196.0, Some(99_999));
        assert_eq!(end, large.total() - 196.0);
        assert_eq!(large.window(end, 196.0).range, 99_995..100_000);
        assert_eq!(large.revealed(end, 196.0, Some(0)), 0.0);
        assert_eq!(large.revealed(0.0, 20.0, Some(10)), 400.0);
        assert_eq!(metrics(3, 0).revealed(end, 196.0, None), 0.0);
    }

    #[test]
    fn invalid_geometry_falls_back_and_extreme_overscan_does_not_overflow() {
        for extent in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert!(UniformMetrics::new(ListBoxAxis::Vertical, 10, extent, 4.0, 2).is_none());
        }
        for gap in [-1.0, f32::NAN, f32::INFINITY] {
            assert!(UniformMetrics::new(ListBoxAxis::Vertical, 10, 36.0, gap, 2).is_none());
        }
        assert_eq!(metrics(10, usize::MAX).window(200.0, 196.0).range, 0..10);
    }
}
