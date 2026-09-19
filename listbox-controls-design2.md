# ListBox Control Design

## Status

Proposed. This is a clean-slate replacement. The existing visual ListBox and
its builders and templates will be removed. There is no backward-compatibility
contract, compatibility adapter, or requirement to preserve its behavior.

Retain and adapt the ListBox inspectors with a narrower scope tied to the new
exposition's concrete composition. Replace the old exposition wiring and place
the new ListBox exposition in Luma Studio's **Selectors** category. Retain or
adapt look and inspection support needed by that composition; remove obsolete
theme APIs coupled only to the old visual control.

Update in-repository consumers to the new model. Shared ControlGroup and
selection infrastructure used by other controls remains independent; removing
ListBox does not imply removing those controls or their shared infrastructure.

## Architectural Position

`ListBox` is primarily a non-visual collection and interaction orchestrator.
It is not a default row renderer, a default viewport, or a default visual
control.

The SDK owns the meaning and behavior of the collection:

- item identity and owned collection snapshots;
- visible-item projection after filtering or other host-defined transforms;
- selection, active item, and anchor state;
- activation and interaction events;
- reusable focus, keyboard, and drag-and-drop mechanics as those capabilities
  are added.

The host owns the concrete composition:

- the scroll/viewport container;
- vertical or horizontal layout;
- row height, spacing, padding, and visible item count;
- row content and visual styling;
- borders, radius, empty states, and scroll policy.

There is intentionally no universal visual default for an arbitrary `T`.
The SDK cannot know whether an item should be rendered as text, a card, a
tree row, a thumbnail, or a custom interactive composition.

## The Target Composition

The design is a Rust/GPUI equivalent of this SwiftUI shape:

```swift
ScrollView(.vertical) {
    LazyVStack(spacing: rowSpacing) {
        ForEach(items, id: \.self) { item in
            renderRow(item)
        }
    }
}
.frame(height: viewportHeight)
```

The equivalent Luma composition keeps the same ownership boundary:

```rust
let listbox = ListBoxState::try_new(
    items,
    |item| item.id,
    SelectionMode::SingleAllowNone,
)?;

let viewport_height =
    row_height * visible_count as f32
    + row_spacing * (visible_count.saturating_sub(1) as f32);

div()
    .h(px(viewport_height))
    .overflow_y_scroll()
    .child(
        vstack! { gap = row_spacing; }
            .children(listbox.visible_items().into_iter().map(|item| {
                render_row(item)
            })),
    )
```

For horizontal composition, the host chooses the other stack explicitly:

```rust
div()
    .w(px(viewport_width))
    .overflow_x_scroll()
    .child(
        hstack! { gap = column_spacing; }
            .children(listbox.visible_items().into_iter().map(|item| {
                render_row(item)
            })),
    )
```

The SDK does not hide the `ScrollView`/`vstack!`/`hstack!` composition inside
the listbox state. The host should be able to read and change the layout
directly.

These examples show composition, not lifecycle or input wiring. Construct the
state once in its owner, handle construction errors there, and read it during
rendering. Rows use stable keys for element identity and send input through
the SDK interaction binding described below; rendering must not recreate or
mutate selection state.

### Horizontal Cards Sized to the Viewport

This SwiftUI example adds a useful layout case: the viewport width determines
card width so five cards and their intervening gaps fit in the usable width.

```swift
import SwiftUI

struct HorizontalViewportList: View {
    let items = (1...20).map { "Card \($0)" }
    let spacing: CGFloat = 8

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            LazyHStack(spacing: spacing) {
                ForEach(items, id: \.self) { item in
                    Text(item)
                        .containerRelativeFrame(
                            .horizontal, count: 5, span: 1, spacing: spacing
                        )
                        .frame(height: 80)
                        .background(Color.blue.opacity(0.2))
                        .clipShape(RoundedRectangle(cornerRadius: 8))
                }
            }
            .scrollTargetLayout()
        }
        .scrollTargetBehavior(.viewAligned)
        .safeAreaPadding(.horizontal, 16)
    }
}
```

