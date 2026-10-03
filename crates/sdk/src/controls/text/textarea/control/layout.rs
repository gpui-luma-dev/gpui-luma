use std::{ops::Range, sync::Arc};

use gpui::{Bounds, Font, Hsla, Pixels, Point, SharedString, ShapedLine, point, px};

use super::TextArea;

#[derive(Clone)]
pub(super) struct TextAreaCachedLine {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) text: String,
    pub(super) line: ShapedLine,
}

// Only inputs affecting wrapping or shaped runs belong here; viewport geometry is refreshed each frame.
#[derive(Clone, PartialEq)]
pub(super) struct TextAreaLayoutKey {
    pub(super) text: SharedString,
    pub(super) wrap_width: Pixels,
    pub(super) font: Font,
    pub(super) font_size: Pixels,
    pub(super) line_height: Pixels,
    pub(super) foreground: Hsla,
    pub(super) scale_factor_bits: u32,
}

// Decoration is cached independently from wrapping and hit-test geometry.
#[derive(Clone, PartialEq)]
pub(super) struct TextAreaDecorationKey {
    pub(super) line_index: usize,
    pub(super) selection: Option<(usize, usize)>,
    pub(super) marked_range: Option<Range<usize>>,
    pub(super) selection_foreground: Hsla,
}

#[derive(Clone)]
pub(super) struct TextAreaPaintLine {
    pub(super) key: TextAreaDecorationKey,
    pub(super) line: ShapedLine,
}

#[derive(Clone)]
pub(super) struct TextAreaLayoutCache {
    pub(super) text_viewport: Bounds<Pixels>,
    pub(super) line_height: Pixels,
    pub(super) lines: Arc<[TextAreaCachedLine]>,
    pub(super) key: Option<TextAreaLayoutKey>,
    pub(super) paint_lines: Vec<TextAreaPaintLine>,
}

impl TextArea {
    pub(super) fn logical_lines(value: &str) -> Vec<(usize, usize, String)> {
        let mut lines = Vec::new();
        let mut start = 0usize;
        let mut current = String::new();

        for (ix, ch) in value.chars().enumerate() {
            if ch == '\n' {
                lines.push((start, ix, std::mem::take(&mut current)));
                start = ix + 1;
            } else {
                current.push(ch);
            }
        }

        lines.push((start, value.chars().count(), current));
        lines
    }

    pub(super) fn max_vertical_scroll_for_cache(cache: &TextAreaLayoutCache) -> Pixels {
        (cache.line_height * cache.lines.len() as f32 - cache.text_viewport.size.height).max(px(0.0))
    }

    pub(super) fn clamped_vertical_scroll(cache: &TextAreaLayoutCache, scroll: Pixels) -> Pixels {
        scroll.max(px(0.0)).min(Self::max_vertical_scroll_for_cache(cache))
    }

    pub(super) fn clamp_vertical_scroll_to_cache(&mut self) -> bool {
        let Some(cache) = self.layout_cache.as_ref() else {
            return false;
        };

        let clamped = Self::clamped_vertical_scroll(cache, self.vertical_scroll);
        if clamped == self.vertical_scroll {
            return false;
        }

        self.vertical_scroll = clamped;
        true
    }

    pub(super) fn scroll_by(&mut self, delta: Pixels) -> bool {
        let Some(cache) = self.layout_cache.as_ref() else {
            return false;
        };

        let next = Self::clamped_vertical_scroll(cache, self.vertical_scroll + delta);
        if next == self.vertical_scroll {
            return false;
        }

        self.vertical_scroll = next;
        true
    }

    pub(super) fn selection_autoscroll_delta_for_point(&self, position: Point<Pixels>) -> Pixels {
        let Some(cache) = self.layout_cache.as_ref() else {
            return px(0.0);
        };

        Self::selection_autoscroll_delta_for_cache(cache, position)
    }

    pub(super) fn selection_autoscroll_delta_for_cache(cache: &TextAreaLayoutCache, position: Point<Pixels>) -> Pixels {
        if Self::max_vertical_scroll_for_cache(cache) <= px(0.5) {
            return px(0.0);
        }

        let top = cache.text_viewport.top();
        let bottom = cache.text_viewport.bottom();
        let overflow = if position.y < top {
            position.y - top
        } else if position.y > bottom {
            position.y - bottom
        } else {
            px(0.0)
        };

        if overflow == px(0.0) {
            return px(0.0);
        }

        let direction = overflow.as_f32().signum();
        let distance = overflow.as_f32().abs();
        let line_height = cache.line_height.as_f32().max(1.0);
        let lines_per_tick = (distance / line_height).clamp(1.0, 6.0);
        px(direction * line_height * lines_per_tick)
    }

