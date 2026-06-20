# SearchSelector refactor design

This document captures how to apply the same layered refactor performed for `combobox` to `search_selector`, with the lessons learned during that work.

## Goal

Adopt the same 3-layer template architecture as `selector`/refactored `combobox`:

1. **Control template**: trigger/chrome and top-level control composition.
2. **Panel template**: popup shell, placement, panel chrome, overflow/scroll host integration.
3. **Item template**: standalone per-item content rendering.

Control logic should own behavior/state only, not display shaping.

---

## Desired architecture for `search_selector`

### Control layer (`search_selector/control.rs`)

Owns:
- filtering/query state
- highlighted index / selected index
- keyboard + pointer behavior
- events (`Change`, `Select`, `Complete`, `Clear`)
- focus lifecycle

Does **not** own:
- popup shell visuals (bg/border/shadow/radius)
- row content shaping
- popup shell placement details (anchored/deferred semantics)

### Panel layer (`search_selector/panel_template.rs`)

Owns:
- popup placement/anchoring
- popup shell chrome and occlusion
- inclusion of sub-sections specific to search selector popup (search field region + list region + separators)
- list host integration (scroll surface output)

Consumes a `SearchSelectorPanelRenderModel` carrying state + pre-rendered list/search sections.

### Item layer (`search_selector/item_template.rs`)

Owns:
- rendering one row’s content
- receives item model with both source and visible index context
- receives selected/active/open/enabled flags

Must be standalone (`SearchSelectorItemTemplate`, `make_search_selector_item_template`).

---

## Public API targets

Builder/model/runtime should expose orthogonal template slots:

- `template(...)` (existing control template)
- `panel_template(...)` (new/first-class)
- `with_item_template(...)` (new/first-class)

Runtime setters should mirror builder options.

Keep legacy `items_template(...)` only as a transitional hook while rewiring, then deprecate/remove once complete.

---

## Render pipeline target

Final popup path should look like:

`items_template.render(...) -> panel_template.render(...) -> control_template.render(...)`

Where:
- items template renders row list content only
- panel template wraps list/search content in popup shell + placement
- control template receives already-composed popup content element

---

## State/data contract suggestions

### Item render model

`SearchSelectorItemRenderModel<'a, SelectionItem>` should include:
- `search_selector_id`
- `item`
- `source_index`
- `visible_index`
- `selected`
- `active`
- `open`
- `enabled`

### Panel render model

`SearchSelectorPanelRenderModel<'a>` should include:
- popup identity (`id`)
- source items + `visible_indices`
- selected/highlight metadata
- `open` / `enabled`
- optional item template ref
- popup geometry (`popup_bounds` or placement data)
- panel look payload (or panel-theme provider)
- pre-rendered `search_content` and `list_content` regions as needed

---

## Stepwise execution plan

1. **Step 0 — behavior freeze**
   - Create checklist for existing keyboard/mouse/focus/filter/event behavior.

2. **Step 1 — scaffolding only**
   - Add `item_template.rs` and `panel_template.rs` with compile-only defaults.

3. **Step 2 — API slots only**
   - Add builder/model/setter support for panel + item template (no behavior rewiring).

4. **Step 3 — panel routing**
   - Route popup composition through panel template while keeping behavior intact.

5. **Step 4 — remove control-side item shaping**
   - Pass source items + visible index mapping into items template.

6. **Step 5 — panel ownership completion**
   - Move popup shell chrome/placement entirely into panel template.

7. **Step 6 — cleanup**
   - Deprecate/remove transitional items-template hooks if superseded.
   - Update docs/exports/comments.

---

## Lessons learned from combobox refactor

1. **Theme ownership must match visual ownership**
   - If panel owns popup shell, panel must resolve panel look from live theme state.
   - Avoid `ThemeTokens::default()` in control render paths for themed visuals.

2. **Template preview panes must use real template pipeline**
   - Gallery/template panes should not manually mimic outdated paths.
   - If preview bypasses panel template, it hides real regressions.

3. **Index mapping is the main correctness hazard**
   - Explicitly separate `source_index` and `visible_index` everywhere.
   - Use consistent mapping for hover/click/highlight/selection states.

4. **Do not mix shell and rows in the same layer**
   - Items template should render rows/content only.
   - Panel template should render shell/placement/occlusion.

5. **Stage with review checkpoints**
   - Each step should compile and be manually testable.
   - Keep event/focus behavior untouched until structural wiring is stable.

6. **Preserve API, add before removing**
   - Introduce new slots first, keep transitional hooks during migration.
   - Remove legacy hooks only after parity is confirmed.

---

## Parity checklist (search selector specific)

- trigger click open/close behavior unchanged
- popup search field focus handoff unchanged
- filter + no-match behavior unchanged
- keyboard navigation semantics unchanged
- enter/escape behavior unchanged
- blur/close semantics unchanged
- scroll + ensure-visible behavior unchanged
- event emission order/types unchanged
- light/dark mode panel visuals match current theme mode
