//! Fixed-item geometry, independent of appearance and item content.

/// Physical list dimensions in logical pixels. Supply finite, nonnegative
/// dimensions. Item content padding, chrome, colors, and radii belong to the host/look.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ListBoxLayout {
    /// Allocated outer item height, including any item border.
    pub item_height: f32,
    /// Space between adjacent items on the flow axis.
    pub spacing: f32,
    /// Content viewport height, excluding the list's insets and border.
    pub viewport_height: f32,
    /// Horizontal list inset on each side.
    pub inset_x: f32,
    /// Vertical list inset on each side.
    pub inset_y: f32,
}

/// Explicit fixed-item flow. This never infers dimensions from a Rust template.
/// Horizontal flow fills the available viewport width with fixed-width cards.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ListBoxFlow {
    /// Reserve a viewport for this many equal-height items and intervening gaps.
    Vertical { visible_items: usize, item_height: f32, gap: f32 },
    /// One horizontal row of equal-sized cards.
    Horizontal { item_width: f32, item_height: f32, gap: f32 },
}

impl ListBoxFlow {
    /// Calculate content geometry with `(horizontal, vertical)` list insets.
    /// Zero visible vertical items reserve zero height, without an extra gap.
    pub fn layout(self, padding: (f32, f32)) -> ListBoxLayout {
        let (height, gap, viewport_height) = match self {
            Self::Vertical { visible_items, item_height, gap } => {
                (item_height, gap, item_height * visible_items as f32 + gap * visible_items.saturating_sub(1) as f32)
            }
            Self::Horizontal { item_height, gap, .. } => (item_height, gap, item_height),
        };
        ListBoxLayout { item_height: height, spacing: gap, viewport_height, inset_x: padding.0, inset_y: padding.1 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_count_reserves_only_inter_item_gaps() {
        for (count, expected) in [(0, 0.0), (1, 36.0), (5, 196.0)] {
            let layout =
                ListBoxFlow::Vertical { visible_items: count, item_height: 36.0, gap: 4.0 }.layout((16.0, 8.0));
            assert_eq!(layout.viewport_height, expected);
            assert_eq!((layout.inset_x, layout.inset_y), (16.0, 8.0));
        }
    }

    #[test]
    fn horizontal_viewport_height_is_independent_of_card_width_and_gap() {
        let layout = ListBoxFlow::Horizontal { item_width: 136.0, item_height: 36.0, gap: 8.0 }.layout((16.0, 8.0));
        assert_eq!(layout.viewport_height, 36.0);
        assert_eq!(layout.spacing, 8.0);
    }
}