    pub(super) fn selection_hit_position(&self, position: Point<Pixels>) -> Point<Pixels> {
        let Some(cache) = self.layout_cache.as_ref() else {
            return position;
        };

        Self::selection_hit_position_for_cache(cache, position)
    }

    pub(super) fn selection_hit_position_for_cache(
        cache: &TextAreaLayoutCache,
        position: Point<Pixels>,
    ) -> Point<Pixels> {
        let top = cache.text_viewport.top();
        let bottom = cache.text_viewport.bottom();
        let left = cache.text_viewport.left();
        let right = cache.text_viewport.right() - px(0.5);

        if position.y < top {
            return point(left, top);
        }

        if position.y > bottom {
            return point(right, bottom - px(0.5));
        }

        let x = position.x.max(left).min(right);
        point(x, position.y)
    }

    pub(super) fn vertical_scroll_to_reveal_cursor(
        cache: &TextAreaLayoutCache,
        cursor: usize,
        scroll: Pixels,
    ) -> Pixels {
        let current = Self::clamped_vertical_scroll(cache, scroll);
        let Some((line_ix, _)) = cache
            .lines
            .iter()
            .enumerate()
            .find(|(_, line)| cursor >= line.start && cursor <= line.end)
            .or_else(|| cache.lines.iter().enumerate().next_back())
        else {
            return current;
        };

        let line_top = cache.line_height * line_ix as f32;
        let line_bottom = line_top + cache.line_height;
        let viewport_bottom = current + cache.text_viewport.size.height;

        if line_top < current {
            return Self::clamped_vertical_scroll(cache, line_top);
        }

        if line_bottom > viewport_bottom {
            return Self::clamped_vertical_scroll(cache, line_bottom - cache.text_viewport.size.height);
        }

        current
    }

    pub(super) fn ensure_cursor_visible(&mut self) -> bool {
        let Some(cache) = self.layout_cache.as_ref() else {
            return false;
        };

        let next = Self::vertical_scroll_to_reveal_cursor(cache, self.state.cursor, self.vertical_scroll);
        if next == self.vertical_scroll {
            return false;
        }

        self.vertical_scroll = next;
        true
    }

    pub(super) fn char_offset_for_point(&self, position: Point<Pixels>) -> usize {
        let Some(cache) = self.layout_cache.as_ref() else {
            return self.model.value.chars().count();
        };

        let local_y = Self::local_axis_position(position.y, cache.text_viewport.top());
        if local_y < px(0.0) {
            return cache.lines.first().map(|line| line.start).unwrap_or(0);
        }
        if local_y > cache.text_viewport.size.height {
            return cache.lines.last().map(|line| line.end).unwrap_or(0);
        }

        let line_ix = Self::line_index_for_local_y(local_y, self.vertical_scroll, cache.line_height, cache.lines.len());
        let Some(line) = cache.lines.get(line_ix) else {
            return 0;
        };
        let local_x = Self::local_axis_position(position.x, cache.text_viewport.left());
        if local_x < px(0.0) {
            return line.start;
        }
        if local_x > cache.text_viewport.size.width {
            return line.end;
        }

        let byte_index = line.line.closest_index_for_x(local_x);
        line.start + Self::byte_to_char_offset_for_line(self.model.value.as_ref(), line.start, line.end, byte_index)
    }

    pub(super) fn char_to_byte_offset(text: &str, char_offset: usize) -> usize {
        text.chars().take(char_offset).map(char::len_utf8).sum()
    }

    /// UTF-8 boundaries indexed by character offset, including the end of the line.
    pub(super) fn char_byte_offsets(text: &str) -> Vec<usize> {
        text.char_indices().map(|(offset, _)| offset).chain(std::iter::once(text.len())).collect()
    }

    pub(super) fn byte_to_char_offset(text: &str, byte_offset: usize) -> usize {
        let mut char_offset = 0usize;
        let mut consumed = 0usize;
        for ch in text.chars() {
            if consumed >= byte_offset {
                break;
            }
            consumed += ch.len_utf8();
            char_offset += 1;
        }
        char_offset
    }

    pub(super) fn byte_to_char_offset_for_line(text: &str, start: usize, end: usize, byte_offset: usize) -> usize {
        let line = text.chars().skip(start).take(end.saturating_sub(start)).collect::<String>();
        Self::byte_to_char_offset(&line, byte_offset)
    }

