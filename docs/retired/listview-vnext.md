# ListView — target usage and implementation notes

**Code map:** `crates/sdk/src/controls/list_view/*` (see `docs/ai/module-map.md`).

This document is the **source of truth** for the agreed **gallery-style grid list** API. The full `list_view!` example below is what we implement toward; comments at the end describe what exists in the repo today and what must change.

Obsolete drafts live under `docs/retired/` and `docs/out-of-date-do-not-read/`.

---

## Target example (complete `list_view!`)

One macro invocation: **columns** for all cell UI (including a real checkbox in column 0), optional **`row_template`** for row padding and row background only. No separate “chrome” concept for rows — use **row**, **row template**, and **`ListViewRowLook`** from theme.

```rust
use std::sync::Arc;

use gpui::{
    AnyElement, Context, FontWeight, Hsla, MouseButton, SharedString, Subscription, Window, div, prelude::*, px,
};
use gpui_luma::controls::checkbox::Checkbox;
use gpui_luma::controls::command::button::ButtonEvent;
use gpui_luma::controls::icon::lucide_glyph;
use gpui_luma::controls::list_view::{
    ListSelectionMode, ListViewColumn, ListViewEvent,
    column_template_with_modifier, default_text_column_template,
};
use gpui_luma::theme::radix::prelude::*;
use gpui_luma::theme::RadixTheme;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

const TASK_COUNT: usize = 100;

#[derive(Clone)]
struct Task {
    id: SharedString,
    title: SharedString,
    email: SharedString,
    tag: &'static str,
    status: &'static str,
    enabled: bool,
}

/// Row model: task data + one spawned checkbox per row (column 0).
#[derive(Clone)]
struct TaskRow {
    task: Task,
    checkbox: Checkbox,
}

#[derive(Clone)]
pub(in crate::gallery) struct ListViewPane {
    list: gpui_luma::controls::list_view::ListView<TaskRow>,
    selected_indices: Vec<usize>,
}

impl ListViewPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, radix_theme: Arc<RadixTheme>) -> Self {
        let tasks = build_task_rows(cx, &radix_theme);

        let list = gpui_luma::list_view! {
            radix = radix_theme;
            id = "listview-tasks";
            items = tasks;
            selection = ListSelectionMode::Single;
            selected_index = 1;
            active_index = 1;
            row_label = |row| row.task.title.clone();
            row_enabled = |row| row.task.enabled;

            grid_view = {
                column!("", width = 44 => |row| {
                    div()
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .on_click(|_, _, cx| cx.stop_propagation())
                        .w_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(row.checkbox.clone())
                }),
                column_emphasis!("Task", width = 108 => |row| row.task.id.clone()),
                column!("Title" => |row| {
                    div()
                        .w_full()
                        .min_w(px(0.0))
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(tag_pill(row.task.tag))
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(0.0))
                                .truncate()
                                .child(row.task.title.clone()),
                        )
                }),
                column!("Status", width = 132 => |row| status_cell(row.task.status)),
                email_column(),
                column!("", width = 44 => |row| {
                    div()
                        .w_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(lucide_glyph(LucideIcon::EllipsisVertical))
                }),
            };

            row_template = |model, cells, _window, _cx| {
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .min_h(px(model.look.min_height))
                    .py(px(10.0))
                    .bg(model.look.background)
                    .text_color(model.look.label_color)
                    .text_size(px(model.look.label_typography.size))
                    .line_height(px(model.look.label_typography.line_height))
                    .font_weight(model.look.label_typography.weight)
                    .child(cells)
            };
        }
        .spawn(cx);

        Self {
            list,
            selected_indices: vec![1],
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.list, |app, _, event: &ListViewEvent, cx| {
            app.panes.list_view.handle_event(event, cx);
        }));
    }

    fn handle_event(&mut self, event: &ListViewEvent, cx: &mut Context<GalleryApp>) {
        if let ListViewEvent::SelectionChanged { selected_indices } = event {
            self.selected_indices = selected_indices.clone();
            cx.notify();
        }
    }
}

/// Built-in **text** column + modifier (WPF-style binding column with app tweak).
fn email_column() -> ListViewColumn<TaskRow> {
    ListViewColumn::fixed(
        "Email",
        200.0,
        column_template_with_modifier(
            default_text_column_template(|row: &TaskRow| row.task.email.clone()),
            |cell, model, _window, _cx| {
                if model.selected {
                    div().font_weight(FontWeight::SEMIBOLD).child(cell)
                } else {
                    cell
                }
            },
        ),
    )
}

fn build_task_rows(cx: &mut Context<GalleryApp>, radix_theme: &Arc<RadixTheme>) -> Vec<TaskRow> {
    (0..TASK_COUNT)
        .map(|index| {
            let task = make_task(index);
            let checkbox = radix_theme
                .ghost_checkbox(format!("listview-task-checkbox-{index}"))
                .with_data(index % 3 == 0)
                .tab_stop(true)
                .spawn(cx);
            TaskRow { task, checkbox }
        })
        .collect()
}

fn make_task(index: usize) -> Task {
    const TAGS: &[&str] = &["UI", "API", "Docs", "Bug"];
    const STATUSES: &[&str] = &["Open", "In progress", "Done", "Blocked"];

    let tag = TAGS[index % TAGS.len()];
    let status = STATUSES[index % STATUSES.len()];

    Task {
        id: format!("T-{index:04}").into(),
        title: format!("Ship list view row {index}").into(),
        email: format!("owner+{index}@luma.dev").into(),
        tag,
        status,
        enabled: index % 17 != 0,
    }
}

fn tag_pill(tag: &'static str) -> impl IntoElement {
    div()
        .px(px(6.0))
        .py(px(2.0))
        .rounded(px(4.0))
        .bg(gpui::hsla(0.12, 0.55, 0.92, 0.18))
        .text_size(px(11.0))
        .line_height(px(14.0))
        .font_weight(FontWeight::MEDIUM)
        .child(tag)
}

fn status_cell(status: &'static str) -> impl IntoElement {
    let (icon, color) = match status {
        "Done" => (LucideIcon::CircleCheck, gpui::hsla(0.35, 0.7, 0.45, 1.0)),
        "Blocked" => (LucideIcon::CircleX, gpui::hsla(0.0, 0.7, 0.55, 1.0)),
        "In progress" => (LucideIcon::LoaderCircle, gpui::hsla(0.58, 0.75, 0.5, 1.0)),
        _ => (LucideIcon::Circle, gpui::hsla(0.0, 0.0, 0.55, 1.0)),
    };

    div()
        .w_full()
        .min_w(px(0.0))
        .flex()
        .items_center()
        .gap(px(6.0))
        .child(lucide_glyph(icon).text_color(color))
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .truncate()
                .child(status),
        )
}
```

