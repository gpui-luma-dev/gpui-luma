# `choice_group` Review & TODO

## Summary

`ChoiceGroup` is a well-architected general-purpose container for groups of button controls (toggles, radio buttons, icon toolbar buttons, etc.). The core design — a single control that covers single/multi selection, managed/unmanaged state, templatable rendering, and a composable builder — is sound and clearly superior to the parallel `toggle_group` / `radio_group` implementations it is meant to replace.

---

## Strengths

### Managed/Unmanaged State Duality
The `default_selected_ids` / `managed_selected_ids` split is clean and well-considered. The fallback-to-default behavior in `effective_selected_ids()` means consumers can opt into external state management without breaking the simple case. The `normalize_model` / `normalize_selected_ids` functions enforce consistency robustly and are well unit-tested.

### Composable Builder
The builder API is excellent. `with_modifier`, `style_preset`, `toolbar_icons`, `radio_group`, etc. layer nicely. The `template_factory` and `item_button_template_factory` escape hatches are well-thought-out. Defaults are sensible.

### Template/Content Separation
The two-tier content model — a `ChoiceGroupContent` closure for generic layouts, and `ChoiceGroupItemButtonTemplate` for fully lookless button rendering — gives the right range of customization without forcing the consumer to re-implement the event wiring.

### Strong Test Coverage
The unit tests in `control.rs` cover normalization, active-id resolution, disabled-item navigation, and selection math for both single and multiple modes.

### Position Encoding
`ChoiceGroupItemPosition` (Only/First/Middle/Last) is propagated correctly into both the visual layer and the content model, allowing templates to handle border-radius joining and dividers correctly.

---

## Issues & TODOs

### 🔴 Critical

#### 1. Redundant outer `div()` wrapper in `Render`

`control.rs` lines 399–412: the template already returns a `Stateful<Div>` as the real layout root. Wrapping it in an extra anonymous `div()` adds a needless layout node that can interfere with flex-child sizing in parent layouts.

```rust
// current — has an extra wrapper
div()
    .child(
        self.model.template.render(...)
            .track_focus(...)
            .key_context(...)
            .on_action(...)
    )
    .into_any_element()
```

**Fix:** Apply `track_focus`, `key_context`, and `on_action` directly to the `Stateful<Div>` returned by `template.render()` and return it without an outer wrapper.

- [ ] Remove outer `div()` wrapper in `ChoiceGroupControl::render`

---

#### 2. `item_position()` duplicated in gallery

The private SDK helper `item_position()` in `control.rs` is re-implemented identically in `apps/gallery/src/gallery/panes/choice_group/pane.rs` lines 373–380 for the static state preview. Since `ChoiceGroupItemPosition` is already public, this function should be made `pub` and re-exported so callers building static `ChoiceGroupRenderItem` values don't have to duplicate it.

- [ ] Make `item_position` `pub` (or at minimum `pub(crate)`) and re-export from `mod.rs`
- [ ] Remove the duplicate in the gallery pane

---

### 🟡 Notable

#### 3. `icon_button_like` is a leaky heuristic

`template.rs` lines 205–206:

```rust
let icon_button_like =
    model.kind == ButtonKind::Ghost && matches!(model.layout, ChoiceGroupLayout::Horizontal);
```

This is inferred from `kind` and `layout`, not an explicit intent flag. A consumer using `Ghost` kind in a non-icon context (e.g. a ghost-styled text button group) would get icon-button sizing and focus-ring behavior unintentionally.

- [ ] Replace the heuristic with an explicit signal — either a check against `ChoiceGroupStylePreset::IconButton` on the render model, or a dedicated flag

---

#### 4. `ChoiceGroupStylePreset` has limited value as a standalone type

The three variants (`Default`, `IconButton`, `Radio`) simply delegate to `icon_button_style()` and `radio_style()` on the builder, which are also `pub`. `Default` is a no-op. The enum adds indirection without adding expressiveness.

- [ ] Either make the enum the canonical configuration path (deprecating the method shortcuts), or drop it from the public API

---

#### 5. `bool_button_template` / `bool_button_template_factory` naming is confusing

These are exact delegates to `item_button_template` / `item_button_template_factory`. The name implies a `bool` *content* template rather than a *selection-state-typed* template (`ButtonTemplate<bool>`, where `bool` = selected).

- [ ] Rename to `selection_button_template` / `selection_button_template_factory`, or remove the aliases and use `item_button_template` everywhere

---

#### 6. Duplicated `apply_modifiers` in `ThemedChoiceGroupTemplate` and `ModifiedChoiceGroupTemplate`

Both structs independently hold `Vec<ChoiceGroupModifier>` and implement `apply_modifiers` identically (`template.rs` lines 49–54 and 114–119).

- [ ] Extract into a shared `ModifierChain` newtype to avoid drift

---

#### 7. `ChoiceGroupRenderModel` exposes `kind` but not style preset

Templates that need to differentiate "Default Standard" from "Radio Standard" (same `kind`, different visual intent) have no clean signal, which is the root cause of the `icon_button_like` heuristic above.

- [ ] Consider surfacing `ChoiceGroupStylePreset` (or an equivalent intent signal) on `ChoiceGroupRenderModel`

---

### 🟢 Minor

#### 8. Gallery description comment will be stale after removal

`apps/gallery/src/gallery/panes/choice_group/pane.rs` line 22–24:
> "Choice Group can model radio-like single-selection **without using RadioGroup**."

- [ ] Update description after `radio_group` is removed to describe `ChoiceGroup` on its own terms

#### 9. Defensive `normalize_selected_id_list` call in `compute_next_selected_ids`

In the `Multiple` path of `compute_next_selected_ids`, `normalize_selected_id_list` is called after pushing to deduplicate. If `normalize_model` has already run, duplicates in `current` should be impossible. The call is correct but worth a comment explaining the defensive intent.

- [ ] Add a comment clarifying why the dedup call is needed

---

## Plan: Removing `toggle_group` and `radio_group`

Both removals are clean and low-risk.

### `toggle_group` → `ChoiceGroup`

`ToggleGroup` is a strict subset — every field and behavior maps 1:1 to `ChoiceGroup` in Unmanaged mode.

Migration friction points for callers:
- `ToggleGroupItem` has no `value` field → add `.value(item.id)` or rely on the id-as-value default in `ChoiceGroupItem::new`
- `ToggleGroupEvent::Change` has no `value` field → any consumer reading `event.value` needs updating

- [ ] Migrate all `toggle_group` callers in gallery to `choice_group`
- [ ] Remove `toggle_group` module from SDK and `mod.rs`

### `radio_group` → `ChoiceGroup`

Already fully replaced. The gallery's `choice_group` pane demonstrates the full pattern end-to-end with `choice_group::radio_group()` + `bool_button_template`.

- [ ] Migrate all `radio_group` callers in gallery to `choice_group`
- [ ] Remove `radio_group` module from SDK and `mod.rs`
