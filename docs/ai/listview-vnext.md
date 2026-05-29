# ListView — v1 summary and vNext roadmap

This document captures the **current** ListView implementation, explicit **scope boundaries**, and **prioritized next steps**. It replaces the obsolete design drafts in `docs/out-of-date-do-not-read/list-control*.md`.

**Code map:** `crates/sdk/src/controls/list_view/*` (see also `docs/ai/module-map.md`).

---

## What ListView is (v1)

A **virtualized, selectable list** built on GPUI `list` / `ListState`, with optional **multi-column layout** (grid-style rows), not a spreadsheet or data grid.

| Layer | Responsibility |
|-------|----------------|
| **Control** (`control.rs`) | Selection, active row, keyboard, pointer, virtualization delegate |
| **Theme** (`theme.rs` + Radix `theme/radix/list_view.rs`) | `resolve_list` / `resolve_row` → `ListViewListAppearance` / `ListViewRowAppearance` |
| **Shell template** (`template.rs`) | Border, radius, header slot, body slot (paints from resolved `model.list` only) |
| **Row chrome** (`row_template.rs`) | Shared row wrapper: padding, divider, adorner, min-height |
| **Item content** (`item_template` / grid columns) | Per-row cell layout; default is single truncated label |

**Construction**

- Builder: `new` / `new_typed`, `.spawn(cx)`
- Declarative: `list_view!` / `column!` with optional `radix = theme`
- Radix: `RadixThemeControlExt::list_view(id)` or macro `radix =` (theme only; shell template is default paint-only)

**Gallery reference:** `apps/gallery/src/gallery/panes/list_view/pane.rs` — 10k-row virtualized grid demo, selection events.

**Tests today:** selection/active normalization unit tests in `control.rs`; no visual regression suite.

---

## What ListView is not

Do **not** evolve ListView into a WPF-style **`GridView` on steroids**. That is a **separate, later project** (working name: data grid / spreadsheet control).

| Stay in ListView vNext | Defer to data grid project |
|------------------------|----------------------------|
| Virtual scroll + roving focus | In-cell editing, clipboard |
| Optional columns (fixed/fill) | Full spreadsheet UX |
| Row / item templates | Range selection, cell focus |
| Theme-driven row chrome | Sort/filter toolbars as grid chrome |
| Modest “list table” polish | Formulas, pinned columns, etc. |

When users say “every list should work like Excel,” treat that as **pressure on a short tabular polish list**, not a license to absorb grid scope into ListView.

---

## Architecture notes (keep for vNext)

1. **Theme resolves before template** — `ListViewRenderModel::list` is filled in the control; shell template does not call `resolve_list` again.
2. **`list_view_template_with_theme`** is a compatibility alias for `default_list_view_template()`; appearance comes from `ListViewTheme` on the builder/control.
3. **Custom row today** means **`with_item_template` / `item_template`** (content inside the fixed row wrapper). There is no first-class **row chrome template** trait yet.
4. **Data model today** — `Vec<T>` in memory + `set_items`. Design docs that described a **delegate + row cache** were aspirational; not implemented.
5. **Radix / shadcn** — adequate for bordered lists; weak for rich table semantics (zebra, sticky header, column chrome). Expect more appearance fields or per-app overrides before catalog tokens alone suffice. Row hover uses `--muted`, not `--accent` (see `docs/ai/radix-shadcn.md`).

**GPUI clipping:** `overflow_hidden` is rectangular. Header corners need `rounded_tl` / `rounded_tr` on the same node as `header_background` (handled in `DefaultListViewShellTemplate`).

---

## vNext workstreams (priority order)

### 1. Specialized row templating (near term)

**Goal:** Prove the public API with a gallery example (custom row content: badges, two-line text, trailing actions).

**Decide during implementation:**

- Is `ListViewItemRenderModel` + `with_item_template` enough?
- Or do we need a **`ListViewRowTemplate`** hook (chrome + content) without becoming a grid?

Do this **before** column resize so row height / measurement contracts are stable.

---

### 2. List-table polish (tabular features — priority list)

User-visible improvements that belong in **ListView**, not the future grid. Suggested order:

