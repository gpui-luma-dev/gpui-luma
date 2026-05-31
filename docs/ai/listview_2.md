# ListView Phase 2 — Pagination, Scroll Snapping, and Dynamic Sizing

This document outlines the design and implementation specifications for the next version of `ListView<T>` in `gpui_luma`.

The goal is to transition the list view control from a simple fixed-height scroll container to a highly flexible, desktop-grade grid component supporting:
1. **Paging Mode**: Sliced item views; paging UI lives outside the control shell.
2. **Scroll Snapping**: Pixel-aligned snap-to-row physics.
3. **Visible Rows Auto-Sizing**: Dynamic container height matching a target number of rows.

**Status (May 2026):**

| Feature | State | SDK location | Gallery |
|---|---|---|---|
| **`visible_rows` shell sizing** | **Done** | `layout.rs`, `template.rs`, `control.rs` | `visible_rows = 10` on scroll-mode demo |
| **Paging mode** | Done | `control.rs`, `layout.rs`, `model.rs`, `paging.rs`, `toolbar.rs` | Gallery **Paging List View** pane via `PagingListViewBuilder` |
| **Scroll snapping** | Done (wheel + scroll handler) | `control.rs` | Not demoed yet |
| **Builder / macro / public API** | Done | `model.rs`, `macros.rs`, `control.rs`, `paging.rs` | Gallery uses raw builders in `shared.rs` |

**Remaining:** gallery coverage for paged + scroll-snap modes; optional spec polish (toolbar copy, page-size control widget); doc-only keyboard-nav behavior is outside this phase.

---

## 1. Configurable Scroll & Paging Modes

**Status: Done** — `ListScrollMode`, builder helpers, and public control methods are implemented.

`ListScrollMode` in `model.rs`:

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
    pub fn scroll_mode(mut self, mode: ListScrollMode) -> Self { /* ... */ }
    pub fn paged(self, page_size: usize) -> Self { /* ... */ }
    pub fn scroll_snap(self, enabled: bool) -> Self { /* ... */ }
}
```

Also implemented (not in original sketch): `set_scroll_mode` on the control, `visible_row_height` override, and `ListViewEvent::{PageChanged, PageSizeChanged}`.

### Public Programmatic API Control Surface
**Status: Done** on `ListViewControl<T>`:

- `current_page`, `page_size`, `page_count`
- `set_page`, `next_page`, `prev_page`, `first_page`, `last_page`
- `set_page_size`
- `scroll_to_row`, `scroll_by_pixels`

---

## 2. Implementation Specifications

### A. Paged Mode (No Scroll)

**Status: Done** — behavior matches intent; a few UI/details differ from the original sketch.

When `scroll_mode` is `ListScrollMode::Paged { page_size }`:

1. **State:** `current_page: usize` (default `0`), clamped on item/page-size changes.
2. **Items subset:** The control keeps the full `items` vector but sets GPUI `ListState` item count to the current page slice and maps local indices via `local_to_global_index` (equivalent to slicing, without copying rows).
3. **Body:** No scroll in paged mode; all rows on the page are visible within the fixed body height.
4. **No in-shell paging footer:** The list shell is header + body only. Paging UI is composed by `PagingListViewControl` via SDK [`toolbar.rs`](../../crates/sdk/src/controls/list_view/toolbar.rs).

### B. Scroll Snapping

**Status: Done** for wheel and programmatic scroll; drag-release snap relies on GPUI `ListState` scroll-handler callbacks (no separate deceleration hook).

When `scroll_mode` is `ListScrollMode::ScrollSnap`:

1. **Wheel:** `on_scroll_wheel` on the shell snaps by one row height per notch.
2. **Scroll handler:** `configure_scroll_handler` aligns offset to row boundaries after scroll events.
3. **Not combined with paged mode** — snap is disabled when `Paged`.

---

## 3. Dynamic Sizing based on "Visible Rows"

**Status: Done** — implemented in `layout.rs` + `template.rs` + `control.rs`. Gallery uses `visible_rows = 10` (scroll mode) and no longer needs a hardcoded `.h(px(...))` wrapper.

Fixed viewport height is **not scroll-mode-only**. Row-count sizing applies to both scroll and paged layouts:

- **Scroll modes:** set `visible_rows` explicitly, or the shell stays `h_full`.
- **Paged mode:** `page_size` drives body height automatically via `effective_visible_rows` even when `visible_rows` is unset.
- **Both set:** explicit `visible_rows` wins for shell height; `page_size` still controls how many rows are shown per page.

```rust
// layout.rs
pub fn effective_visible_rows(visible_rows: Option<usize>, scroll_mode: ListScrollMode) -> Option<usize> {
    visible_rows.or(match scroll_mode {
        ListScrollMode::Paged { page_size } => Some(page_size),
        _ => None,
    })
}
```

### Builder configuration
```rust
impl<T> ListViewBuilder<T> {
    pub fn visible_rows(mut self, count: usize) -> Self { /* ... */ }
    pub fn visible_row_height(mut self, height: f32) -> Self { /* optional override */ }
}
```

### Height calculation (implemented)

Logic lives in `layout.rs` (`body_rows_height`, `compute_shell_height`, `header_height`) and is consumed from `control.rs` → `ListViewRenderModel` → `DefaultListViewShellTemplate::paint_shell`.

**Row height (corrected from early sketch):** GPUI lays out rows at `min_height` when using `.min_h(min_height).py(padding_y)` on the same node — **not** `min_height + 2×padding_y`. See `default_row_height()` in `layout.rs`.

**Body height:** `count × row_height + (count − 1) × 1px` divider (`ROW_DIVIDER_WIDTH`).

**Header height:** `header_typography.line_height + padding_y + (padding_y × 0.75)` — matches the shell header slot in `template.rs`.

**Shell:** `body + header? + 2 × SHELL_BORDER_WIDTH`; root gets `.h(px(shell_height))`, body slot gets fixed `.h(px(body_rows_height))` + `overflow_hidden`.

**Scroll fix:** when `visible_rows` is set, `ListState` uses `measure_all()` so scroll range covers the full item list inside the fixed viewport.

### Example — paged list without explicit `visible_rows`

```rust
list_view! {
    id = "tasks-paged";
    items = tasks;
    page_size = 10;   // body sized for 10 rows
    /* ... */
}
```

### Example — scroll list with explicit viewport

```rust
list_view! {
    id = "tasks-scroll";
    items = tasks;
    visible_rows = 10;
    scroll_snap = true;   // optional; independent of sizing
    /* ... */
}
```

---

## 4. Macro Syntax Additions

**Status: Done** — optional clauses in `list_view!`:

```rust
list_view! {
    radix = radix_theme;
    id = "tasks-list";
    items = tasks;

    visible_rows = 10;          // shell/body height (scroll modes)
    page_size = 20;             // → .paged(20); also sizes shell if visible_rows omitted
    scroll_snap = true;         // → .scroll_snap(true)

    grid_view = {
        column!("Title" => |row| row.title.clone()),
    };
}
```

---

## 5. Validation & next steps

| Item | Notes |
|---|---|
| Gallery scroll + `visible_rows` | **Done** — `scrolling_list_view_pane.rs` |
| Gallery paged demo | **Done** — `paging_list_view_pane.rs` via `PagingListViewBuilder` |
| Gallery scroll-snap demo | Add `scroll_snap = true` or separate pane |
| Toolbar spec alignment | Optional: dropdown for page size; `X of Y selected` copy |
| `listview_2.md` | Updated with implementation status (this file) |