### Macro clauses (what each field does)

| Clause | Responsibility |
|--------|----------------|
| `radix = radix_theme` | Applies `list_view_theme()` on the builder |
| `id`, `items` | Control id and row backing data |
| `selection`, `selected_index`, `active_index` | Selection mode and initial state |
| `row_label`, `row_enabled` | Accessibility / keyboard labeling and disabled rows |
| `grid_view = { … }` | Column strip: `column!` (fully custom), `column_text!` / `column_emphasis!` / … (built-ins), or `ListViewColumn` values (e.g. `email_column()`) |
| `row_template = \|model, cells, …\|` | **Row wrapper only** — use `model.look` (resolved row theme); `cells` is the column strip from `grid_view` |

### Design rules (from this example)

1. **Columns are the main customization point** — use a **built-in column template** for plain bindings; use `column!` when the cell is custom (checkbox, composite title, status icon row, menu).
2. **Prefer built-ins for ~80% of columns** — Task id and Email should not hand-roll `div().truncate().child(...)`; that duplicates theme and slot layout.
3. **`row_template` is optional** — row padding and background only, not column layout.
4. **Interactive controls in cells** — `column!` closures are `|row| → element`; spawn entities on the row model (`TaskRow.checkbox`), then `.child` them in the column. Stop pointer propagation on the cell wrapper so row selection does not fire when toggling the checkbox (see **Agreed technical decisions**).
5. **ListView is not a spreadsheet** — no in-cell editors, range select, or column resize in this tranche (see deferred work below).