| Priority | Feature | Notes |
|----------|---------|--------|
| **P0** | **Column resize** (drag header dividers, min widths) | Top user expectation for grid-style lists |
| **P1** | **Horizontal overflow** when Σ column width > viewport | Required for resize to feel correct |
| **P1** | **Shared width state** (header + body columns stay aligned) | Same layout model for header and rows |
| **P2** | **Double-click auto-fit column** | Cheap follow-up after resize |
| **P2** | **Optional column hide / reorder** | Only if cheap; otherwise grid project |
| **P3** | **Click-to-sort whole list** (row order, not cell grid) | Keep semantics “list sort,” not spreadsheet |
| **Defer** | Cell editors, range select, sticky header as grid feature | Data grid project |

Document this block as **“list table mode”** so it is not confused with the future data grid.

---

### 3. Huge-scale data sources (structural)

**Goal:** Backing store that does not require materializing `Vec<T>` for every row.

**Direction (from retired design docs, scoped down):**

- **Delegate / provider:** `count()`, `item_at(index)` or `page_at(n)`, optional `label` / `enabled` without storing all `T`
- **Invalidation** when filter, sort, or page changes
- **Row height:** stable default height first; variable height + cache later if needed
- Integrate with `ListState` remeasure on data/theme/size changes

Prerequisite for paging mode and for real “10k+ without cloning the world” apps.

---

### 4. Paging vs scrolling navigation (TBD — spec later)

Two **navigation models**, not just visual tweaks:

| Mode | Behavior (sketch) |
|------|-------------------|
| **Scroll** (current) | Continuous virtual list; scrollbar; arrow / Page Up-Down move active row and scroll viewport |
| **Page** (future) | Discrete pages (fixed page size or cursor); prev/next/first/last; may still virtualize *within* a page |

**Tool / helper (to be designed):** shared builder flag + optional footer/controls + keybinding profile so apps do not fork `control.rs` behavior.

**Open questions (fill in when spec’ing):**

- Page size vs viewport-filled pages
- Whether selection is page-local or global across pages
- How Page Down / Page Up interact with `ListState` vs page index
- Relationship to delegate `fetch_page(n)`

Plan API as something like `ListViewNavigationMode::Scroll | Page { ... }` early so scroll and page paths do not accrete as special cases.

---

## Radix / theming (later, not blocking vNext code)

- Catalog keys for list shell, header strip, row states, dividers may need expansion.
- Retro / tweakcn themes will keep exposing “accent is not row hover” class of bugs — test gallery with non-default CSS.
- Per-control overrides (`list_appearance_override`, `square_corners`) remain valid escape hatches.

---

## Doc hygiene

| Action | Status |
|--------|--------|
| Remove / archive `docs/ai/list-control.md`, `list-control-refine.md` | Moved to `docs/out-of-date-do-not-read/` |
| **This file** | Source of truth for ListView roadmap |
| `docs/ai/module-map.md` | Module listing; link here for vNext |
| `docs/ai/radix-shadcn.md` | Token mapping for ListView |

---

## Quick reference — current public API

```rust
// Radix-themed builder
radix_theme.list_view("id")
    .items(...)
    .grid_columns()
    .column_fixed("Name", 180.0, |row| row.name.clone())
    .finish()
    .spawn(cx);

// Declarative macro
list_view! {
    radix = radix_theme;
    id = "id";
    items = ...;
    grid_view = { column!("Col", width = 120 => |t| ...) };
}
.square_corners()  // optional list_appearance_override
.spawn(cx);

// Custom content (row wrapper still from SDK)
.with_item_template(|model, window, cx| { ... })
```

---

## Success criteria for “vNext tranche 1”

1. Gallery demonstrates a **non-default item/row template** without forking the control.
2. **Column resize + horizontal scroll** shipped for grid-style lists.
3. **Delegate-backed** prototype (even if only `count` + `fetch` by index) replaces “must hold `Vec<T>`” for one gallery or test scenario.
4. **Paging vs scroll** written up in this doc (new section) once product rules are defined; helper API sketched.

Until then, v1 ListView remains the right foundation: virtualized list with optional columns, not a spreadsheet.
