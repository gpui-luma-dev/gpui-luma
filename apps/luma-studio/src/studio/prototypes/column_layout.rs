use gpui::{AnyElement, Div, IntoElement, div, px, prelude::*};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TileHeightClass {
    Compact,
    #[default]
    Normal,
    Wide,
}

impl TileHeightClass {
    pub fn estimated_height(self) -> f32 {
        match self {
            Self::Compact => 200.0,
            Self::Normal => 400.0,
            Self::Wide => 600.0,
        }
    }
}

pub struct ColumnTile {
    pub element: AnyElement,
    pub height_class: TileHeightClass,
}

impl ColumnTile {
    pub fn new(element: impl IntoElement) -> Self {
        Self { element: element.into_any_element(), height_class: TileHeightClass::Normal }
    }

    pub fn height_class(mut self, class: TileHeightClass) -> Self {
        self.height_class = class;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColumnPlacementStrategy {
    #[default]
    GreedyByEstimatedHeight,
}

pub struct ColumnLayout {
    columns: usize,
    column_min_width: f32,
    column_preferred_width: Option<f32>,
    gap_x: f32,
    gap_y: f32,
    strategy: ColumnPlacementStrategy,
    tiles: Vec<ColumnTile>,
}

impl ColumnLayout {
    pub fn new() -> Self {
        Self {
            columns: 1,
            column_min_width: 320.0,
            column_preferred_width: None,
            gap_x: 0.0,
            gap_y: 0.0,
            strategy: ColumnPlacementStrategy::GreedyByEstimatedHeight,
            tiles: Vec::new(),
        }
    }

    pub fn columns(mut self, count: usize) -> Self {
        self.columns = count.max(1);
        self
    }

    pub fn column_min_width(mut self, column_min_width: f32) -> Self {
        self.column_min_width = column_min_width.max(1.0);
        self
    }

    pub fn column_preferred_width(mut self, column_preferred_width: f32) -> Self {
        self.column_preferred_width = Some(column_preferred_width.max(self.column_min_width));
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

    pub fn strategy(mut self, strategy: ColumnPlacementStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn tiles(mut self, tiles: impl IntoIterator<Item = ColumnTile>) -> Self {
        self.tiles.extend(tiles);
        self
    }

    pub fn build(self) -> Div {
        let ColumnLayout { columns, column_min_width, column_preferred_width, gap_x, gap_y, strategy, tiles } = self;
        let partitioned = partition_tiles(columns, strategy, tiles);

        let mut root = div().w_full().min_w(px(0.0)).flex().flex_wrap().items_start().content_start();
        root.style().gap.width = Some(px(gap_x).into());
        root.style().gap.height = Some(px(gap_y).into());

        for column_tiles in partitioned {
            let mut column = div().flex_1().min_w(px(column_min_width)).flex().flex_col();
            if let Some(column_preferred_width) = column_preferred_width {
                column = column.flex_basis(px(column_preferred_width));
            }
            column.style().gap.height = Some(px(gap_y).into());

            for tile in column_tiles {
                column = column.child(div().w_full().child(tile.element));
            }

            root = root.child(column);
        }

        root
    }
}

impl Default for ColumnLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoElement for ColumnLayout {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        self.build()
    }
}

fn partition_tiles(columns: usize, _strategy: ColumnPlacementStrategy, tiles: Vec<ColumnTile>) -> Vec<Vec<ColumnTile>> {
    let columns = columns.max(1);
    let mut partitioned: Vec<Vec<ColumnTile>> = (0..columns).map(|_| Vec::new()).collect();
    let mut heights = vec![0.0_f32; columns];

    for tile in tiles {
        let (shortest_index, _) = heights
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal))
            .expect("at least one column");
        heights[shortest_index] += tile.height_class.estimated_height();
        partitioned[shortest_index].push(tile);
    }

    partitioned
}

#[cfg(test)]
mod tests {
    use super::{ColumnLayout, ColumnPlacementStrategy, ColumnTile, TileHeightClass, partition_tiles};
    use gpui::{IntoElement, div};

    fn assert_into_element<E: IntoElement>(_element: E) {}

    #[test]
    fn column_layout_is_into_element() {
        assert_into_element(
            ColumnLayout::new()
                .columns(3)
                .gap_x(16.0)
                .gap_y(16.0)
                .strategy(ColumnPlacementStrategy::GreedyByEstimatedHeight)
                .tiles([ColumnTile::new(div()).height_class(TileHeightClass::Compact)]),
        );
    }

    #[test]
    fn greedy_partition_uses_shortest_estimated_column() {
        let partitioned = partition_tiles(
            2,
            ColumnPlacementStrategy::GreedyByEstimatedHeight,
            vec![
                ColumnTile::new(div()).height_class(TileHeightClass::Wide),
                ColumnTile::new(div()).height_class(TileHeightClass::Compact),
                ColumnTile::new(div()).height_class(TileHeightClass::Compact),
            ],
        );

        let heights: Vec<_> = partitioned
            .iter()
            .map(|column| column.iter().map(|tile| tile.height_class.estimated_height()).sum::<f32>())
            .collect();
        assert_eq!(heights, vec![600.0, 400.0]);
    }
}