---

## Agreed technical decisions

### 1. `ListViewRowRenderModel` includes resolved row look (required)

The control resolves `ListViewRowLook` from `ListViewTheme::resolve_row` when building each row and sets **`model.look`** on `ListViewRowRenderModel` before calling `row_template`. Users must not capture `list_theme` or call `resolve_row` manually.

```rust
// control.rs (sketch)
ListViewRowRenderModel {
    // ...
    look: self.model.theme.resolve_row(selected, interaction, self.model.size),
}
```

`row_template` then reads `model.look.background`, `min_height`, typography, etc. This is **tranche 1**, not a follow-up.

### 2. Interactive columns stop propagation to the row

GPUI delivers mouse events to the deepest hit target first. A checkbox in column 0 should toggle without also driving row `on_click` / selection. Wrap interactive cell content and call **`cx.stop_propagation()`** on `on_mouse_down` and `on_click` (see checkbox column in the target example). Implementations should document this pattern for any column with buttons, menus, or checkboxes.

### 3. `grid_view` macro: heterogeneous columns via `expr`

Do **not** re-parse `column!` / `column_text!` token trees inside `list_view!`. Match columns as expressions and build a `Vec<ListViewColumn<T>>`:

```rust
// list_view! grid_view arm (target)
grid_view = {
    $($col:expr),* $(,)?
};
// expands to, conceptually:
let columns: Vec<ListViewColumn<_>> = vec![$($col),*];
builder.grid_view(columns)
```

Each of `column!(…)`, `column_text!(…)`, and `email_column()` must type-check as `ListViewColumn<T>`. `column!` / `column_text!` remain **standalone** macros that produce a column value; `list_view!` only aggregates them.

---

## Built-in column templates

Yes — we should ship **more than one** built-in. Most grid-style lists repeat the same few cell shapes; only a minority of columns need full `column!` bodies.

| Built-in | Macro (target) | Typical use | Typography / layout |
|----------|----------------|-------------|---------------------|
| **Text** | `column_text!("Email", width = 200 => \|row\| row.task.email)` | Default binding column | Row `label_color`, truncate, theme slot padding |
| **Muted text** | `column_muted!(…)` | Secondary fields, timestamps, metadata | Muted foreground from list theme |
| **Emphasis text** | `column_emphasis!("Task", width = 108 => \|row\| row.task.id)` | Id, name, primary key column | Semibold (or header-weight token) |
| **Numeric** | `column_numeric!(…)` | Counts, amounts | Right-aligned tabular figures (when token/metrics exist) |

**Custom** `column!(…)` stays for everything else: checkbox column, fill title with tag pill, status icon + label, trailing actions.

### Modifiers (SelectionPanel-style)

Built-ins render through a shared pipeline (`column_template.rs`):

- `default_text_column_template(|row| value)` → `ListViewColumnCellTemplate<T>`
- `column_template_with_modifier(base, |cell, model, window, cx| …)` — wrap or replace the cell element using `ListViewColumnRenderModel` (row index, selected, hovered, etc.)

The example uses **text + modifier** for Email (`email_column()`). Task uses **emphasis** without a modifier.

Standalone preview (gallery / tests): same templates accept `ListViewColumnRenderModel` + `window` + `cx` so a cell can be rendered outside the list.

### Macro `grid_view` column forms (target)

