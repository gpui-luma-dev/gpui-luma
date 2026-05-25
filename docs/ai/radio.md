# Radio Group Declarative Composition Control

## Goal

This control is not intended to be a purely headless primitive where app code
manually creates every radio button, subscribes to every click event, and keeps
all selected state synchronized by hand.

The goal is a declarative composition control:

- The control owns selection behavior, keyboard behavior, focus behavior, child
  control wiring, state synchronization, and change events.
- The caller owns item presentation through an item template.
- The control does not force one visual layout such as a horizontal row,
  vertical stack, bordered segmented group, or built-in panel.
- Common layouts can still be provided as convenience presets built on top of
  the same API.

This sits between two extremes:

- Too opinionated: `ChoiceGroup`-style controls that own behavior, item chrome,
  group chrome, and horizontal/vertical layout.
- Too low-level: a fully headless `SelectionGroup` where every consumer repeats
  registration, subscription, sync, and keyboard glue.

The desired API should remove the repetitive behavior code without taking away
the caller's ability to compose custom radio layouts.

## Core API Shape

The group should be generic over the caller's item type. The item type is domain
data owned by the caller; the radio group should not require a radio-specific
item model unless a convenience wrapper later chooses to provide one.

Pseudo-code:

```rust
pub struct RadioGroup<T> {
    id: SharedString,
    items: Vec<T>,
    selected: Option<T>,
    mode: SelectionMode,
    item_template: RadioItemTemplate<T>,
    on_change: Option<RadioGroupChangeHandler<T>>,
}
```

If selecting by full `T` is too restrictive, the implementation can separate
item data from identity:

```rust
pub struct RadioGroup<T, Id> {
    id: SharedString,
    items: Vec<T>,
    selected_id: Option<Id>,
    mode: SelectionMode,
    item_id: ItemIdFn<T, Id>,
    item_template: RadioItemTemplate<T>,
    on_change: Option<RadioGroupChangeHandler<Id>>,
}
```

The public builder surface should stay small:

```rust
impl<T> RadioGroup<T> {
    pub fn new(id: impl Into<SharedString>) -> Self;
    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self;
    pub fn selected(mut self, selected: T) -> Self;
    pub fn mode(mut self, mode: SelectionMode) -> Self;
    pub fn item_template<F>(mut self, template: F) -> Self;
    pub fn on_change<F>(mut self, handler: F) -> Self;
    pub fn build(self, cx: &mut Context<_>) -> Entity<RadioGroupControl<T>>;
}
```

`new(id)` only creates the builder/model with defaults. It should not create
child radio buttons or subscriptions. That work happens in `build(context)`,
after the caller has supplied the final items, selection mode, selected value,
template, and change handler.

Pseudo-code:

```rust
pub fn new(id: impl Into<SharedString>) -> Self {
    Self {
        id: id.into(),
        items: Vec::new(),
        selected: None,
        mode: SelectionMode::SingleRequired,
        item_template: default_radio_item_template(),
        on_change: None,
    }
}
```

## Exact Code Example

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
enum Density {
    Compact,
    Comfortable,
    Expanded,
}

impl Density {
    fn id(&self) -> &'static str {
        match self {
            Density::Compact => "compact",
            Density::Comfortable => "comfortable",
            Density::Expanded => "expanded",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Density::Compact => "Compact",
            Density::Comfortable => "Comfortable",
            Density::Expanded => "Expanded",
        }
    }
}

let density_radio_group = radio_group::new("density")
    .mode(SelectionMode::SingleRequired)
    .items([Density::Compact, Density::Comfortable, Density::Expanded])
    .selected(Density::Comfortable)
    .item_template(|density, selected, active, context| {
        div()
            .w(px(280.0))
            .flex()
            .items_center()
            .gap_2()
            .border_1()
            .rounded(px(8.0))
            .p_2()
            .when(active, |element| element.border_color(gpui::hsla(0.62, 0.60, 0.55, 1.0)))
            .child(
                radio_button::new(format!("density-{}", density.id()))
                    .typed(selected)
                    .template(context.theme().radio_button_template())
                    .spawn(context),
            )
            .child(
                div()
                    .text_size(px(13.0))
                    .line_height(px(18.0))
                    .child(density.label()),
            )
            .into_any_element()
    })
    .on_change(|selection_group_event, context| {
        tracing::info!(
            "density selection changed: changed_id={}, selected_ids={:?}",
            selection_group_event.changed_id(),
            selection_group_event.selected_ids()
        );
        context.notify();
    })
    .build(context);

