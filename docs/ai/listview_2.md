# ListView Phase 2 — Pagination, Scroll Snapping, and Dynamic Sizing

This document outlines the design and implementation specifications for the next version of `ListView<T>` in `gpui_luma`.

The goal is to transition the list view control from a simple fixed-height scroll container to a highly flexible, desktop-grade grid component supporting:
1. **Paging Mode**: Sliced item views with a standard pagination footer.
2. **Scroll Snapping**: Pixel-aligned snap-to-row physics.
3. **Visible Rows Auto-Sizing**: Dynamic container height matching a target number of rows.

---

## 1. Configurable Scroll & Paging Modes

We will introduce a `ListScrollMode` enum in `model.rs` and make it configurable via the builder:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ListScrollMode {
    /// Smooth pixel-by-pixel scrolling (current default)
    ScrollSmooth,
    /// Smooth scrolling with snap-to-row boundaries on release
    ScrollSnap,
    /// Paginated layout displaying N items at a time with page selectors
    Paged { page_size: usize },
}
```

### Builder API Additions
```rust
impl<T> ListViewBuilder<T> {
    /// Configures the scroll or pagination mode
    pub fn scroll_mode(mut self, mode: ListScrollMode) -> Self {
        self.model.scroll_mode = mode;
        self
    }

    /// Convenience helper to set paged mode
    pub fn paged(self, page_size: usize) -> Self {
        self.scroll_mode(ListScrollMode::Paged { page_size })
    }

    /// Convenience helper to enable scroll snapping
    pub fn scroll_snap(self, enabled: bool) -> Self {
        if enabled {
            self.scroll_mode(ListScrollMode::ScrollSnap)
        } else {
            self.scroll_mode(ListScrollMode::ScrollSmooth)
        }
    }
}
```

### Public Programmatic API Control Surface
To support custom pagination layouts designed entirely outside of the default list view templates, the control surface on `ListViewControl<T>` will expose public methods for paging actions:

```rust
impl<T> ListViewControl<T> {
    /// Returns the active page index (0-indexed)
    pub fn current_page(&self) -> usize;

    /// Returns the page size if currently in Paged mode
    pub fn page_size(&self) -> Option<usize>;

    /// Returns the total page count based on items count and page size
    pub fn page_count(&self) -> usize;

    /// Programmatically sets the active page index (clamped to page boundaries)
    pub fn set_page(&mut self, page: usize, cx: &mut Context<Self>);

    /// Programmatically navigates to the next page
    pub fn next_page(&mut self, cx: &mut Context<Self>);

    /// Programmatically navigates to the previous page
    pub fn prev_page(&mut self, cx: &mut Context<Self>);

    /// Programmatically navigates to the first page
    pub fn first_page(&mut self, cx: &mut Context<Self>);

    /// Programmatically navigates to the last page
    pub fn last_page(&mut self, cx: &mut Context<Self>);

    /// Changes the active page size dynamically
    pub fn set_page_size(&mut self, page_size: usize, cx: &mut Context<Self>);

    /// Programmatically scrolls the list to bring a specific row index into view
    pub fn scroll_to_row(&mut self, index: usize, cx: &mut Context<Self>);

