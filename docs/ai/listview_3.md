# ListView Phase 3 — Facade Re-unification and Explicit Macros

This plan outlines pulling the paging toolbar into the SDK `list_view` folder and introducing explicit facades and macros for `scrolling_list_view` and `paging_list_view`.

The core goal is **minimal impact on the existing `ListViewControl` code**, while creating a facade that fully manages paging state synchronization under the hood.

## Phased Execution Strategy

To isolate compiler diagnostics and ensure logic correctness before introducing complex macro expansion rules, the refactoring will proceed in two distinct phases:

### Phase 1: Core Builders & Facades Integration — **Done**

We first implement the type-safe primitives and composite control using standard Rust structures:

1. ✅ Move the paging toolbar from the gallery to the SDK as [`toolbar.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/list_view/toolbar.rs).
2. ✅ Implement the composite facade `PagingListViewControl` and its builder `PagingListViewBuilder` in [`paging.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/list_view/paging.rs).
3. ✅ Expose the new types in [`mod.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/list_view/mod.rs).
4. ✅ Migrate the gallery panes to raw builder syntax:
   - [`paging_list_view_pane.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/list_view/paging_list_view_pane.rs) — `PagingListViewBuilder::new(...).spawn(cx)`
   - [`scrolling_list_view_pane.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/list_view/scrolling_list_view_pane.rs) — `ListViewBuilder::visible_rows(...).spawn(cx)`
   - Shared task grid/data in [`shared.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/list_view/shared.rs)
5. ✅ Compile and test — SDK unit tests pass; gallery paging pane no longer owns toolbar sync state.

