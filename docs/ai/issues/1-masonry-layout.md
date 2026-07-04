# Issue: Lookless Column Layout for Card Galleries (Estimated Masonry)

## Description

Theme Studio's card gallery requires a responsive, column-first layout where cards stack vertically without vertical empty gaps. 

Since GPUI/Taffy does not support runtime bounding-box measurements during render-time tree construction, we implement a lookless estimated masonry layout:
1. **Column-first structure**: A horizontal flex row (`flex_row`) containing $N$ vertical columns (`flex_col`).
2. **Greedy placement**: Packing cards dynamically into columns by tracking caller-provided estimated card heights.
3. **Horizontal spans**: Allowing specific cards (like the 2x Payments card) to span across columns.

---

## Layout Concept & Placement Algorithms

Rather than forcing the parent app to manually bundle cards, `ColumnLayout` distributes items across columns using one of three strategies:

### 1. Sequential
Items are distributed round-robin (`column = index % N`). This is simple and works well when all cards have identical heights.

### 2. Greedy By Estimated Height (`GreedyByEstimatedHeight`)
To stack cards snugly, the primitive tracks the cumulative estimated height of each column. Each card is placed in the column that is currently the "shortest".
* Height weights are resolved from the child's `TileHeightClass` or estimated height metadata.

### 3. Manual Columns (`Manual`)
An escape hatch where the application passes pre-partitioned lists of elements directly to specific columns.

---

## Proposed SDK API

We introduce this primitive under `crates/sdk/src/layouts/column_layout.rs` (exported as `ColumnLayout` from `crates/sdk/src/layout.rs`).

### Tile Metadata

Each element is wrapped in a `ColumnTile` to define its layout properties:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileHeightClass {
    Compact, // E.g., ~200px
    Normal,  // E.g., ~400px
    Wide,    // E.g., ~600px
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
    pub element: gpui::AnyElement,
    pub col_span: usize,
    pub height_class: TileHeightClass,
}

impl ColumnTile {
    pub fn new(element: impl IntoElement) -> Self {
        Self {
            element: element.into_any_element(),
            col_span: 1,
            height_class: TileHeightClass::Normal,
        }
    }

    pub fn col_span(mut self, span: usize) -> Self {
        self.col_span = span;
        self
    }

    pub fn height_class(mut self, class: TileHeightClass) -> Self {
        self.height_class = class;
        self
    }
}
```

### Layout Container API

The layout container coordinates partitioning, column constraints, and child wrapping:

```rust
pub enum ColumnPlacementStrategy {
    Sequential,
    GreedyByEstimatedHeight,
    Manual,
}

pub struct ColumnLayout {
    columns: Option<usize>,
    target_tile_width: Option<f32>,
    gap_x: f32,
    gap_y: f32,
    strategy: ColumnPlacementStrategy,
    tiles: Vec<ColumnTile>,
    manual_columns: Vec<Vec<gpui::AnyElement>>,
}

impl ColumnLayout {
    pub fn new() -> Self {
        Self {
            columns: None,
            target_tile_width: None,
            gap_x: 0.0,
            gap_y: 0.0,
            strategy: ColumnPlacementStrategy::GreedyByEstimatedHeight,
            tiles: Vec::new(),
            manual_columns: Vec::new(),
        }
    }

    pub fn columns(mut self, count: usize) -> Self {
        self.columns = Some(count);
        self
    }

    pub fn target_tile_width(mut self, width: f32) -> Self {
        self.target_tile_width = Some(width);
        self
    }

    pub fn gap_x(mut self, gap: f32) -> Self {
        self.gap_x = gap;
        self
    }

    pub fn gap_y(mut self, gap: f32) -> Self {
        self.gap_y = gap;
        self
    }

    pub fn strategy(mut self, strategy: ColumnPlacementStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn tile(mut self, tile: ColumnTile) -> Self {
        self.tiles.push(tile);
        self
    }

    pub fn tiles(mut self, tiles: impl IntoIterator<Item = ColumnTile>) -> Self {
        self.tiles.extend(tiles);
        self
    }
}
```

---

## Column Calculation & Width Responsiveness

Rather than forcing the parent views to compute columns from viewport widths, `ColumnLayout` can dynamically determine the column count internally if `target_tile_width` is provided:
1. The container queries its current width constraints during the layout pass.
2. It computes the column count: `columns = (available_width + gap_x) / (target_tile_width + gap_x)`.
3. If the computed column count changes, the placement algorithm runs to partition the elements into columns.

### Multi-Column Spanning (Horizontal Spans)

If a tile specifies `col_span: 2`:
* The greedy placement algorithm finds adjacent columns $j$ and $j+1$ that have the lowest combined height.
* The element is rendered inside a spanning layout structure (e.g., using `absolute` positioning or partitioned layout wrappers within the row container) to align correctly.

---

## Theme Studio Card Integration

Migrating the Theme Studio cards panel to use `ColumnLayout`:

```rust
ColumnLayout::new()
    .target_tile_width(380.0) // Computes N columns automatically
    .gap_x(16.0)
    .gap_y(16.0)
    .strategy(ColumnPlacementStrategy::GreedyByEstimatedHeight)
    .tiles(all_panels.into_iter().map(|(id, child)| {
        let span = if id == InspectableId::Payments { 2 } else { 1 };
        let height_class = match id {
            InspectableId::CreateAccount | InspectableId::Chat => TileHeightClass::Compact,
            InspectableId::Payments | InspectableId::UpgradeSubscription => TileHeightClass::Wide,
            _ => TileHeightClass::Normal,
        };
        ColumnTile::new(child)
            .col_span(span)
            .height_class(height_class)
    }))
```

---

## Verification Plan

### Automated Tests
* Test that sequential layout distributes items round-robin.
* Test that `GreedyByEstimatedHeight` places a new item in the column with the lowest accumulated height.
* Test dynamic column calculation from container width constraints.

### Manual Verification
* Resize the Theme Studio window and ensure columns wrap and adjust.
* Verify the 2x Payments card spans and aligns with the adjacent columns correctly without creating layout overlaps.
