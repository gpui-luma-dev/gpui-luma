use gpui::{AnyElement, Div, IntoElement, div, px, prelude::*};
use std::collections::BTreeMap;

/// A lightweight grid-like layout primitive that compiles declarative track
/// definitions into nested flex containers.
///
/// Unlike CSS grid, `GridLayout` preserves GPUI's standard layout and
/// hit-testing flow by emitting normal `div().flex()` rows and fixed/star-sized
/// cell wrappers. Cells may span horizontally across multiple columns.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GridTrack {
    /// Fixed width in pixels.
    Px(f32),
    /// Proportional share of the remaining horizontal space.
    Star(f32),
}

pub struct GridChild {
    pub row: usize,
    pub col: usize,
    pub col_span: usize,
    pub element: AnyElement,
}

pub struct GridLayout {
    columns: Vec<GridTrack>,
    row_count: usize,
    gap_x: f32,
    gap_y: f32,
    children: Vec<GridChild>,
}

impl GridLayout {
    pub fn new() -> Self {
        Self { columns: Vec::new(), row_count: 0, gap_x: 0.0, gap_y: 0.0, children: Vec::new() }
    }

    pub fn columns(mut self, cols: impl IntoIterator<Item = GridTrack>) -> Self {
        self.columns = cols.into_iter().collect();
        self
    }

    pub fn rows(mut self, count: usize) -> Self {
        self.row_count = count;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap_x = gap;
        self.gap_y = gap;
        self
    }

    pub fn gap_x(mut self, gap_x: f32) -> Self {
        self.gap_x = gap_x;
        self
    }

    pub fn gap_y(mut self, gap_y: f32) -> Self {
        self.gap_y = gap_y;
        self
    }

    pub fn child(mut self, element: impl IntoElement, row: usize, col: usize) -> Self {
        self.children.push(GridChild { row, col, col_span: 1, element: element.into_any_element() });
        self.row_count = self.row_count.max(row + 1);
        self
    }

    pub fn child_with_span(mut self, element: impl IntoElement, row: usize, col: usize, col_span: usize) -> Self {
        self.children
            .push(GridChild { row, col, col_span: col_span.max(1), element: element.into_any_element() });
        self.row_count = self.row_count.max(row + 1);
        self
    }

    pub fn build(self) -> Div {
        let GridLayout { columns, row_count, gap_x, gap_y, children } = self;

        let mut root = div().flex().flex_col().w_full().min_w(px(0.0)).gap(px(gap_y));
        if columns.is_empty() || row_count == 0 {
            return root;
        }

        let mut children_by_row: BTreeMap<usize, Vec<GridChild>> = BTreeMap::new();
        for child in children {
            if child.col >= columns.len() {
                debug_assert!(
                    child.col < columns.len(),
                    "grid child column {} is out of bounds for {} columns",
                    child.col,
                    columns.len()
                );
                continue;
            }

            children_by_row.entry(child.row).or_default().push(child);
        }

        for row_children in children_by_row.values_mut() {
            row_children.sort_by_key(|child| child.col);
        }

        for row in 0..row_count {
            let row_children = children_by_row.remove(&row).unwrap_or_default();
            root = root.child(build_grid_row(&columns, gap_x, row_children));
        }

        root
    }

    pub fn into_any_element(self) -> AnyElement {
        self.build().into_any_element()
    }
}

impl Default for GridLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoElement for GridLayout {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        self.build()
    }
}

fn build_grid_row(columns: &[GridTrack], gap_x: f32, row_children: Vec<GridChild>) -> Div {
    let mut row = div().flex().w_full().min_w(px(0.0)).items_center().gap(px(gap_x));
    let mut row_children = row_children.into_iter().peekable();
    let mut col = 0;

    while col < columns.len() {
        while matches!(row_children.peek(), Some(child) if child.col < col) {
            debug_assert!(false, "overlapping grid child placement detected at row column {}", col);
            row_children.next();
        }

        if let Some(next_child) = row_children.peek()
            && next_child.col == col
        {
            let child = row_children.next().expect("peeked grid child should exist");
            let span = child.col_span.min(columns.len() - col).max(1);
            row = row.child(build_grid_cell(Some(child.element), &columns[col..col + span], gap_x));
            col += span;
            continue;
        }

        row = row.child(build_grid_cell(None, &columns[col..col + 1], gap_x));
        col += 1;
    }

    row
}

fn build_grid_cell(element: Option<AnyElement>, tracks: &[GridTrack], gap_x: f32) -> Div {
    let GridSpanSizing { fixed_width, star_weight } = measure_grid_span(tracks, gap_x);

    let mut cell = if star_weight > 0.0 {
        let mut cell = div().min_w(px(fixed_width)).flex_1();
        cell.style().flex_grow = Some(star_weight);
        cell
    } else {
        div().w(px(fixed_width)).flex_shrink_0()
    };

    if let Some(element) = element {
        cell = cell.child(element);
    }

    cell
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct GridSpanSizing {
    fixed_width: f32,
    star_weight: f32,
}

fn measure_grid_span(tracks: &[GridTrack], gap_x: f32) -> GridSpanSizing {
    let mut fixed_width = 0.0;
    let mut star_weight = 0.0;

    for track in tracks {
        match track {
            GridTrack::Px(width) => fixed_width += *width,
            GridTrack::Star(weight) => star_weight += *weight,
        }
    }

    if tracks.len() > 1 {
        fixed_width += gap_x * (tracks.len() - 1) as f32;
    }

    GridSpanSizing { fixed_width, star_weight }
}

#[cfg(test)]
mod tests {
    use super::{GridLayout, GridTrack, measure_grid_span};
    use gpui::{IntoElement, div};

    fn assert_into_element<E: IntoElement>(_element: E) {}

    #[test]
    fn grid_layout_is_into_element() {
        assert_into_element(
            GridLayout::new()
                .rows(2)
                .columns([GridTrack::Px(80.0), GridTrack::Star(1.0)])
                .child(div(), 0, 0)
                .child(div(), 1, 1),
        );
    }

    #[test]
    fn grid_layout_preserves_child_metadata() {
        let grid = GridLayout::new()
            .rows(2)
            .columns([GridTrack::Px(80.0), GridTrack::Star(1.0), GridTrack::Px(20.0)])
            .child(div(), 0, 0)
            .child_with_span(div(), 1, 1, 2);

        let placements: Vec<_> = grid.children.iter().map(|child| (child.row, child.col, child.col_span)).collect();
        assert_eq!(placements, vec![(0, 0, 1), (1, 1, 2)]);
    }

    #[test]
    fn grid_layout_tracks_max_row_from_children() {
        let grid = GridLayout::new().columns([GridTrack::Star(1.0)]).child(div(), 3, 0);
        assert_eq!(grid.row_count, 4);
    }

    #[test]
    fn grid_span_measurement_includes_internal_gap() {
        assert_eq!(
            measure_grid_span(&[GridTrack::Px(10.0), GridTrack::Px(20.0)], 4.0),
            super::GridSpanSizing { fixed_width: 34.0, star_weight: 0.0 }
        );
        assert_eq!(
            measure_grid_span(&[GridTrack::Px(12.0), GridTrack::Star(2.0), GridTrack::Star(1.0)], 5.0),
            super::GridSpanSizing { fixed_width: 22.0, star_weight: 3.0 }
        );
    }
}