### Phase 2: Declarative Macro Rollout
Once the builder integration is fully verified and warning-free, we add the declarative syntactic sugar:
1. Implement the `scrolling_list_view!` and `paging_list_view!` macros inside [`macros.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/list_view/macros.rs).
2. Clean up the gallery pane's instantiation code to use the new macros instead of the raw builder methods.

---

## 1. Directory & File Organization

The new SDK structure will pull in the pagination toolbar from the gallery and introduce a paging facade:

```
crates/sdk/src/controls/list_view/
  ├── mod.rs                # Exposes ScrollingListView, PagingListView, builders
  ├── control.rs            # Core list logic (ListViewControl<T>) - UNCHANGED
  ├── paging.rs             # [NEW] PagingListViewControl facade & PagingListViewBuilder
  ├── toolbar.rs            # [NEW] PagingToolbar control (moved from gallery)
  ├── row.rs                # Row wrapper - UNCHANGED
  ├── column_template.rs    # Column cell helpers - UNCHANGED
  ├── layout.rs             # Height calculations - UNCHANGED
  ├── macros.rs             # Explicit scrolling_list_view! and paging_list_view! macros
  ├── template.rs           # Shell template - UNCHANGED
  └── theme.rs              # Colors and themes - UNCHANGED
```

---

## 2. Built-in Paging Toolbar (`toolbar.rs`)

We move the paging toolbar from the gallery to the SDK crate. It renders the default Radix pagination controls:

```rust
// crates/sdk/src/controls/list_view/toolbar.rs

use std::sync::Arc;
use gpui::{AnyElement, ClickEvent, Context, EventEmitter, Render, Window, div, prelude::*, px};
use crate::controls::icon::lucide_glyph;
use crate::theme::RadixTheme;
use lucide_icons::Icon as LucideIcon;

const PAGE_SIZE_OPTIONS: [usize; 2] = [10, 25];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PagingToolbarLayout {
    pub selected_count: usize,
    pub total_rows: usize,
    pub current_page: usize,
    pub page_count: usize,
    pub page_size: usize,
}

impl PagingToolbarLayout {
    fn selection_summary(&self) -> String {
        format!("{} of {} row(s) selected.", self.selected_count, self.total_rows)
    }

    fn page_indicator(&self) -> String {
        format!("Page {} of {}", self.current_page + 1, self.page_count.max(1))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PagingToolbarEvent {
    FirstPage,
    PrevPage,
    NextPage,
    LastPage,
    SetPageSize(usize),
}

pub type PagingToolbarTemplate = Arc<
    dyn Fn(&PagingToolbarLayout, &mut Window, &mut App) -> AnyElement
        + Send
        + Sync
        + 'static,
>;

pub struct PagingToolbar {
    theme: Arc<RadixTheme>,
    layout: PagingToolbarLayout,
    page_size_open: bool,
    custom_template: Option<PagingToolbarTemplate>,
}

impl EventEmitter<PagingToolbarEvent> for PagingToolbar {}

impl PagingToolbar {
    pub fn new(theme: Arc<RadixTheme>, layout: PagingToolbarLayout) -> Self {
        Self {
            theme,
            layout,
            page_size_open: false,
            custom_template: None,
        }
    }

    pub fn with_custom_template(mut self, template: PagingToolbarTemplate) -> Self {
        self.custom_template = Some(template);
        self
    }

    pub fn set_layout(&mut self, layout: PagingToolbarLayout, cx: &mut Context<Self>) {
        if self.layout != layout {
            self.layout = layout;
            cx.notify();
        }
    }

    pub fn update_selection(&mut self, selected_count: usize, total_rows: usize, cx: &mut Context<Self>) {
        if self.layout.selected_count != selected_count || self.layout.total_rows != total_rows {
            self.layout.selected_count = selected_count;
            self.layout.total_rows = total_rows;
            cx.notify();
        }
    }

    pub fn update_page(&mut self, current_page: usize, page_size: usize, page_count: usize, cx: &mut Context<Self>) {
        if self.layout.current_page != current_page
            || self.layout.page_size != page_size
            || self.layout.page_count != page_count
        {
            self.layout.current_page = current_page;
            self.layout.page_size = page_size;
            self.layout.page_count = page_count;
            cx.notify();
        }
    }
}

impl Render for PagingToolbar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(template) = &self.custom_template {
            return template(&self.layout, window, cx.app_mut());
        }

        let chrome = self.theme.chrome();
        let layout = self.layout;
        let at_first = layout.current_page == 0;
        let at_last = layout.current_page + 1 >= layout.page_count.max(1);

        div()
            .w_full()
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(12.0))
            .py(px(8.0))
            .text_color(chrome.muted_text)
            .text_size(px(11.0))
            .line_height(px(14.0))
            .child(div().flex_1().min_w(px(0.0)).truncate().child(layout.selection_summary()))
            .child(
                div()
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap(px(16.0))
                    .child(
                        div()
                            .flex_none()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(div().flex_none().child("Rows per page"))
                            .child(render_page_size_select(cx, &self.theme, layout.page_size, self.page_size_open)),
                    )
                    .child(
                        div()
                            .flex_none()
                            .text_size(px(11.0))
                            .line_height(px(14.0))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(chrome.body_text)
                            .child(layout.page_indicator()),
                    )
                    .child(
                        div()
                            .flex_none()
                            .flex()
                            .items_center()
                            .gap(px(2.0))
                            .child(render_nav_button(cx, &self.theme, "<<", at_first, PagingToolbarEvent::FirstPage))
                            .child(render_nav_button(cx, &self.theme, "<", at_first, PagingToolbarEvent::PrevPage))
                            .child(render_nav_button(cx, &self.theme, ">", at_last, PagingToolbarEvent::NextPage))
                            .child(render_nav_button(cx, &self.theme, ">>", at_last, PagingToolbarEvent::LastPage)),
                    ),
            )
            .into_any_element()
    }
}