    pub(super) fn local_axis_position(position: Pixels, viewport_start: Pixels) -> Pixels {
        position - viewport_start
    }

    pub(super) fn line_index_for_local_y(
        local_y: Pixels,
        vertical_scroll: Pixels,
        line_height: Pixels,
        line_count: usize,
    ) -> usize {
        if line_count == 0 || line_height <= px(0.0) {
            return 0;
        }

        let scrolled_y = (local_y + vertical_scroll).max(px(0.0));
        let line_ix = (scrolled_y.as_f32() / line_height.as_f32()).floor() as usize;
        line_ix.min(line_count.saturating_sub(1))
    }

    pub(super) fn range_bounds_from_cache(
        cache: &TextAreaLayoutCache,
        text: &str,
        vertical_scroll: Pixels,
        range: Range<usize>,
    ) -> Option<Bounds<Pixels>> {
        let start = range.start.min(range.end);
        let end = range.end.max(range.start);
        let (line_ix, line) = cache
            .lines
            .iter()
            .enumerate()
            .find(|(_, line)| start >= line.start && start <= line.end)
            .or_else(|| cache.lines.iter().enumerate().next())?;
        let line_text = text.chars().skip(line.start).take(line.end.saturating_sub(line.start)).collect::<String>();
        let local_start = start.saturating_sub(line.start);
        let local_end = end.min(line.end).saturating_sub(line.start).max(local_start);
        let start_x = line.line.x_for_index(Self::char_to_byte_offset(&line_text, local_start));
        let end_x = line.line.x_for_index(Self::char_to_byte_offset(&line_text, local_end));
        let top = cache.text_viewport.top() + cache.line_height * line_ix as f32 - vertical_scroll;

        Some(Bounds::from_corners(
            point(cache.text_viewport.left() + start_x, top),
            point(cache.text_viewport.left() + end_x.max(start_x), top + cache.line_height),
        ))
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Bounds, ShapedLine, point, px, size};

    use super::super::TextArea;
    use super::{TextAreaCachedLine, TextAreaLayoutCache};

    #[test]
    fn char_byte_offsets_preserve_unicode_and_empty_line_boundaries() {
        for text in ["", "ascii text", "é中🙂", "e\u{301} 👩\u{200d}💻"] {
            let offsets = TextArea::char_byte_offsets(text);
            assert_eq!(offsets.len(), text.chars().count() + 1);
            for (char_offset, &byte_offset) in offsets.iter().enumerate() {
                assert_eq!(byte_offset, TextArea::char_to_byte_offset(text, char_offset));
                assert!(text.is_char_boundary(byte_offset));
            }
            let reconstructed: String = offsets.windows(2).map(|pair| &text[pair[0]..pair[1]]).collect();
            assert_eq!(reconstructed, text);
        }
    }

    fn cache_with_lines(lines: Vec<(usize, usize)>) -> TextAreaLayoutCache {
        TextAreaLayoutCache {
            key: None,
            paint_lines: Vec::new(),
            text_viewport: Bounds::new(point(px(0.0), px(0.0)), size(px(100.0), px(60.0))),
            line_height: px(20.0),
            lines: lines
                .into_iter()
                .map(|(start, end)| TextAreaCachedLine { start, end, text: String::new(), line: ShapedLine::default() })
                .collect(),
        }
    }

    #[test]
    fn local_axis_position_uses_window_coordinates() {
        assert_eq!(TextArea::local_axis_position(px(110.0), px(100.0)), px(10.0));
        assert_eq!(TextArea::local_axis_position(px(180.0), px(100.0)), px(80.0));
    }

    #[test]
    fn local_axis_position_preserves_above_viewport_points() {
        assert_eq!(TextArea::local_axis_position(px(90.0), px(100.0)), px(-10.0));
    }

    #[test]
    fn line_index_for_local_y_maps_each_visible_line() {
        assert_eq!(TextArea::line_index_for_local_y(px(0.0), px(0.0), px(20.0), 4), 0);
        assert_eq!(TextArea::line_index_for_local_y(px(19.0), px(0.0), px(20.0), 4), 0);
        assert_eq!(TextArea::line_index_for_local_y(px(20.0), px(0.0), px(20.0), 4), 1);
        assert_eq!(TextArea::line_index_for_local_y(px(40.0), px(0.0), px(20.0), 4), 2);
    }