```rust
grid_view = {
    column!(…),           // -> ListViewColumn<T>
    column_text!(…),      // -> ListViewColumn<T>
    column_emphasis!(…),  // -> ListViewColumn<T>
    email_column(),       // -> ListViewColumn<T>
};
```

Parsed as `$($col:expr),*` (see **Agreed technical decisions §3**).

---

## What compiles today (repo macro, May 2026)

The example above is the **target**. Current `list_view!` in `crates/sdk/src/controls/list_view/macros.rs` differs:

```rust
// TODAY — grid arm only; no row_template in the same invocation
let list = gpui_luma::list_view! {
    radix = radix_theme;
    id = "listview-tasks";
    items = tasks;
    selection = ListSelectionMode::Single;
    selected_index = 1;
    active_index = 1;
    row_label = |row| row.task.title.clone();
    row_enabled = |row| row.task.enabled;
    grid_view = {
        column!("", width = 44 => |row| {
            div()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_click(|_, _, cx| cx.stop_propagation())
                .w_full()
                .flex()
                .items_center()
                .justify_center()
                .child(row.checkbox.clone())
        }),
        column!("Task", width = 108 => |row| row.task.id.clone()),
        column!("Status", width = 132 => |row| status_cell(row.task.status)),
        column!("Email", width = 200 => |row| row.task.email.clone()),
        column!("", width = 44 => |row| {
            div()
                .w_full()
                .flex()
                .items_center()
                .justify_center()
                .child(lucide_glyph(LucideIcon::EllipsisVertical))
        }),
    };
}
.spawn(cx);
```

| Gap | Today | Target |
|-----|--------|--------|
| `grid_view` + `row_template` together | Two separate macro arms | Single arm |
| `row_template` signature | `\|model, window, cx\|` with `row` = `model.row` (replaces entire row body) | `\|model, cells, window, cx\|` — wrap SDK-built column strip |
| Fill column in `grid_view` arm | `column!` supports fill; `list_view!` grid arm only accepts `width = …` | Accept `column!("Title" => \|row\| …)` |
| Built-in column templates | None — every column is a raw `column!` closure | `column_text!`, `column_muted!`, `column_emphasis!`, `column_numeric!`, `column_template_with_modifier` |
| `ListViewColumn` expressions in macro | Not supported | `grid_view = { $($col:expr),* }` → `vec![…]` + `.grid_view(columns)` |
| `model.look` on row template | Not on `ListViewRowRenderModel` | Control resolves `ListViewRowLook` per row before `row_template` |
| Gallery | `apps/gallery/.../list_view/pane.rs` — 10k `DemoUser` text columns | Replace with tasks example above |

---

## Implementation notes

### Layering (current code)

| Layer | File | Role |
|-------|------|------|
| Control | `control.rs` | Virtualization, selection, pointer, keyboard; calls row template then `render_list_view_row_chrome` |
| Theme | `theme.rs`, `theme/radix/list_view.rs` | `resolve_look`, `resolve_row` → `ListViewLook`, `ListViewRowLook` |
| Shell template | `template.rs` | List border, header slot, body slot |
| Row wrapper | `row_chrome.rs` | Default row padding, divider, adorner, min-height (internal name; public docs say **row**) |
| Columns | `model.rs` | `ListViewColumn`, `render_grid_view_cells`, fixed 12px `px` + `.truncate()` per slot |
| Column templates | `column_template.rs` (new) | Built-in text / muted / emphasis / numeric; `ListViewColumnRenderModel`; modifiers |
| Macros | `macros.rs` | `column!`, `column_text!`, …, `list_view!` |

### Required changes to match the target example