fn render_page_size_select(
    cx: &mut Context<PagingToolbar>,
    theme: &RadixTheme,
    page_size: usize,
    open: bool,
) -> impl IntoElement {
    let chrome = theme.chrome();

    div()
        .relative()
        .flex_none()
        .w(px(56.0))
        .on_mouse_down_out(cx.listener(|this, _, _, cx| {
            if this.page_size_open {
                this.page_size_open = false;
                cx.notify();
            }
        }))
        .child(
            div()
                .id("paging-page-size-trigger")
                .w_full()
                .h(px(28.0))
                .flex()
                .items_center()
                .justify_between()
                .gap(px(4.0))
                .px(px(8.0))
                .rounded(px(6.0))
                .border_1()
                .border_color(chrome.border)
                .bg(chrome.panel_background)
                .text_color(chrome.body_text)
                .cursor_pointer()
                .child(format!("{page_size}"))
                .child(div().text_color(chrome.muted_text).child(lucide_glyph(if open {
                    LucideIcon::ChevronUp
                } else {
                    LucideIcon::ChevronDown
                })))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.page_size_open = !this.page_size_open;
                    cx.notify();
                })),
        )
        .when(open, |slot| {
            slot.child(
                div()
                    .absolute()
                    .top(px(32.0))
                    .left(px(0.0))
                    .occlude()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .p(px(4.0))
                    .min_w(px(56.0))
                    .rounded(px(6.0))
                    .border_1()
                    .border_color(chrome.border)
                    .bg(chrome.panel_background)
                    .shadow_md()
                    .children(PAGE_SIZE_OPTIONS.iter().copied().map(|option| {
                        let is_selected = option == page_size;
                        div()
                            .id(format!("paging-page-size-{option}"))
                            .px(px(8.0))
                            .py(px(4.0))
                            .rounded(px(4.0))
                            .when(is_selected, |row| row.bg(chrome.border))
                            .text_size(px(11.0))
                            .line_height(px(14.0))
                            .text_color(chrome.body_text)
                            .cursor_pointer()
                            .child(format!("{option}"))
                            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                                this.page_size_open = false;
                                cx.emit(PagingToolbarEvent::SetPageSize(option));
                                cx.notify();
                            }))
                    })),
            )
        })
}

fn render_nav_button(
    cx: &mut Context<PagingToolbar>,
    theme: &RadixTheme,
    label: &'static str,
    disabled: bool,
    event: PagingToolbarEvent,
) -> impl IntoElement {
    let chrome = theme.chrome();

    div()
        .id(format!("paging-nav-{label}"))
        .flex_none()
        .size(px(28.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(6.0))
        .border_1()
        .border_color(chrome.border)
        .bg(chrome.panel_background)
        .text_color(chrome.body_text)
        .text_size(px(11.0))
        .line_height(px(14.0))
        .font_weight(gpui::FontWeight::MEDIUM)
        .when(disabled, |slot| slot.opacity(0.4))
        .when(!disabled, |slot| slot.cursor_pointer())
        .child(label)
        .when(!disabled, |slot| {
            slot.on_click(cx.listener(move |_, _, _, cx| {
                cx.emit(event);
            }))
        })
}
```

---

## 3. Paging Facade & Builders (`paging.rs`)

The facade maps standard `ListViewEvent` notifications to toolbar layout updates, listens to toolbar navigation commands, and bubbles up outer events:

```rust
// crates/sdk/src/controls/list_view/paging.rs

use std::sync::Arc;
use gpui::{AnyElement, AppContext, Context, Entity, EventEmitter, Render, Window, div, prelude::*};
use crate::theme::RadixTheme;
use super::control::{ListViewControl, ListViewEvent};
use super::toolbar::{PagingToolbar, PagingToolbarEvent, PagingToolbarLayout, PagingToolbarTemplate};
use super::model::ListViewBuilder;

pub type PagingListView<T> = Entity<PagingListViewControl<T>>;

pub struct PagingListViewControl<T: 'static> {
    list: Entity<ListViewControl<T>>,
    toolbar: Entity<PagingToolbar>,
}

