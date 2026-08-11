# Issue: Upgrade panel plan `RadioGroup` — focus regression & over-engineering

**Panel:** `apps/luma-studio/src/studio/panels/upgrade.rs`  
**Related:** `docs/ai/issues/0-theme-reconcile.md` (Cards / tweakcn parity)  
**Status:** Visual parity achieved; keyboard focus / roving tabindex broken. Fix not yet applied.

---

## Summary

The Upgrade panel “Plan” section needs **side-by-side card-style radio options** (Starter / Pro) matching tweakcn: bordered tiles, radio + title + description, selected tile gets a **subtle surface adjustment** (darker in light mode, lighter in dark mode), **no focus ring on the radio dot**.

Current code achieves the **paint** but breaks **composite control focus** by replacing the radio-group item surface with a custom card wrapper and forcing `focused: false` on the embedded indicator.

---

## What tweakcn wants (behavior + paint)

| Aspect | Expected |
|--------|----------|
| Control | Single-select radio group (one plan always selected) |
| Layout | Two equal-width cards, horizontal |
| Unselected | Border only; no fill (inherits parent card surface) |
| Selected | Fill = slight adjustment of card surface (not a separate token like raw `muted`) |
| Radio | Standard primary radio indicator; **no outer focus ring** on the dot |
| Keyboard | Arrow keys move active item; Space selects; focus stays on **one group** (roving active descendant) |
| Text | Title + description; description wraps inside card |

---

## What we implemented (current `upgrade.rs`)

1. **`horizontal_radio_group("upgrade-plan")`** — still a `ControlGroupControl` with `single_required()` semantics.
2. **Full custom `RadioGroupTemplate`** (`plan_option_group_template`) — manually iterates items and wires `item_hovers` / `item_clicks` onto a **card `div`**, not onto the SDK’s standard item row or radio button.
3. **Detached indicator** — `ButtonFamilyRole::Icon`, empty content, `compact`, `elevation: false`, and **`InteractionState { focused: false, ... }`** while still passing hover/pressed from the item.
4. **SDK changes** to support embedding:
   - `ThemedRadioButtonTemplate`: skip adorner / oversize for `Icon` role.
   - `ShadcnLook::adjust_surface_color` for mode-aware surface shifts.
5. **Selected card bg** — resolved each render: `adjust_surface_color(card, Pressed)`; unselected: no `.bg()`.

**Visual result:** Correct.  
**Focus result:** Broken (active descendant / keyboard navigation no longer aligned with painted affordances).

---

## How SDK radio groups are supposed to work

`RadioGroup` is a type alias for `ControlGroupControl` with `ControlSelectionMode::SingleRequired` (`crates/sdk/src/controls/radio_group/mod.rs`).

### Focus model (one handle, roving active item)

```text
ControlGroupControl
  └─ focus_handle (single tab stop on the group)
  └─ active_id     (which item is the active descendant when group is focused)
```

On render (`control_group/control.rs`):

- `CompositeItemState.active` = group focused **and** this item is `active_id`.
- `CompositeItemState.focus_visible` = keyboard modality **and** active.
- `CompositeItemState::interaction_state()` maps **`focused` → `active`** (not `focus_visible`):

```35:37:crates/sdk/src/controls/state.rs
    pub fn interaction_state(self) -> InteractionState {
        InteractionState { hovered: self.hovered, pressed: self.pressed, focused: self.active, disabled: self.disabled }
    }
```

So the **radio `ButtonRenderModel` must receive `item.state.interaction_state()` unchanged** if the indicator should reflect “this item is the focused descendant.” Clearing `focused` severs that link.

### Two template layers (do not conflate)

| Layer | Type | Responsibility |
|-------|------|----------------|
| **Group template** | `ControlGroupTemplate` | Outer layout (horizontal row, gaps). Calls either `render_radio_button_rows` **or** `render_control_group_items`. |
| **Item template** | `ControlGroupItemTemplate` | Per-item **content only** (optional). Used by `render_control_group_items`, **not** by `render_radio_button_rows`. |

**Default radio path** (`look.radio_group()` / `radio_group_buttons_template`):

- Group template → `render_radio_button_rows`
- Each row is a **full radio button** (indicator + label); handlers attach **to the button root**.
- `ButtonRenderModel.state` = `item.state.interaction_state()`.