`containerRelativeFrame` accounts for the gaps before dividing the usable
container width; each card is not simply one fifth of the outer viewport.
The stack receives `.scrollTargetLayout()` so its cards become alignment
targets for `.viewAligned`. See Apple's [container-relative sizing example](https://developer.apple.com/videos/play/wwdc2023/10159/)
and [view-aligned scrolling documentation](https://developer.apple.com/documentation/swiftui/scrolltargetbehavior/viewaligned).

For Luma, the host measures the usable viewport width, computes a uniform card
width, and composes `visible_items()` in an `hstack!`. Card wrappers must retain
that width without flex shrinking; the stack can extend beyond the viewport
and scroll horizontally. Recompute widths when the viewport resizes.

Card height, corner radius, inset padding, scrollbar visibility, and optional
snapping remain host layout/presentation choices. This SwiftUI sample illustrates
layout only; Luma adds selection through the same SDK interaction binding used
for vertical rows. Lazy composition and view-aligned snapping are reference
capabilities, not prerequisites for the initial Luma implementation.

## SwiftUI-to-Luma Mapping

| SwiftUI concept | Luma/GPUI responsibility |
| --- | --- |
| `items` | `ListBoxState<T, K>` snapshot |
| `ForEach` | `state.visible_items().map(...)` |
| `id:` | key extractor `Fn(&T) -> K` |
| `ScrollView(.vertical)` | host-owned vertical scroll container |
| `ScrollView(.horizontal)` | host-owned horizontal scroll container |
| `LazyVStack` | host-owned `vstack!` for the initial measured-flow implementation |
| `LazyHStack` | host-owned `hstack!` for the initial measured-flow implementation |
| row view | host-owned row composition or app template |
| `.frame(height:)` | host-owned viewport dimension |
| `.containerRelativeFrame(..., count:, spacing:)` | host measures viewport and calculates item extent |
| `.safeAreaPadding` | host accounts for viewport insets in usable extent |
| `.scrollTargetLayout()` / `.viewAligned` | optional host scroll-target registration and snapping |
| selection behavior | `SelectionModel<K>` inside `ListBoxState` |
| list interaction events | `ListBoxEvent<K>` |

GPUI's initial implementation may eagerly compose visible snapshot items. True
virtualization is a later optimization and is not required to establish the
composition or behavior boundary.

## Core SDK Model

```rust
pub struct ListBoxState<T, K> {
    snapshot: ListBoxSnapshot<T, K>,
    selection: SelectionModel<K>,
    projection: ListBoxProjection<K>,
    // Private interaction policy and focus state are omitted here.
}

pub struct ListBoxVisibleItem<'a, T, K> {
    pub key: K,
    pub item: &'a T,
    pub source_index: usize,
    pub visible_index: usize,
    pub state: ListBoxItemState,
}
```

`ListBoxState` is a model. It does not implement `Render`, does not create a
viewport, and does not choose a row template.

The host may store the state in a GPUI `Entity`, a parent view, or another
appropriate owner. That lifecycle choice is separate from listbox behavior.

The types above are API sketches. Snapshot, selection, and projection are
read-only through accessors. All mutation goes through `ListBoxState`
operations so validation, reconciliation, and event production happen as one
transaction. Do not expose mutable access to the internal `SelectionModel`.

`ListBoxItemState` exposes at least `selected`, `active`, and `enabled`.
Pointer hover, pressed appearance, and focus-visible presentation belong to
the interaction binding and look layer, rather than the collection snapshot.

### Identity

Keys `K: Clone + Eq + Hash` must be stable and unique within a snapshot.
`ListBoxSnapshot` owns its items and captures their keys on construction.
Items are exposed by shared reference; replacing an item requires a snapshot
update. A key change represents removal of the old item and insertion of a
new item. Selection, active item, and anchor are reconciled on replacement.

Construction and replacement reject duplicate keys with a typed error rather
than panicking or silently merging items. Failed updates leave the previous
state intact. Replacing a value under an existing key preserves its selection
unless its eligibility changes.

Snapshot construction also accepts a host-provided enabled predicate, defaulting
to all enabled, and captures the result for each item. Disabled items may be
displayed but cannot be selected, activated, or used as navigation targets.

### Selection

```rust
pub enum SelectionMode {
    None,
    SingleRequired,
    SingleAllowNone,
    Multiple,
    Extended,
}
```

- `None`: selection stays empty; active-item navigation and activation remain
  available for enabled items.
- `SingleRequired`: exactly one enabled source item is selected whenever one
  exists. With no enabled items, selection is empty.
- `SingleAllowNone`: at most one enabled item is selected. An SDK policy controls
  repeated-click behavior (`KeepSelected` by default or `ToggleOff`); the host
  configures that policy rather than implementing selection logic itself.
- `Multiple`: independent toggle selection.
- `Extended`: desktop-style replacement, modifier toggle, and range selection.

Selection is key-based. The host resolves keys back to domain values when it
needs to perform application work.

Reconciliation and interaction rules:

- Filtering preserves selection of enabled source items, including hidden ones.
  Removing or disabling an item clears its selection. `SingleRequired` falls
  back to the first enabled item in source order if its selection is lost.
- Active items must be enabled and in the projection. When the active key is
  lost or hidden, use the first visible selected item, then the first enabled
  visible item, or `None` if neither exists.
- The range anchor must also be enabled and visible. Clear it when it is
  removed, disabled, or hidden. A range gesture without an anchor uses the
  pre-gesture active item, or its target if no active item exists.
- `Extended` plain clicks replace selection and set the anchor; platform toggle
  modifiers toggle the target and set the anchor. Shift replaces selection with
  the inclusive visible range; toggle-plus-Shift adds that range. Disabled rows
  are skipped, and the anchor remains fixed while extending a range.
- Plain navigation moves only the active item by default. An explicit
  `selection_follows_active` policy may select it in single-select modes.
  Navigation does not wrap. Shift navigation in `Extended` applies range rules.
- Selection events list keys in source order. Reordering alone does not count
  as a selection change when membership is unchanged.

### Visible Projection

The projection is an ordered subset of snapshot keys. The default projection
contains every source key in source order. Hosts may supply an ordered key
sequence produced by filtering, sorting, or another transform; a later filter
helper can produce the same representation. Unknown or repeated keys are
validation errors. Group headers and synthetic rows are outside the initial
projection contract.

`visible_items()` iterates the projection, not just the rows currently inside
the viewport. Each item retains both source and visible indices. Navigation
and range selection use visible order; indices are valid only for the current
snapshot/projection. Persistent state and callbacks use keys.

Snapshot replacement accepts the next projection in the same transaction. If
omitted, it resets to all source keys; hosts retaining a filter or sort supply
the newly computed projection. Query text belongs to the host or filter helper,
not to the core projection model.

## Mutation and Interaction Contract

The model exposes operations for snapshot replacement, projection replacement,
programmatic selection replacement, selection-policy changes, and semantic
input. Semantic input includes row selection with modifiers, next/previous,
first/last, explicit activation, and focus entry/exit. Input targets use keys;
stale, hidden, or disabled row targets are ignored.

Each successful mutation returns a `ListBoxUpdate<K>` containing a `changed`
flag, ordered `ListBoxEvent<K>` values, and host effects such as
`RevealItem { key }`. Invalid replacement data returns an error. Programmatic
selection replacement rejects unknown, disabled, duplicate, or mode-incompatible
keys; empty selection is invalid for `SingleRequired` when eligible items exist.

The state commits and reconciles before returning the update. The GPUI owner
forwards events to its event stream and calls `cx.notify()` when `changed` is
true. A model embedded in a parent does not need its own entity or `Render`
implementation. Construction establishes initial invariants without emitting
change events.

### GPUI Interaction Binding

A small SDK binding translates GPUI input into semantic operations and applies
updates through the owner. It does not create a viewport or choose row content.
Establish its contract in Phase 1, implementing keyboard mapping, activation,
and reveal effects in Phase 2:

- stable row element IDs scoped by list identity and item key;
- one list focus handle/tab stop, with active item distinct from keyboard focus;
- row clicks selecting the target and moving active state; selection alone
  does not imply `ItemActivated`;
- explicit activation from Enter or a double-click, and selection from Space;
- host-configured vertical/horizontal key mapping to axis-agnostic commands;
- focus-within tracking and an event boundary so nested SDK controls can handle
  input without also selecting or activating their containing row;
- `RevealItem` requests after keyboard movement, with the host resolving keys
  to row bounds and scrolling its own viewport.

For double-click activation, apply selection only once for that gesture so
toggle modes do not immediately undo the first click. Focus exit preserves
selection and active state. The binding supplies focus semantics and state for
accessible row presentation without requiring a universal visual template.

App-owned layout containers may use `div`, `vstack!`, and `hstack!`. Interactive
controls within rows use SDK controls and look factories such as `ShadcnLook`,
builders, and `.spawn(cx)`. The row selection surface uses the SDK binding;
applications should not reimplement input handling as raw styled `div` controls.

## Event Contract

Events communicate behavior without requiring the SDK to own visual elements:

```rust
pub enum ListBoxEvent<K> {
    SelectionChanged {
        selected: Vec<K>,
        active: Option<K>,
    },
    ActiveItemChanged {
        active: Option<K>,
    },
    ItemActivated {
        key: K,
    },
    ProjectionChanged {
        visible_count: usize,
    },
    FocusChanged {
        is_focused: bool,
    },
}
```

The event contract describes what happened. The host decides how the event
changes application data or presentation.

For a transaction producing multiple events, order them as `ProjectionChanged`,
`SelectionChanged`, `ActiveItemChanged`, `FocusChanged`, then `ItemActivated`.
Emit each change event at most once, only when that aspect changes. If both
selection and active item change, emit both events; their payloads describe the
same committed state. Programmatic mutations and reconciliation follow the
same rules as user input. Explicit activation may emit an event without a
state change. An operation that leaves all state unchanged emits no change
events; toggle policies may still change selection on a repeated click.

`ProjectionChanged` means projected key membership or ordering changed, even
when `visible_count` is unchanged. Updating item content under unchanged keys
still marks the update as changed for rendering, without inventing a selection
or projection change. Scroll requests are effects, not assertions that scrolling
has already happened.

## Drag-and-Drop Boundary

ListBox is an early consumer of shared SDK drag-and-drop infrastructure, not
the owner of all drag-and-drop mechanics.

Shared SDK infrastructure should eventually provide:

- drag threshold and lifecycle;
- pointer tracking and cancellation;
- bounds/hit testing;
- previews and ghost handling;
- edge auto-scroll;
- extension of the basic interactive-child boundaries to drag gestures.

ListBox-specific policy should provide:

- which items are draggable;
- how the current selection becomes a drag session;
- what `before`, `after`, empty-list, and disallowed targets mean;
- which `ListBoxEvent` is emitted;
- how the host mutates its collection after a drop.

The host remains responsible for applying a reorder or cross-list move to its
domain collection and providing the next snapshot to `ListBoxState`.

This keeps the first DND implementation useful without making ListBox the
place where reusable DND infrastructure is invented or hidden.

## Layout and Viewport Policy

The listbox state is axis-agnostic. Vertical and horizontal layout are host
composition choices, not state configuration that silently changes rendering.

For a fixed number of visible items:

```text
viewport_size = item_extent * visible_count
                + spacing * max(visible_count - 1, 0)
                + container_padding
```

The host may express this in GPUI logical pixels initially. A later SDK
helper may calculate the dimension from a visible-item count, row metrics, and
spacing. Both forms are layout concerns and should not be embedded in the
selection model.

For the inverse case, where the viewport is known and a fixed number of cards
should fit, use:

```text
usable_extent = max(viewport_extent - leading_inset - trailing_inset, 0)
item_extent = max(usable_extent - spacing * (visible_count - 1), 0)
              / visible_count
```

Here `visible_count` must be positive and spacing/insets nonnegative. Account
for insets only once: an already-measured content area is the usable extent.
With five cards, spacing 8, and 16-point insets on each side, an outer width
of 600 gives `(600 - 32 - 32) / 5 = 107.2` logical pixels per card. The count
specifies layout slots, not the number of items in the collection or projection.
If the usable extent cannot accommodate the gaps, the host must reduce spacing
or the requested count; clamping alone cannot guarantee that the cards fit.

Scroll snapping, variable-height measurement, and virtualization are separate
capabilities. They should be added only after the basic composition works.

## Studio Exposition and Inspectors

The new ListBox exposition belongs in **Selectors**, alongside the other
selector controls. Move its catalog entry from `ControlCategory::Choice` to
`ControlCategory::Selection`, and update its description and code sample to
show the new state-driven composition.

Keep the inspectors, adapting their existing infrastructure to the smaller
surface actually demonstrated by the host:

- inspect colors and interaction-state styling used by the exposition's row
  composition and any styled viewport;
- inspect the row metrics, spacing, padding, and viewport dimensions actually
  used by that composition;
- expose only applicable parts, states, and sizes; remove obsolete controls
  and values that described the legacy visual ListBox.

Inspector values must come from the same look and layout inputs used to render
the sample. Label host-owned layout values as composition settings. These
inspectors describe the Studio example, not a universal ListBox theme or an
SDK-owned viewport. Their adapters and metadata may change without preserving
legacy APIs; the inspector capability remains part of the new exposition.

## Development Phases

### Phase 1: Small Working Slice

Implement and demonstrate exactly two single-select examples in Luma Studio's
Selectors category: vertical rows and horizontal cards. Both use the same SDK
state and input binding, with independent selection and scrolling.

1. Create `ListBoxState` with a small set of typed items.
2. Compose a bounded vertical viewport showing five of twenty rows.
3. Compose a horizontal viewport with twenty compact cards, 8px spacing, and
   36px card height and fixed 112px width. Cards never shrink to fit the pane;
   narrower viewports show fewer cards and scroll to the remaining items.
   Use named card and row components, keeping item presentation separate from
   list state, input binding, and exposition assembly.
   Both examples contain wheel scrolling at their endpoints so it does not
   propagate to the exposition pane.
4. Compose the examples with `vstack!` and `hstack!`, respectively.
5. Bind stable row IDs, list focus, clicks, basic keyboard navigation, and
   activation through the SDK interaction binding. Reveal keyboard targets
   through each host-owned viewport.
6. Apply selection through `ListBoxState` and forward the returned update.
7. Render the selected state and show labeled events from both examples.
8. Retain scoped color and layout inspectors for both compositions.

This phase is complete only when the result is visible and manually testable in
Luma Studio, and model tests cover identity validation, selection reconciliation,
and event production. Replace the old exposition, adapt its inspectors, and
update consumers to remove dependencies on the old visual ListBox. Remove
obsolete ListBox-specific APIs while retaining the look and inspection support
used by the new composition; no compatibility layer is part of this phase.

### Phase 2: Basic Variants

The Studio examples now live in independent `vertical.rs` and `horizontal.rs`
modules. Each owns its item type, item component, state, binding, layout, and
scroll handling; neither is an axis variant of a shared example view. Both
currently use `SelectionMode::Multiple` and display a selected-item count.
Selection events remain in source order; reorder-only changes do not emit a
selection event. Extended Shift-range gestures remain a separate future mode.

- multiple toggle selection (implemented): click or Space toggles an enabled
  item, Cmd/Ctrl+A selects all enabled items, and Cmd/Ctrl+Shift+A clears selection
  while keeping list focus. Escape leaves focus through the enclosing focus scope;
  programmatic `set_selected_keys` validates and replaces selection atomically;
- visible-count and row-metric helpers;
- filtering and active-item presentation;
- extended selection and its anchor/modifier rules.

These variants reuse the same state and projection model. They do not require
a visual ListBox builder.

### Phase 3: Shared Interaction Infrastructure

- extend the existing row event boundaries for drag gestures;
- drag lifecycle and pointer tracking;
- listbox drop proposals and host mutation events;
- edge auto-scroll and drag previews.

### Phase 4: Performance Enhancements

- measured variable-height flow improvements;
- opt-in uniform-row virtualization;
- indexed height estimation if variable-height virtualization is needed.

## Explicit Non-Goals

The initial design does not require:

- a `ListBox<T, K>` visual builder;
- a default row renderer;
- injectable shell or item-template callbacks;
- an SDK-owned viewport or stack;
- a universal styling preset for arbitrary domain values;
- full DND before the basic vertical composition is working;
- backward compatibility with the legacy visual ListBox API.

The legacy visual ListBox is removed, not retained as an optional shell for the
new model. Existing legacy APIs do not constrain naming or behavior here.

If repeated application composition later demonstrates a real need for a
reusable visual shell, that can be designed separately. It should not be
assumed as part of the behavior model.

## Acceptance Criteria

The design is working when:

- the Luma Studio exposition appears in Selectors and visibly shows a scrolling
  vertical list;
- scoped inspectors remain available and reflect the new composition's actual
  styling and layout inputs;
- the list is built from `ListBoxState.visible_items()`;
- rows are composed by the host using `vstack!`;
- clicking a row updates selection through `ListBoxState` and changes its visual state;
- a `ListBoxEvent::SelectionChanged` appears in the event stream;
- the same state model can later feed an explicit `hstack!` composition;
- the SDK does not need to know what an arbitrary item looks like.

Behavioral acceptance checks, as the corresponding phases land:

- duplicate snapshot keys and invalid projections fail without partial mutation;
- snapshot replacement preserves stable-key selection and reconciles removal or
  disabled items, including empty and all-disabled collections;
- filtering preserves hidden selection while reconciling active item and anchor;
- sorted/filtered range selection follows projected order;
- repeated clicks, programmatic updates, and reconciliation emit exactly the
  documented events, with no duplicate change events for no-op operations;
- keyboard input respects focus and nested controls, and reveals the active row
  through the host viewport in both vertical and horizontal compositions;
- in-repository consumers compile against the replacement, with no dependency
  on removed visual ListBox APIs; retained inspectors use the new composition's
  look and layout inputs.