impl<T: 'static> PagingListViewControl<T> {
    pub fn new(list: Entity<ListViewControl<T>>, toolbar: Entity<PagingToolbar>, cx: &mut Context<Self>) -> Self {
        // Sync List changes -> Toolbar
        let toolbar_clone = toolbar.clone();
        cx.subscribe(&list, move |_, list_control, event, cx| {
            match event {
                ListViewEvent::SelectionChanged { selected_indices } => {
                    let count = selected_indices.len();
                    let total = list_control.items().len();
                    toolbar_clone.update(cx, |tb, cx| tb.update_selection(count, total, cx));
                }
                ListViewEvent::PageChanged { page } => {
                    let page_size = list_control.page_size().unwrap_or(10);
                    let page_count = list_control.page_count();
                    toolbar_clone.update(cx, |tb, cx| tb.update_page(*page, page_size, page_count, cx));
                }
                ListViewEvent::PageSizeChanged { page_size } => {
                    let current_page = list_control.current_page();
                    let page_count = list_control.page_count();
                    toolbar_clone.update(cx, |tb, cx| tb.update_page(current_page, *page_size, page_count, cx));
                }
                _ => {}
            }
        }).detach();

        // Forward Toolbar clicks -> List
        let list_clone = list.clone();
        cx.subscribe(&toolbar, move |_, _, event, cx| {
            list_clone.update(cx, |list_control, cx| {
                match event {
                    PagingToolbarEvent::FirstPage => list_control.first_page(cx),
                    PagingToolbarEvent::PrevPage => list_control.prev_page(cx),
                    PagingToolbarEvent::NextPage => list_control.next_page(cx),
                    PagingToolbarEvent::LastPage => list_control.last_page(cx),
                    PagingToolbarEvent::SetPageSize(size) => list_control.set_page_size(*size, cx),
                }
            });
        }).detach();

        // Forward Selection/Active events out to parent views
        cx.subscribe(&list, |_, _, event, cx| {
            cx.emit(event.clone());
        }).detach();

        Self { list, toolbar }
    }

    pub fn list(&self) -> &Entity<ListViewControl<T>> {
        &self.list
    }

    pub fn toolbar(&self) -> &Entity<PagingToolbar> {
        &self.toolbar
    }
}

impl<T: 'static> EventEmitter<ListViewEvent> for PagingListViewControl<T> {}

impl<T: 'static> Render for PagingListViewControl<T> {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .flex()
            .flex_col()
            .child(self.list.clone())
            .child(self.toolbar.clone())
    }
}

pub struct PagingListViewBuilder<T> {
    list_builder: ListViewBuilder<T>,
    theme: Arc<RadixTheme>,
    toolbar_template: Option<PagingToolbarTemplate>,
}

impl<T: 'static> PagingListViewBuilder<T> {
    pub fn new(list_builder: ListViewBuilder<T>, theme: Arc<RadixTheme>) -> Self {
        Self {
            list_builder,
            theme,
            toolbar_template: None,
        }
    }

    pub fn toolbar_template(mut self, template: PagingToolbarTemplate) -> Self {
        self.toolbar_template = Some(template);
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> PagingListView<T> {
        let list = cx.new(|cx| ListViewControl::from_builder(self.list_builder, cx));

        let initial_layout = {
            let list_read = list.read(cx);
            PagingToolbarLayout {
                selected_count: list_read.selected_indices().len(),
                total_rows: list_read.items().len(),
                current_page: list_read.current_page(),
                page_count: list_read.page_count(),
                page_size: list_read.page_size().unwrap_or(10),
            }
        };

        let mut toolbar = PagingToolbar::new(self.theme.clone(), initial_layout);
        if let Some(template) = self.toolbar_template {
            toolbar = toolbar.with_custom_template(template);
        }

        let toolbar_entity = cx.new(|_| toolbar);
        cx.new(|cx| PagingListViewControl::new(list, toolbar_entity, cx))
    }
}
```

---

## 4. Explicit Macros (`macros.rs`)

We replace the implicit `list_view!` macro with two explicit, highly visible macro configurations:

```rust
// crates/sdk/src/controls/list_view/macros.rs