**Themed control-group path** (`vertical_group_template` / `horizontal_group_template`):

- Group template → `ThemedControlGroupTemplate` → `render_control_group_items`
- Handlers attach to a **wrapper `div`** around item content.
- Supports **`ControlGroupBuilder::with_item_template`** / `button_item_template`.

**Upgrade panel today:** custom group template that **neither** calls `render_radio_button_rows` **nor** `render_control_group_items` — it re-implements handler wiring on a card shell and renders a neutered indicator beside copy.

### Modifiers (what they are for)

| Modifier | Applies to | Example use |
|----------|------------|-------------|
| `ThemedRadioButtonTemplate::with_modifier` | Radio **button root** after build | Zero padding for embedded indicator; optional transparent focus treatment |
| `ControlGroupBuilder::with_template_modifier` | Group chrome container | Row gap, full width |
| `ControlTemplate::with_modifier` on group theme | List chrome (`ControlGroupChromeModel`) | Border/background on group shell |

Modifiers adjust an **already-rendered** control element. They do **not** replace group/item structure or focus wiring.

**Disabling focus visual on the radio dot** should mean: keep real `InteractionState` (especially `active` → `focused`), but make the **adorner transparent** or use `compact` so no ring is painted — **not** `focused: false` and not global adorner removal for all `Icon` radios.

---

## Why focus broke

1. **Handlers on card wrapper, indicator not the control row** — Clicks work via duplicated handlers, but the focused/active semantics are tied to how `ControlGroupControl` builds `CompositeItemState` for items whose interactive surface is the SDK row/button path.
2. **`focused: false` on `ButtonRenderModel`** — Overrides `interaction_state()` so the radio never sees `active`, even when the group has keyboard focus on that item.
3. **Custom template bypasses `render_radio_button_rows`** — That function is where the SDK consistently attaches hover/click/focus integration for radio groups using the default look helpers.
4. **Gallery `delivery_window_template`** uses the same “card + manual handlers” pattern but **without** radio indicators — it hides focus issues that appear when you embed a real radio.

---

## What was over-engineered vs what was actually needed

| Needed | Over-built instead |
|--------|-------------------|
| Card layout (radio + title + description) | Entire custom group template duplicating handler iterators |
| Unselected: transparent; selected: `adjust_surface_color(card, Pressed)` | Painting unselected with `card` token (earlier bug); then fixing with render-time resolution only in custom template |
| No ring on radio dot | `focused: false` + SDK changes to skip adorner for `Icon` role |
| Horizontal side-by-side tiles | `ButtonFamilyRole::Icon` split + padding modifier on radio |
| Mode-aware tint | `ShadcnLook::adjust_surface_color` (reasonable; could stay) |

**Minimal visual recipe (once focus is correct):**

- Unselected: no background on tile.
- Selected: `look.adjust_surface_color(look.color(ShadcnToken::Card), InteractionLayer::Pressed)`.
- Resolve colors **each render** (or on theme/mode change), not once at panel spawn.

---

## Recommended fix (panel + SDK, no new concepts)

### Path A — **Preferred:** stock group template + `item_template`

Use the **themed control-group** horizontal template and an **item template** that renders the card interior.

```rust
horizontal_radio_group("upgrade-plan")
    .template(horizontal_group_container_template())  // NOT radio_group_buttons_template
    .with_item_template(plan_option_item_template(look))
    .items(plan_items())
    .selected("starter")
```

`plan_option_item_template` should:

1. Receive `ControlGroupItemRenderModel<'_, PlanOptionItem>` (has `selected`, `active`, `state`, `item.title` / description).
2. Render **one row**: bordered card shell (bg when `item.selected`), containing:
   - Radio via `button_item_template` **or** inline `ButtonRenderModel` with **rich `content`** (title + description vstack).
3. Pass **`item.state.interaction_state()`** to the radio — do **not** clear `focused`.
4. Suppress dot focus ring via **`ThemedRadioButtonTemplate::with_modifier`** (transparent adorner / `compact`) — not by falsifying state.

Handlers stay on the **item wrapper** from `render_control_group_items` (standard SDK path). Group `focus_handle` + `active_id` + arrow/space handling in `ControlGroupControl` remain intact.