1. **Column templates (`column_template.rs`, export from `mod.rs`)**
   - `ListViewColumnRenderModel<'a, T>` — per-cell context (row ref, index, selected, hovered, enabled, list id).
   - `default_text_column_template`, `default_muted_column_template`, `default_emphasis_column_template`, `default_numeric_column_template` — each returns a cell renderer used by `ListViewColumn::fixed` / `fill`.
   - `column_template_with_modifier` — same pattern as `selection_panel::item_template_with_modifier`.
   - Built-ins apply typography and truncation inside the column slot (eventually share slot layout with `render_grid_view_column_slot`).

2. **`ListViewRowRenderModel` + `control.rs` (required, first)**
   - Add `look: ListViewRowLook` to `ListViewRowRenderModel`.
   - In `render_row`, resolve row look once (same `selected` / interaction inputs as today) and pass on the model before invoking `row_template`.

3. **Macro (`macros.rs`)**
   - `column_text!`, `column_muted!`, `column_emphasis!`, `column_numeric!` — each expands to a `ListViewColumn<T>` expression (not parsed inside `list_view!`).
   - Merge arms so `grid_view` and `row_template` can appear in one `list_view!`.
   - `grid_view = { $($col:expr),* $(,)? }` → `vec![$($col),*]` then `.grid_view(columns)`.
   - Change `row_template` binding to inject `cells`: SDK builds the column strip, user closure wraps it.

4. **Composition (`control.rs`, `model.rs`)**
   - When `grid_view` is set and `row_template` is custom: `cells = render_grid_view_cells(…)`; user `row_template(model, cells, …)` returns row body.
   - When only `grid_view`: internal row template = render cells only (unchanged behavior).
   - When only `row_template`: plain-list behavior (single label or full custom body).
   - Custom `row_template` still runs inside `render_list_view_row` for divider / adorner / default pointer handlers unless documented opt-out is added later.

5. **Column slots (`model.rs`)**
   - Consider per-column flags for padding / truncate (today hard-coded in `render_grid_view_column_slot`).

6. **Gallery**
   - Implement `ListViewPane` from the target example once the above lands.
   - Subscribe per-row checkbox `ButtonEvent` in the pane (not shown in the example block; add when wiring gallery).
   - Checkbox column uses `stop_propagation` on the wrapper `div` (required pattern, not best-effort).

### Rename (internal, non-blocking)

- `row_chrome.rs` → `row.rs`
- `render_list_view_row_chrome` → `render_list_view_row`

### Already shipped (v1 foundation)

- `ListView` / `ListViewControl` on GPUI `list` + `ListState`
- `grid_view` / `.grid_columns().column_fixed(…).finish()` builder path
- `column!` macro (fixed + fill)
- `row_template` / `with_row_template` for **full** row replacement (plain list or grid-only internal use)
- `row_label`, `row_enabled`, `ListSelectionMode`, Radix theme via `radix =`
- `look_override`, `square_corners()` on builder
- Tests: selection/active normalization in `control.rs`

---

## Deferred (not in this tranche)

| Feature | Notes |
|---------|--------|
| Column resize, horizontal scroll, shared width state | List-table polish; header + body must share widths |
| Delegate / paging backing store | `Vec<T>` only today |
| Data grid / spreadsheet | Separate control — in-cell edit, range select, sticky header as grid features |
| Column `|row, window, cx|` on `column!` only | Optional; built-ins + modifiers use `ListViewColumnRenderModel`; entity-per-row still uses custom `column!` |

---

## Success criteria for this tranche

1. `ListViewRowRenderModel::look` populated in `render_row`; target `row_template` compiles without captured theme.
2. `list_view!` uses `grid_view = { $($col:expr),* }`; example compiles with `column!`, built-in column macros, and `email_column()`.
3. Gallery tasks pane: built-in column + modified column + checkbox column with propagation stopped on row handlers.
4. Manual or automated check: toggling checkbox does not change row selection.

---

## Related docs

- `docs/ai/module-map.md` — module listing
- `docs/ai/radix-shadcn.md` — ListView tokens (row hover uses `--muted`, not `--accent`)