#[macro_export]
macro_rules! scrolling_list_view {
    // grid_view + row_template
    (
        radix = $radix:expr;
        id = $id:expr;
        items = $items:expr;
        $( selection = $selection:expr; )?
        $( selected_index = $selected_index:expr; )?
        $( active_index = $active_index:expr; )?
        $( row_label = |$label_row:ident| $label_body:expr; )?
        $( row_enabled = |$enabled_row:ident| $enabled_body:expr; )?
        $( visible_rows = $visible_rows:expr; )?
        $( scroll_snap = $scroll_snap:expr; )?
        grid_view = { $($col:expr),* $(,)? };
        row_template = |$model:ident, $cells:ident, $win:ident, $cx:ident| $body:expr $(;)?
    ) => {{
        let builder = $crate::controls::list_view::new_typed($id).items($items);
        let builder = builder.theme($radix.list_view_theme());
        $( let builder = builder.selection_mode($selection); )?
        $( let builder = builder.selected_index($selected_index); )?
        $( let builder = builder.active_index($active_index); )?
        $( let builder = builder.row_label(move |$label_row: &_| $label_body); )?
        $( let builder = builder.row_enabled(move |$enabled_row: &_| $enabled_body); )?
        let builder = builder.grid_view(vec![$($col),*]);
        let builder = builder.row_template(std::sync::Arc::new(move |$model, $cells, $win, $cx| {
            $body
        }));
        $( let builder = builder.visible_rows($visible_rows); )?
        $( let builder = builder.scroll_snap($scroll_snap); )?
        builder
    }};

    // grid_view only
    (
        radix = $radix:expr;
        id = $id:expr;
        items = $items:expr;
        $( selection = $selection:expr; )?
        $( selected_index = $selected_index:expr; )?
        $( active_index = $active_index:expr; )?
        $( row_label = |$label_row:ident| $label_body:expr; )?
        $( row_enabled = |$enabled_row:ident| $enabled_body:expr; )?
        $( visible_rows = $visible_rows:expr; )?
        $( scroll_snap = $scroll_snap:expr; )?
        grid_view = { $($col:expr),* $(,)? } $(;)?
    ) => {{
        let builder = $crate::controls::list_view::new_typed($id).items($items);
        let builder = builder.theme($radix.list_view_theme());
        $( let builder = builder.selection_mode($selection); )?
        $( let builder = builder.selected_index($selected_index); )?
        $( let builder = builder.active_index($active_index); )?
        $( let builder = builder.row_label(move |$label_row: &_| $label_body); )?
        $( let builder = builder.row_enabled(move |$enabled_row: &_| $enabled_body); )?
        let builder = builder.grid_view(vec![$($col),*]);
        $( let builder = builder.visible_rows($visible_rows); )?
        $( let builder = builder.scroll_snap($scroll_snap); )?
        builder
    }};
}