**Note:** `radio_group::horizontal()` only sets `layout: Horizontal` on the model; default template from `radio_group::new()` is still `vertical_group_template()`. For horizontal **chrome**, set `.template(horizontal_group_container_template())` explicitly.

### Path B — extend `render_radio_button_rows` (SDK)

Keep `radio_group_buttons_template` but allow **custom per-item content** (today content is only `item.label()`):

- Add optional content callback or wire `ControlGroupItemTemplate` into `render_radio_button_rows`.
- Card chrome could wrap the returned button; handlers remain on the button root (best for radio).

Gallery-quality card radios would use this path; less boilerplate than Path A for “radio is the row.”

### Path C — card wrapper **around** `render_radio_button_rows` output (panel-only)

Custom group template that:

1. Calls `render_radio_button_rows` with a radio template whose **content** is title+description (not a separate Icon-only radio).
2. Wraps each returned button in a card `div` for border/bg only — **do not** re-bind handlers on the wrapper; **do not** force `focused: false`.

Selected background on the wrapper; pointer events still hit the button child.

---

## SDK enhancements (optional, if we want cleaner app code)

| Enhancement | Why |
|-------------|-----|
| **`radio_group_card_item_template(look, …)`** in look-shadcn or sdk | Encodes Path A: card shell + `adjust_surface_color` + typography; apps pass items only. |
| **`render_radio_button_rows_with_content(fn)`** | Path B; avoids full custom group templates for “radio + rich label.” |
| **`ThemedRadioButtonTemplate::without_focus_ring()`** | Documented modifier/helper: transparent adorner, keep `compact`, no `Icon`-role special case in core template. |
| **`radio_group::horizontal` sets horizontal group template** | Today only flips `layout`; easy to ship wrong template (vertical chrome + horizontal layout flag). |
| **Gallery delivery window: document focus** | Same manual-handler pattern; fine for non-focusable tiles, misleading as radio reference. |

**Revert candidates** (after panel fix lands):

- Radio template: `indicator_only` adorner skip — only needed if apps falsify focus instead of using transparent adorner.
- Forced `focused: false` in upgrade panel — remove, not relocate.

Keep unless used elsewhere:

- `ShadcnLook::adjust_surface_color` — correct abstraction for tweakcn-style surface tints.
- `ButtonFamilyRole::Icon` on radio — optional for split layouts; not required if radio `content` holds title+description in one row.

---

## Files involved

| Area | Path |
|------|------|
| Broken panel | `apps/luma-studio/src/studio/panels/upgrade.rs` |
| Radio group API | `crates/sdk/src/controls/radio_group/` |
| Group focus / active | `crates/sdk/src/controls/control_group/control.rs` |
| Item render paths | `crates/sdk/src/controls/control_group/template.rs`, `crates/sdk/src/controls/radio_group/themed_template.rs` |
| Item template helper | `crates/sdk/src/controls/control_group/button_item_template.rs` |
| Composite state | `crates/sdk/src/controls/state.rs` |
| Radio template / modifiers | `crates/sdk/src/controls/radio_button/template.rs` |
| Surface adjustment | `crates/look-shadcn/src/look.rs` (`adjust_surface_color`), `state_color.rs` |
| Reference (card tiles, no radio) | `apps/gallery/src/gallery/panes/radio_group/pane.rs` (`delivery_window_template`) |
| Reference (stock radio rows) | `look.radio_group()`, `render_radio_button_rows` |

---

## Test plan (after fix)

1. **Mouse** — Click either card selects; only one selected; selected tint visible in light and dark.
2. **Keyboard** — Tab to plan group; arrow left/right moves **active** item (visible on radio/active styling if any); Space selects; focus does not escape group incorrectly.
3. **No dot focus ring** — Keyboard focus does not paint outer ring on radio indicator (card may use selected bg only).
4. **Theme toggle** — Switch light/dark in luma-studio; tints flip direction (darker vs lighter); text remains readable.
5. **Regression** — Other `RadioGroup` gallery panes and default `look.radio_group()` unchanged.

---

## Takeaway

The Upgrade panel should stay a **real `RadioGroup`** (single focus handle, roving `active_id`, `interaction_state()` driving the radio). Card chrome and surface tint are **presentation on the item template or wrapper**; suppressing focus visual is a **modifier/adorner** problem, not a reason to fork the group template and zero out `focused`.
