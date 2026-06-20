use super::model::ListScrollMode;
use super::template::SHELL_BORDER_WIDTH;
use super::theme::{ListViewLook, ListViewRowLook};

/// GPUI row elements use `min_h(min_height)` with `py(padding_y)` on the same node, so the
/// laid-out row height is [`ListViewRowLook::min_height`], not min_height + padding.
pub const ROW_DIVIDER_WIDTH: f32 = 1.0;

/// Per-row layout height for [`ListViewRowLook::min_height`] rows (see module comment).
pub fn default_row_height(row_look: &ListViewRowLook) -> f32 {
    row_look.min_height
}

pub fn visible_row_height(row_look: &ListViewRowLook, override_height: Option<f32>) -> f32 {
    override_height.unwrap_or_else(|| default_row_height(row_look))
}

/// Matches [`DefaultListViewShellTemplate::paint_shell`] header slot padding.
pub fn header_height(list_look: &ListViewLook) -> f32 {
    list_look.header_typography.line_height + list_look.padding_y + (list_look.padding_y * 0.75)
}

/// Total scroll-body height for `visible_rows` data rows, including inter-row dividers.
pub fn body_rows_height(visible_rows: usize, row_height: f32) -> f32 {
    let row_count = visible_rows as f32;
    let dividers = visible_rows.saturating_sub(1) as f32 * ROW_DIVIDER_WIDTH;
    row_count * row_height + dividers
}

pub fn compute_shell_height(visible_rows: usize, row_height: f32, list_look: &ListViewLook, has_header: bool) -> f32 {
    let mut total_height = body_rows_height(visible_rows, row_height);

    if has_header {
        total_height += header_height(list_look);
    }

    total_height + (2.0 * SHELL_BORDER_WIDTH)
}

pub fn effective_visible_rows(visible_rows: Option<usize>, scroll_mode: ListScrollMode) -> Option<usize> {
    visible_rows.or(match scroll_mode {
        ListScrollMode::Paged { page_size } => Some(page_size),
        _ => None,
    })
}

pub fn page_count(item_count: usize, page_size: usize) -> usize {
    let page_size = page_size.max(1);
    if item_count == 0 {
        1
    } else {
        item_count.div_ceil(page_size)
    }
}

pub fn page_start(current_page: usize, page_size: usize) -> usize {
    current_page.saturating_mul(page_size.max(1))
}

pub fn visible_item_count(item_count: usize, current_page: usize, page_size: usize) -> usize {
    let page_size = page_size.max(1);
    let start = page_start(current_page, page_size);
    if start >= item_count {
        0
    } else {
        (start + page_size).min(item_count) - start
    }
}

pub fn clamp_page(current_page: usize, item_count: usize, page_size: usize) -> usize {
    page_count(item_count, page_size).saturating_sub(1).min(current_page)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::LumaTextStyle;

    fn sample_row_look() -> ListViewRowLook {
        ListViewRowLook {
            background: gpui::transparent_black(),
            label_color: gpui::black(),
            divider: gpui::black(),
            adorner: None,
            label_typography: LumaTextStyle { size: 13.0, line_height: 18.0, weight: gpui::FontWeight::NORMAL },
            radius: 0.0,
            padding_x: 8.0,
            padding_y: 8.0,
            min_height: 36.0,
            label_baseline_shift: 1.0,
        }
    }

    fn sample_list_look() -> ListViewLook {
        ListViewLook {
            background: gpui::white(),
            border: gpui::black(),
            header_background: gpui::white(),
            header_label_color: gpui::black(),
            header_typography: LumaTextStyle { size: 11.0, line_height: 14.0, weight: gpui::FontWeight::MEDIUM },
            radius: 6.0,
            padding_x: 0.0,
            padding_y: 4.0,
        }
    }

    #[test]
    fn default_row_height_matches_gpui_min_h_layout() {
        let row = sample_row_look();
        assert_eq!(default_row_height(&row), 36.0);
    }

    #[test]
    fn body_rows_height_includes_dividers() {
        assert_eq!(body_rows_height(5, 36.0), 5.0 * 36.0 + 4.0);
    }

    #[test]
    fn header_height_matches_shell_header_slot() {
        let list = sample_list_look();
        assert_eq!(header_height(&list), 14.0 + 4.0 + 3.0);
    }

    #[test]
    fn page_count_handles_empty_items() {
        assert_eq!(page_count(0, 10), 1);
        assert_eq!(page_count(25, 10), 3);
    }

    #[test]
    fn visible_item_count_on_last_page() {
        assert_eq!(visible_item_count(25, 2, 10), 5);
        assert_eq!(visible_item_count(25, 0, 10), 10);
    }

    #[test]
    fn clamp_page_stays_in_bounds() {
        assert_eq!(clamp_page(99, 25, 10), 2);
    }
}