#[macro_export]
macro_rules! paging_list_view {
    // grid_view + row_template
    (
        radix = $radix:expr;
        id = $id:expr;
        items = $items:expr;
        page_size = $page_size:expr;
        $( selection = $selection:expr; )?
        $( selected_index = $selected_index:expr; )?
        $( active_index = $active_index:expr; )?
        $( row_label = |$label_row:ident| $label_body:expr; )?
        $( row_enabled = |$enabled_row:ident| $enabled_body:expr; )?
        $( visible_rows = $visible_rows:expr; )?
        $( toolbar_template = |$layout:ident, $win:ident, $cx:ident| $toolbar_body:expr; )?
        grid_view = { $($col:expr),* $(,)? };
        row_template = |$model:ident, $cells:ident, $rwin:ident, $rcx:ident| $body:expr $(;)?
    ) => {{
        let builder = $crate::controls::list_view::new_typed($id).items($items);
        let builder = builder.theme($radix.list_view_theme());
        $( let builder = builder.selection_mode($selection); )?
        $( let builder = builder.selected_index($selected_index); )?
        $( let builder = builder.active_index($active_index); )?
        $( let builder = builder.row_label(move |$label_row: &_| $label_body); )?
        $( let builder = builder.row_enabled(move |$enabled_row: &_| $enabled_body); )?
        let builder = builder.grid_view(vec![$($col),*]);
        let builder = builder.row_template(std::sync::Arc::new(move |$model, $cells, $rwin, $rcx| {
            $body
        }));
        $( let builder = builder.visible_rows($visible_rows); )?
        
        let paging_builder = $crate::controls::list_view::PagingListViewBuilder::new(
            builder.paged($page_size),
            $radix.clone(),
        );
        
        $(
            let paging_builder = paging_builder.toolbar_template(std::sync::Arc::new(
                move |$layout, $win, $cx| { $toolbar_body }
            ));
        )?
        
        paging_builder
    }};

    // grid_view only
    (
        radix = $radix:expr;
        id = $id:expr;
        items = $items:expr;
        page_size = $page_size:expr;
        $( selection = $selection:expr; )?
        $( selected_index = $selected_index:expr; )?
        $( active_index = $active_index:expr; )?
        $( row_label = |$label_row:ident| $label_body:expr; )?
        $( row_enabled = |$enabled_row:ident| $enabled_body:expr; )?
        $( visible_rows = $visible_rows:expr; )?
        $( toolbar_template = |$layout:ident, $win:ident, $cx:ident| $toolbar_body:expr; )?
        grid_view = { $($col:expr),* $(,)? } $(;)?
    ) => {{
        let builder = $crate::controls::list_view::new_typed($id).items($items);
        let builder = builder.theme($radix.list_view_theme());
        $( let builder = builder.selection_mode($selection); )?
        $( let builder = builder.selected_index($selected_index); )?
        $( let builder = builder.active_index($active_index); )?
        $( let builder = builder.row_label(move |$label_row: &_| $label_body); )?
        $( let builder = builder.row_enabled(move |$enabled_row: &_| $enabled_body); )?
        let builder = builder.grid_view(vec![$($col),*]);
        $( let builder = builder.visible_rows($visible_rows); )?
        
        let paging_builder = $crate::controls::list_view::PagingListViewBuilder::new(
            builder.paged($page_size),
            $radix.clone(),
        );
        
        $(
            let paging_builder = paging_builder.toolbar_template(std::sync::Arc::new(
                move |$layout, $win, $cx| { $toolbar_body }
            ));
        )?
        
        paging_builder
    }};
}
```

---

## 5. Exposing in SDK (`mod.rs`)

We modify `crates/sdk/src/controls/list_view/mod.rs` to expose the new types:

```diff
mod control;
mod column_template;
mod layout;
mod macros;
mod model;
mod row;
mod template;
mod theme;
+mod toolbar;
+mod paging;

pub use control::{ListViewControl, ListViewEvent};
+pub use paging::{PagingListView, PagingListViewControl, PagingListViewBuilder};
+pub use toolbar::{PagingToolbar, PagingToolbarEvent, PagingToolbarLayout, PagingToolbarTemplate};

// ScrollingListView is a type alias to the base list view entity
+pub type ScrollingListView<T> = Entity<ListViewControl<T>>;
+pub type ScrollingListViewBuilder<T> = ListViewBuilder<T>;
```

---

## 6. Gallery Pane Re-wiring

Gallery list view demos live in two panes under `apps/gallery/src/gallery/panes/list_view/`:

- **`scrolling_list_view_pane.rs`** — `task_list_builder(...).visible_rows(N).spawn(cx)`
- **`paging_list_view_pane.rs`** — `PagingListViewBuilder::new(task_list_builder(...).paged(N), theme).spawn(cx)`

Shared task grid/data is in **`shared.rs`**. The paging pane renders `self.list_view.clone()` directly; the facade owns list + toolbar sync (no gallery-side toolbar entity or manual page-state subscriptions).

Phase 2 will optionally migrate these to `scrolling_list_view!` / `paging_list_view!` macros.