    #[test]
    fn line_index_for_local_y_accounts_for_scroll_and_clamps_to_last_line() {
        assert_eq!(TextArea::line_index_for_local_y(px(0.0), px(20.0), px(20.0), 4), 1);
        assert_eq!(TextArea::line_index_for_local_y(px(500.0), px(0.0), px(20.0), 4), 3);
        assert_eq!(TextArea::line_index_for_local_y(px(-50.0), px(0.0), px(20.0), 4), 0);
    }

    #[test]
    fn line_index_for_local_y_keeps_top_edge_as_first_line() {
        assert_eq!(TextArea::line_index_for_local_y(px(0.0), px(0.0), px(20.0), 4), 0);
    }

    #[test]
    fn max_vertical_scroll_uses_content_minus_viewport_height() {
        let cache = cache_with_lines(vec![(0, 4), (5, 9), (10, 14), (15, 19), (20, 24)]);

        assert_eq!(TextArea::max_vertical_scroll_for_cache(&cache), px(40.0));
    }

    #[test]
    fn clamped_vertical_scroll_stays_inside_scrollable_range() {
        let cache = cache_with_lines(vec![(0, 4), (5, 9), (10, 14), (15, 19), (20, 24)]);

        assert_eq!(TextArea::clamped_vertical_scroll(&cache, px(-10.0)), px(0.0));
        assert_eq!(TextArea::clamped_vertical_scroll(&cache, px(30.0)), px(30.0));
        assert_eq!(TextArea::clamped_vertical_scroll(&cache, px(100.0)), px(40.0));
    }

    #[test]
    fn vertical_scroll_to_reveal_cursor_scrolls_down_to_caret_line() {
        let cache = cache_with_lines(vec![(0, 4), (5, 9), (10, 14), (15, 19), (20, 24)]);

        assert_eq!(TextArea::vertical_scroll_to_reveal_cursor(&cache, 21, px(0.0)), px(40.0));
    }

    #[test]
    fn vertical_scroll_to_reveal_cursor_scrolls_up_to_caret_line() {
        let cache = cache_with_lines(vec![(0, 4), (5, 9), (10, 14), (15, 19), (20, 24)]);

        assert_eq!(TextArea::vertical_scroll_to_reveal_cursor(&cache, 2, px(40.0)), px(0.0));
    }

    #[test]
    fn selection_autoscroll_delta_is_zero_inside_viewport() {
        let cache = cache_with_lines(vec![(0, 4), (5, 9), (10, 14), (15, 19), (20, 24)]);

        assert_eq!(TextArea::selection_autoscroll_delta_for_cache(&cache, point(px(10.0), px(30.0))), px(0.0));
    }

    #[test]
    fn selection_autoscroll_delta_follows_pointer_overflow_direction() {
        let cache = cache_with_lines(vec![(0, 4), (5, 9), (10, 14), (15, 19), (20, 24)]);

        assert_eq!(TextArea::selection_autoscroll_delta_for_cache(&cache, point(px(10.0), px(-10.0))), px(-20.0));
        assert_eq!(TextArea::selection_autoscroll_delta_for_cache(&cache, point(px(10.0), px(70.0))), px(20.0));
    }

    #[test]
    fn selection_autoscroll_delta_accelerates_and_caps_by_distance() {
        let cache = cache_with_lines(vec![
            (0, 4),
            (5, 9),
            (10, 14),
            (15, 19),
            (20, 24),
            (25, 29),
            (30, 34),
            (35, 39),
            (40, 44),
        ]);

        assert_eq!(TextArea::selection_autoscroll_delta_for_cache(&cache, point(px(10.0), px(260.0))), px(120.0));
    }

    #[test]
    fn selection_hit_position_clamps_to_viewport_edges() {
        let cache = cache_with_lines(vec![(0, 4), (5, 9), (10, 14), (15, 19), (20, 24)]);

        assert_eq!(
            TextArea::selection_hit_position_for_cache(&cache, point(px(-20.0), px(-10.0))),
            point(px(0.0), px(0.0))
        );
        assert_eq!(
            TextArea::selection_hit_position_for_cache(&cache, point(px(120.0), px(90.0))),
            point(px(99.5), px(59.5))
        );
        assert_eq!(
            TextArea::selection_hit_position_for_cache(&cache, point(px(-20.0), px(30.0))),
            point(px(0.0), px(30.0))
        );
        assert_eq!(
            TextArea::selection_hit_position_for_cache(&cache, point(px(120.0), px(30.0))),
            point(px(99.5), px(30.0))
        );
    }
}