// What build(context) does:
// 1. Creates the group entity and focus handle.
// 2. Creates child radio button entities from item_template.
// 3. Subscribes each child radio button click event to the selection controller.
// 4. Binds keyboard actions on the group container:
//    - SelectNextItem
//    - SelectPreviousItem
//    - SelectFirstItem
//    - SelectLastItem
//    - ActivateControl
// 5. Synchronizes child radio button selected data from selection state.
// 6. Emits one centralized selection change event to on_change handler.
```

## Important Design Boundary

`item_template(...)` is the main point of this design. It is not an escape hatch
around the control. It is how consumers declaratively compose the item UI while
the radio group still owns the behavior.

The template may decide whether an item looks like:

- a plain row with radio button plus label
- a compact horizontal control
- a full-width card
- a grid tile
- a panel with description text and metadata
- any other caller-owned composition

The template should not decide selection behavior. Selection changes should flow
through the group.

The group should not inject opinionated group chrome. It should not force a
specific border, background, segmented-control frame, gap, row direction, or
column direction for custom templates.

## Implementation Notes

- Keep the public surface compact:
  - `radio_group::new(id)`
  - `.mode(SelectionMode)`
  - `.items([...])`
  - `.selected(id)`
  - `.item_template(...)`
  - `.on_change(...)`
  - `.build(context)`

- `SelectionMode` behavior:
  - `SelectionMode::SingleRequired`: one option must always remain selected.
  - `SelectionMode::SingleAllowNone`: selection can be cleared.

- `build(context)` owns all wiring:
  - group entity/focus handle
  - keyboard action handlers
  - child click subscriptions
  - selected-state synchronization into child radio button `.with_data(...)`
  - unified change event emission

- `item_template(...)` owns item presentation:
  - The template receives current state (`selected`, `active`) and renders from
    that state.
  - The template can use the standard radio button template or any composed UI
    that still delegates activation to the group.
  - The template should not contain selection state transitions.
  - The template should be portable across row, column, card, panel, and grid
    compositions.

- `on_change(...)` is the single integration point for app state updates:
  - form model writes
  - analytics
  - command dispatch
  - dependent UI updates

- Keep item identity stable (`RadioItem::id`) so state synchronization and
  eventing remain deterministic.
- Keep control IDs deterministic (for example `format!("density-{}", item.id)`)
  for predictable subscriptions and debugging.
- Preserve one source of truth for selection inside the radio group controller.
- Prefer emitting full event payload (`changed_id`, `selected_ids`) in
  `on_change(...)` callbacks.
- Keep keyboard behavior at the group level, not independently implemented by
  each item.
- Keep per-item rendering stateless apart from incoming `selected` and `active`
  values.
- Avoid hidden side effects in templates; state transitions should happen
  through group actions only.
- If needed, expose read methods on the built control for external inspection:
  - `selected_id()`
  - `selected_ids()`
  - `active_id()`
  - `mode()`
- If item enable/disable state is supported, disabled items should be skipped by
  keyboard navigation and ignored by activation.
- If dynamic item updates are supported, re-normalize selection after every item
  list change.
- If mode is switched at runtime, re-normalize selection immediately to satisfy
  mode constraints.
- Keep event emission order stable: update internal state first, then emit
  `on_change(...)`.
- Keep focus behavior consistent: activation operates on the current active item.

## Presets Built On Top

Simple layouts should be easy to provide without changing the core design. These
presets should be thin wrappers around the same composition model, not separate
behavior implementations.

Examples:

```rust
let density_radio_group = radio_group::new("density")
    .items(density_items)
    .selected("comfortable")
    .vertical()
    .build(context);

let density_radio_group = radio_group::new("density")
    .items(density_items)
    .selected("comfortable")
    .horizontal()
    .build(context);
```

Those helpers can provide default item templates and simple container layout,
while custom layouts continue to use `.item_template(...)`.

## Relationship To Other Controls

- `RadioGroup` is the ergonomic public API for radio-specific composition.
- An internal selection controller/core may still exist, but it should not be
  the main consumer-facing API for this use case.
- `ChoiceGroup` can remain the more opinionated visual composite for cases where
  built-in group chrome and layout are desired.

Treat this as a mini-control: behavior-complete, visually composable,
declarative, and API-minimal.