    /// Programmatically scrolls the list by a specific pixel offset (smooth or snapped)
    pub fn scroll_by_pixels(&mut self, pixels: f32, cx: &mut Context<Self>);
}
```

---

## 2. Implementation Specifications

### A. Paged Mode (No Scroll)
When `scroll_mode` resolves to `ListScrollMode::Paged { page_size }`:
1. **State Slicing**: The list control maintains an active page index (`current_page: usize`, defaulting to `0`).
2. **Items Subset**: Instead of feeding the entire `items` vector to GPUI's `ListState`, the control slices the items for the current page:
   ```rust
   let start = current_page * page_size;
   let end = (start + page_size).min(items.len());
   let page_items = &items[start..end];
   ```
3. **Default Pagination Toolbar**: The `DefaultListViewShellTemplate` renders a default toolbar at the bottom matching standard layouts:
   - **Selection Summary (Left)**: Renders `X of Y row(s) selected.` dynamically by reading the list view selection count.
   - **Rows per Page (Center-Right)**: A drop-down select element to change the page size dynamically (e.g. 10, 25, 50, 100).
   - **Page Indicator (Right)**: Shows text like `Page X of Y`.
   - **Controls (Far Right)**: Navigation buttons: `<<` (first page), `<` (previous page), `>` (next page), and `>>` (last page) that update the active page slice and trigger `cx.notify()`.

4. **Custom Toolbar Template Slot**: Allow developers to override the default toolbar layout by passing a custom template closure/function via the builder:
   ```rust
   pub type ListViewPagingToolbarTemplate<T> = Arc<
       dyn Fn(&ListViewModel<T>, &mut Window, &mut App) -> AnyElement + Send + Sync + 'static
   >;

   impl<T> ListViewBuilder<T> {
       pub fn paging_toolbar_template(mut self, template: ListViewPagingToolbarTemplate<T>) -> Self {
           self.model.paging_toolbar_template = Some(template);
           self
       }
   }
   ```
   If a custom template is provided, the shell template delegates footer rendering directly to the custom toolbar closure instead of drawing the default one.

### B. Scroll Snapping
When `scroll_mode` is set to `ListScrollMode::ScrollSnap`:
1. **Snapping Wheel Events**: To snap the scroll position to clean row boundaries during discrete scroll actions, the outer container intercepts mouse-wheel scroll events:
   ```rust
   .on_scroll(|event, cx| {
       // Check if scroll is vertical and snap to the nearest row height increment
   })
   ```
2. **Snapping Drag Release**: Trackpad gestures can scroll smoothly, but on release (when deceleration terminates), the list computes the current offset and triggers a snapping scroll transition to align the top row boundary:
   ```rust
   self.list_state.scroll_to(snapped_offset);
   ```

---

## 3. Dynamic Sizing based on "Visible Rows"

To avoid hardcoded container heights (e.g. `.h(px(420.0))`), the builder will support a `visible_rows` parameter. This adjusts the container height dynamically based on active theme metrics, ensuring no "partial" rows are displayed.

### Builder configuration
```rust
impl<T> ListViewBuilder<T> {
    pub fn visible_rows(mut self, count: usize) -> Self {
        self.model.visible_rows = Some(count);
        self
    }
}
```

### Height Calculation (template.rs)
Inside `DefaultListViewShellTemplate::paint_shell`, if `visible_rows` is configured, we compute the target height bounds programmatically:

```rust
if let Some(count) = model.visible_rows {
    // 1. Calculate row height = min_height + double vertical padding
    let row_height = row_appearance.min_height + (2.0 * row_appearance.padding_y);
    let mut total_height = (count as f32) * row_height;

    // 2. Add header height if column headers are visible
    if has_header {
        let header_height = list_appearance.header_typography.size + (2.0 * list_appearance.padding_y);
        total_height += header_height;
    }

    // 3. Add pagination footer height if paged mode is active
    if let ListScrollMode::Paged { .. } = model.scroll_mode {
        let footer_height = metrics.control_height(model.size);
        total_height += footer_height;
    }

    // 4. Add shell borders
    total_height += 2.0 * SHELL_BORDER_WIDTH;

    // Style root container with target pixel height instead of h_full
    root = root.h(px(total_height));
} else {
    root = root.h_full();
}
```

---

## 4. Macro Syntax Additions

The `list_view!` macro in `macros.rs` will be extended to accept these optional clauses:

```rust
list_view! {
    radix = radix_theme;
    id = "tasks-list";
    items = tasks;
    
    // Sizing (optional)
    visible_rows = 10;
    
    // Pagination / Scrolling Configuration (optional)
    page_size = 20;       // Macro maps to ListScrollMode::Paged
    scroll_snap = true;   // Macro maps to ListScrollMode::ScrollSnap
    
    // Custom Paging Toolbar Slot (optional override)
    paging_toolbar_template = |model, _window, _cx| {
        div().child("Custom paging toolbar")
    };
    
    grid_view = {
        column!("Title" => |row| row.title.clone()),
    };
}
```
