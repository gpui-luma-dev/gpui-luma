# ListBox Control Design

## Status

Implemented foundation with planned extensions. The clean-slate replacement is
working in Luma Studio's **Selectors** category: independent vertical and
horizontal examples, plus a pair demonstrating single/group transfers,
same-list reordering, gap insertion, and configurable basic linear auto-scroll.
The user has manually tested these interactions. Model tests cover selection,
reconciliation, drop validation, and event production.

The legacy visual ListBox, builders, and templates have been replaced without
a compatibility layer. The inspectors remain, scoped to the exposition's
concrete composition. Shared ControlGroup and selection infrastructure used by
other controls remains independent.

Selection modes now include no selection, both single modes, multiple toggle,
and extended ranges, with runtime policy controls on the vertical/horizontal
examples. The user has confirmed the new selection policies, focus-required
wheel scrolling, and single-click focus exit are tested and working in Studio.

### Local Commit Checkpoint — 2026-09-21

The current implementation is a working checkpoint to commit locally before
starting SDK DnD extraction. Completed and manually verified: the independent
vertical/horizontal examples, retained inspectors, selection policies and their
runtime controls, focus-aware wheel routing, and the DnD pair's single/group
transfers, same-list reordering, gap insertion, auto-scroll, and event tracking.

The latest automated verification passed 37 SDK ListBox tests, 15 Studio transfer
tests, and Clippy for the SDK and Studio. Changed Rust files are formatted;
workspace formatting still reports the unrelated existing `team.rs` issue.
The app was tested by the user, not launched by the agent.

The subsequent SDK DnD extraction is implemented. Shared sessions, keyed gap
admission, drop proposals, lifecycle notifications, and nested-control boundaries
now live in `luma::infra::drag_drop`; selection capture lives in `ListBoxState`.
Studio uses these helpers while retaining domain collection mutation and visual
composition. The user has tested the extracted SDK wiring and confirmed it working.

### SDK and Look Migration — User Verified

The user tested the local builder and markup composition and approved migration
of the reusable implementation. The macro implementation stays in the exposition
for continued syntax review.

- **SDK (`controls::listbox`):** `ListBoxControl`, `ListBoxRenderParts`, fixed-item
  `ListBoxFlow`/`ListBoxLayout`, `ListBoxItemRenderModel`, and the optional
  `make_listbox_item_template` adapter. State, focus, scroll handles, and the host
  input callback persist across renders. Layout describes geometry only; the SDK
  has no Shadcn dependency or knowledge of Studio's inspector layout types.
- **look-shadcn:** `ListBoxBuilder` and `ShadcnLook::render_listbox` compose SDK
  bindings with the look's surfaces, selection/hover/focus appearance, corner
  radius, insertion highlights, and optional DnD. Explicit `.look(&look)` follows
  other look builders. The lower-level builder borrows host-owned state/handles
  and finishes with `.build(window, cx)`.
- **Exposition:** `markup.rs` retains the two `listbox!` grammar arms, calling the
  look renderer with SDK flow values. Sample data, inline/named content templates,
  event logging, inspector metrics, previews, and domain transfer rules stay here.

The DnD pair now uses the exported Shadcn builder. Domain mutation and lifecycle
completion remain host-owned, preserving atomic cross-list updates. Omitting
`.drag_and_drop(...)` installs ordinary selection and scrolling only. Direct SDK
state mutations deliver rendering/reveal effects through `control.handle_update`.

The builder's headless DnD dispatch tests moved to look-shadcn (its `test-support`
feature enables them). SDK tests cover geometry and existing ListBox behavior;
Studio keeps macro integration, inspector geometry, and domain mutation tests.
No macro is exported by the SDK or the look. The app has not been launched by
the agent; the user confirmed the migrated implementation working.

### Local Markup Adapter — User Verified

The two top exposition examples now read as nested `listbox!` → `scroll_view!`
→ stack declarations, with an inline `item_template` closure in each stack.
`markup.rs` implements two small `macro_rules!` arms over the migrated typed
renderer and builder. The outer macro consumes the scroll and stack declarations;
they are not additional global macros or extensions to the SDK's normal stack grammar.
Template expressions are ordinary Rust and may use the existing `hstack!`,
`vstack!`, GPUI elements, named components, and SDK controls.

- The vertical example declares five visible items, 36px item height, and 4px
  spacing. The adapter computes the 196px viewport; it never inspects the template
  implementation to discover dimensions. The complete list remains 250px wide.
- The horizontal example retains 136px cards, 36px height, and 8px spacing in
  an available-width viewport, preserving the previously tested card sizing.
- Each template receives `ListBoxItemRenderModel` (item, selected, active,
  enabled) and `&mut App`, returning GPUI content. Named functions and inline
  closures share the same contract. Named functions can return `Div` or
  `AnyElement`, following existing SDK templates; an opaque Rust 2024 return
  needs `impl IntoElement + use<>` to avoid capturing the input lifetimes.
- Templates run during rendering and can borrow local data, with no imposed
  `Send`, `Sync`, or `'static` closure requirement. Retained GPUI callbacks still
  follow GPUI's ordinary ownership requirements.
- The SDK `ListBoxControl` retains the existing SDK state, binding, scroll handle,
  and host input callback across renders. Runtime policy changes, selection
  events, disabled items, focus-required scrolling, and keyboard reveal remain
  driven by the SDK. Shadcn factories style the interaction surface around the
  template; templates do not receive or wire that surface.

This is a local composition prototype, not a general UI language. Declaration
order is fixed, sizing is explicit, and the markup currently targets these two
non-DnD examples. The DnD pair continues to use the lower-level Shadcn builder.
Relative-width card counts and variable-size flow are not introduced by this
adapter. Phase 4 adds opt-in uniform-item virtualization underneath the same
markup and template contract.

Migration verification passes 40 SDK ListBox tests, four look tests (including
three migrated headless builder/DnD tests), and 18 Studio ListBox tests. These
include rendering of the actual exposition to check inspector geometry, named
and inline templates, keyboard selection/reveal on both axes, runtime policies,
disabled-item skipping, empty collections, focus-based wheel routing with scroll
retention, and the existing transfer/reorder behavior.
Both SDK ListBox doctests, Clippy across the three crates, and workspace formatting
checks also pass.

Still planned: filtered/sorted projections, viewport-relative card sizing, custom
auto-scroll curves. Scroll snapping has been removed at the user’s request (Phase 4c). Content-sized vertical rows now support
both eager and measured virtualized rendering (see Phase 4b). Phase 4 below records the
new uniform-item virtualization implementation and its verification status.

## Architectural Position

`ListBox` is primarily a non-visual collection and interaction orchestrator.
It is not a default row renderer, a default viewport, or a default visual
control.

The SDK owns the meaning and behavior of the collection:

- item identity and owned collection snapshots;
- visible-item iteration in snapshot order; filtered/sorted projections are planned;
- selection, active item, and a stable-key range anchor;
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

This is a reference layout option, not the current Studio example. Studio uses
fixed 136px cards; fitting five cards to a measured viewport remains a host choice.

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
| selection behavior | key-based selection owned by `ListBoxState` |
| list interaction events | `ListBoxEvent<K>` |

Eager composition remains the default. Phase 4 adds opt-in uniform-item
virtualization on both axes without changing the state or template contract.

## Core SDK Model

```rust
pub struct ListBoxState<T, K> {
    snapshot: ListBoxSnapshot<T, K>,
    selected: HashSet<K>,
    active: Option<K>,
    policy: SelectionPolicy,
    anchor: Option<K>,
    focused: bool,
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

The types above show the current model shape. Snapshot and selection are
read-only through accessors. There is no separate `SelectionModel` or configurable
`ListBoxProjection` in this implementation. All mutation goes through `ListBoxState`
operations so validation, reconciliation, and event production happen as one
transaction. Internal selection membership is not exposed for mutation.

`ListBoxItemState` exposes at least `selected`, `active`, and `enabled`.
Pointer hover, pressed appearance, and focus-visible presentation belong to
the interaction binding and look layer, rather than the collection snapshot.

### Identity

Keys `K: Clone + Eq + Hash` must be stable and unique within a snapshot.
`ListBoxSnapshot` owns its items and captures their keys on construction.
Items are exposed by shared reference; replacing an item requires a snapshot
update. A key change represents removal of the old item and insertion of a
new item. Selection and active item are reconciled on replacement. The range anchor survives reordering and is cleared if its item is removed or disabled.

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

- `None`: selection stays empty; navigation and activation remain available.
- `SingleRequired`: exactly one enabled source item is selected whenever one
  exists. With no enabled items, selection is empty.
- `SingleAllowNone`: at most one enabled item is selected. Repeated clicks keep
  it selected unless `toggle_off` is enabled; clearing selection is also an explicit operation.
- `Multiple`: independent toggle selection.
- `Extended`: plain click/Space replaces selection; Cmd/Ctrl toggles the target;
  Shift selects an inclusive range, and Cmd/Ctrl+Shift adds the range. Disabled
  items are skipped. Shift+arrows/Home/End extends the range and reveals its target.

`SelectionPolicy { mode, toggle_off, selection_follows_active }` is configurable
at runtime with `set_selection_policy`; `set_selection_mode` changes only mode.
Both flags default to false. `toggle_off` applies only to `SingleAllowNone`.
`selection_follows_active` selects subsequent navigation targets in either single
mode; enabling it alone does not immediately change selection. Flags are retained
but inactive in other modes.

A policy change preserves focus and active item, clears the anchor, and reconciles
selection atomically. `None` clears it. A transition to a single mode retains the
selected active key, or the first selected source key. Entering `SingleRequired`
with empty selection chooses the active enabled item, then the first enabled item.
Empty/all-disabled snapshots remain unselected. Identical policy updates are no-ops.
`SelectionPolicyChanged` reports a changed policy, followed by any selection change.
Programmatic nonempty selection in `None` mode returns `SelectionDisabled`.

Plain/toggle gestures establish the range anchor; ordinary navigation establishes
it at the new active item. Shift gestures keep it fixed. With no anchor, use the
pre-gesture active item or the target. Clear/select-all and successful explicit
selection replacement clear the anchor. Failed replacements leave it intact. Anchor-only changes set `changed` without
inventing selection events.

```rust
let update = state.set_selection_policy(SelectionPolicy {
    mode: SelectionMode::SingleAllowNone,
    toggle_off: true,
    selection_follows_active: true,
});
scroll.handle_update(&update, cx);
// Deliver update.events through the host's event handling.
```

Selection is key-based. The host resolves keys back to domain values when it
needs to perform application work.

Current reconciliation removes selected keys that are removed or disabled.
`SingleRequired` falls back to the first enabled item. A lost/disabled active
item falls back to the first selected enabled item, then the first enabled item,
or `None`. Navigation moves only the active item, skips disabled rows, and does
not wrap. Selection events list keys in source order; reordering alone does not
emit a selection event.

Implemented range rules and planned projection reconciliation:

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

Currently `visible_items()` iterates every snapshot item in source order,
including items outside the viewport. Source and visible indices are identical.
There is no projection-replacement API yet. The following is the planned contract.

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

The model currently exposes snapshot replacement, programmatic selection
replacement, combined snapshot/selection/active replacement, and semantic input.
Input includes select/toggle, modifier selection, range navigation, select-all/clear,
next/previous, first/last, activation, and focus entry/exit. Targets use keys; stale
or disabled row targets are ignored. Runtime policies are supported; projection
replacement remains planned.

Each successful mutation returns a `ListBoxUpdate<K>` containing a `changed`
flag, ordered `ListBoxEvent<K>` values, and an optional `reveal: Option<K>` scroll
request. Invalid replacement data returns an error. Programmatic selection
replacement deduplicates repeated keys and rejects unknown, disabled, or
mode-incompatible keys; empty selection is invalid for `SingleRequired` when
eligible items exist. Snapshot construction still rejects duplicate item keys.

`replace_snapshot_with_selection(snapshot, keys, active)` validates selection
and an explicit active key against the new snapshot before committing anything.
It preserves focus and returns one final update. An `active` value of `None`
chooses the first selected enabled item, then the first enabled item, or stays
empty when none exists. The operation does not request scrolling automatically.

The state commits and reconciles before returning the update. The GPUI owner
forwards events to its event stream and calls `cx.notify()` when `changed` is
true. A model embedded in a parent does not need its own entity or `Render`
implementation. Construction establishes initial invariants without emitting
change events.

### GPUI Interaction Binding

The SDK binding translates GPUI input into semantic operations and applies
updates through the owner. It does not create a viewport or choose row content.
The current binding supplies (construct it with `ListBoxBinding::new(cx)` and
pass `state.selection_mode()` into `bind_root` each render):

- stable row element IDs scoped by list identity and item key;
- one list focus handle/tab stop, with active item distinct from keyboard focus;
- row clicks selecting the target and moving active state; selection alone
  does not imply `ItemActivated`;
- Enter selects the active item if needed, then emits activation in the same
  transaction; double-click activates, while Space retains selection/toggle behavior;
- host-configured vertical/horizontal key mapping to axis-agnostic commands;
- focus-within tracking and an event boundary so nested SDK controls can handle
  input without also selecting or activating their containing row;
- `reveal` requests after keyboard movement, with the host resolving keys
  to row bounds and scrolling its own viewport, directly or through the
  optional `ListBoxScrollHandle<K>` adapter.

`ListBoxScrollHandle<K>` attaches to a host-provided surface and scrolling
stack. In eager mode the stack must have one direct child per visible item, in
visible order. Uniform virtualization instead uses the adapter's `virtual_window`
and `bind_virtualized` path, with spacers for items outside the rendered range.
It resolves reveal keys against the current snapshot and delegates measured
scrolling on either axis to GPUI in eager mode; uniform virtualization calculates
item offsets from fixed dimensions. It also contains wheel propagation and reveals
the focused active item after viewport resize. Item dimensions, layout, and
appearance remain host choices; eager mode needs no uniform-row arithmetic.
The host calls `handle_update` for repaint/reveal effects and delivers events to
its own application code.

Wheel scrolling defaults to hover targeting. A host can opt into
`ListBoxScrollHandle::default().require_focus_for_scroll(true)` for lists
embedded in a scrolling page, as all four Studio exposition lists do. This
uses the existing binding's focus state: an unfocused list passes wheel input
through to the page; a focused list contains wheel input at both endpoints.
Wheel input outside the list still reaches the page. Bind the whole surface as
the list root so clicking its padding also focuses it. Tab and Escape retain
the SDK's existing focus-scope behavior; focus loss preserves selection and
scroll position. The look paints a surface focus border for pointer and
keyboard focus. Drag edge auto-scroll remains independent of focus. Pending
item reveal requests wait until focus returns when this policy is enabled.

For double-click activation, apply selection only once for that gesture so
toggle modes do not immediately undo the first click. Focus exit preserves
selection and active state. The binding supplies focus semantics and state for
accessible row presentation without requiring a universal visual template.

Enter maps to `ConfirmActive`: an unselected active item follows the mode's
plain selection policy (add in Multiple, replace in Extended/single modes).
An already-selected item stays selected, preserving any selected group even
with toggle-off enabled. None mode only activates. Any `SelectionChanged` event
precedes `ItemActivated`; programmatic `Activate`/`ActivateActive` remain
activation-only operations.

App-owned layout containers may use `div`, `vstack!`, and `hstack!`. Interactive
controls within rows use SDK controls and look factories such as `ShadcnLook`,
builders, and `.spawn(cx)`. The row selection surface uses the SDK binding;
applications should not reimplement input handling as raw styled `div` controls.

## Event Contract

Events communicate behavior without requiring the SDK to own visual elements:

```rust
pub enum ListBoxEvent<K> {
    SelectionPolicyChanged {
        policy: SelectionPolicy,
    },
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
`SelectionPolicyChanged`, `SelectionChanged`, `ActiveItemChanged`, `FocusChanged`, then `ItemActivated`.
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

Cross-list drops return one update per list. Destination snapshot, selection,
and active item commit together through `replace_snapshot_with_selection`;
source reconciliation also returns only its final changes. Both states are
committed before the host delivers either update. A later GPUI focus change is
a separate interaction, not an intermediate selection event from the drop.

The Studio tracker formats typed SDK `DragDropEvent<S, K>` notifications alongside
the ListBox state-change events. `S` identifies the host collection; `K` identifies
an item. These notifications are independent of domain mutation:

- `DragStarted`: once when GPUI starts the gesture, with source-list label and
  captured keys; pressing the mouse without dragging does not emit it.
- `ItemsRemoved` and `ItemsAdded`: one batch per source/destination for a committed
  cross-list transfer, with keys in final list order and the other list's identity.
- `ItemsReordered`: one batch for a changed same-list drop, with keys and gap.
- `Dropped`: once for an accepted drop, identifying source, target, keys, requested
  `before` anchor (`None` means append), and `changed`.
- `DropRejected`: when validation rejects an owned drop, including its error.
- `DragEnded`: once per started drag, with keys and outcome: `Transferred`,
  `Reordered`, `Unchanged`, `Rejected`, or `Cancelled`.

After a successful commit, final SDK state-change events precede the batch
mutation notifications, `Dropped`, and `DragEnded`. Accepted no-op self-drops
emit `Dropped { changed: false }` and `DragEnded`, without mutation notifications.
Escape or release outside an accepted target ends the drag as `Cancelled`,
without `Dropped` or mutation notifications. Native preview release supplies
the cancellation fallback; an end guard prevents duplicate `DragEnded` entries.
These notifications are separate from `ListBoxEvent`: shared DnD infrastructure
uses host-provided collection identities and reports the result of a host commit.

## Drag-and-Drop Boundary

ListBox is an early consumer of shared SDK drag-and-drop infrastructure, not
the owner of all drag-and-drop mechanics.

GPUI supplies native threshold detection, pointer tracking, hit testing, and
preview lifetime. The SDK adds reusable mechanics without owning domain items:

- `ListBoxState::drag_keys(&key)` captures selected keys in source order when
  starting on a selected row, otherwise only that enabled row. Unknown/disabled
  targets return `None`; capture never mutates selection or the collection.
- `KeyedDrag<S, K>` captures unique nonempty keys and a source collection ID.
  A shared owner `EntityId` scopes cooperating lists. Pending, foreign, cancelled,
  and completed sessions cannot enter drop targets or trigger auto-scroll.
- `bind_drag_source` attaches GPUI dragging to a host surface, reports
  `DragStarted`, and watches native preview release for cancellation. Clones share
  a lifecycle guard, so every started session ends at most once.
- `KeyedDropTarget` attaches scope admission and produces `DropProposal<S, K>`
  with source, destination, captured keys, and a stable `before` key. `None`
  means append, including empty destinations. Hosts validate keys against current
  collections at commit time; proposals do not assert that keys still exist.
- `DropZone` positions before/after half-item hit areas on either axis, including
  the following gap and a marker kept visible at clipped viewport edges. The
  host supplies item extent, gap, marker width, colors, and relative wrappers.
- `DropProposal::committed(changed, ordered_keys)` produces typed batch/drop/end
  notifications after a host commit. `rejected(error)` reports failure without
  mutation. Deferred handlers must check `is_active()` before mutating; cancelled
  or completed proposals cannot produce a second terminal notification.
- `DragDropElementExt::drag_boundary()` wraps embedded interactive controls:
  child clicks do not select the parent row, pointer gestures do not arm the
  parent's drag, and child-owned native drags remain available.
- `cancel_drag_on_escape()` handles the SDK Escape action while a native drag is
  active, otherwise propagating to the normal enclosing focus scope.

Studio retains row/preview rendering, insertion-line colors, domain-specific
acceptance and collection updates, destination selection/focus policy, and tracker
formatting. Its `TransferModel::move_items` still validates and commits the two
snapshots; no domain transfer logic was moved into the SDK. `ListBoxScrollHandle`
continues to own measured reveal and configurable basic linear edge auto-scroll.

Typical host flow:

```rust
// During composition, omit dragging for ineligible or domain-restricted rows.
let drag = state.drag_keys(&key)
    .and_then(|keys| KeyedDrag::new(scope_entity_id, source_list_id, keys));
// Bind the host surface/preview with bind_drag_source and each keyed gap with
// KeyedDropTarget::bind. In the drop handler, validate and mutate domain data,
// deliver final ListBox updates, then report committed(...) or rejected(...).
```

Verification includes unchanged Studio transfer/reorder model tests and SDK
session/notification/geometry tests. An opt-in `test-support` feature enables
headless GPUI event-dispatch tests for nested children, native gap drops, outside
release, and Escape cancellation; these use TestWindow and do not launch Studio.

```sh
cargo test -p gpui-luma-core --features test-support infra::drag_drop --offline
```

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

Variable-height measurement and virtualization are independent capabilities.
Scroll snapping was evaluated and removed (see Phase 4c).

## Studio Exposition and Inspectors

The ListBox exposition is in **Selectors**, alongside the other selector
controls, using `ControlCategory::Selection`. Its description and code sample
show the state-driven composition.

The retained inspectors describe the vertical and horizontal compositions:

- inspect colors and interaction-state styling used by the exposition's row
  composition and any styled viewport;
- inspect the row metrics, spacing, padding, and viewport dimensions actually
  used by that composition;
- expose applicable parts, states, and sizes for these compositions rather
  than the legacy visual ListBox.

Inspector values must come from the same look and layout inputs used to render
the sample. Label host-owned layout values as composition settings. These
inspectors describe the Studio example, not a universal ListBox theme or an
SDK-owned viewport. Their adapters and metadata may change without preserving
legacy APIs; the inspector capability remains part of the new exposition.

## Development Phases

### Phase 1: Small Working Slice — Complete

The initial slice delivered two single-select examples in Luma Studio's
Selectors category: vertical rows and horizontal cards. They now default to extended
selection with runtime policy controls, and Phase 3 added the DnD pair. All share SDK state/input mechanics
while owning independent selection and scrolling. The delivered foundation:

1. `ListBoxState` owns a small set of typed items.
2. The bounded vertical list is 250px wide including insets and border, and
   shows five of 1,000 rows with uniform virtualization enabled.
3. The horizontal viewport contains 1,000 compact cards with 8px spacing,
   36px height, and fixed 136px width, including a leading checkmark.
   Cards never shrink to fit the pane;
   narrower viewports show fewer cards and scroll to the remaining items.
   Named card and row components keep item presentation separate from
   list state, input binding, and exposition assembly.
   Both examples require focus for wheel scrolling; unfocused wheel input
   reaches the exposition pane. Focused lists contain scrolling at endpoints.
4. The examples use `vstack!` and `hstack!`, respectively.
5. The SDK interaction binding supplies stable row IDs, list focus, clicks,
   keyboard navigation, and activation. The scroll adapter reveals keyboard
   targets through each host-owned viewport.
6. Selection goes through `ListBoxState`; owners forward the returned update.
7. Both examples render selected state and show labeled events.
8. Scoped color and layout inspectors remain available for both compositions.

The examples are manually tested in Luma Studio. Model tests cover identity
validation, selection reconciliation, and event production. The old visual
ListBox and exposition wiring have been replaced; applicable look and inspector
support remains. There is no compatibility layer.

### Phase 2: Basic Variants — Partially Complete

The Studio examples now live in independent `vertical.rs` and `horizontal.rs`
modules. Each owns its item type, item component, state, binding, layout, and
scroll handle; neither is an axis variant of a shared example view. Both
default to `SelectionMode::Extended` and display a selected-item count. A shared
SDK selector and two checkboxes change policies on both examples at runtime;
their item/selection/scroll states remain independent. The DnD pair retains
`SelectionMode::Multiple`. Flags are enabled only in applicable modes.
Each item reserves a leading Lucide checkmark slot: visible when selected and
transparent otherwise, so selection stays distinct from hover without label movement.
Selection events remain in source order; reorder-only changes do not emit a
selection event. Extended Shift-range gestures now work in snapshot order.

The SDK now supplies scroll/reveal mechanics through `ListBoxScrollHandle`.
Studio's `ExamplePresentation` supplies headings, selected-count display,
surface decoration, theme updates, and event logging. Inspector assembly stays
in the exposition module. The independent examples compose these helpers and
retain their own data and named row/card components.

- multiple toggle selection (implemented): click or Space toggles an enabled
  item, Cmd/Ctrl+A selects all enabled items, and Cmd/Ctrl+Shift+A clears selection
  while keeping list focus. Escape leaves focus through the enclosing focus scope;
  programmatic `set_selected_keys` validates and replaces selection atomically;
- fixed visible-count/row-metric helpers (implemented in `ListBoxFlow`);
  viewport-relative card sizing remains planned;
- filtered/sorted projection and active-item reconciliation (planned; active-item
  presentation and keyboard reveal already work for the full snapshot);
- extended selection and its anchor/modifier rules (implemented);
- no-selection mode, single-select toggle-off, selection-following-navigation,
  and runtime policy changes with final-state events (implemented).

The selection work in this phase is complete and confirmed working by the user.
The phase remains partially complete because viewport-relative sizing and
projections are still planned.

These variants use the same state. Filtered/sorted projection remains planned. They
do not require a visual ListBox builder.

### Phase 3: Shared Interaction Infrastructure — Extracted and User Verified

The first transfer example is a side-by-side pair under "Drag and drop" in
the ListBox exposition. Each list is fixed at 250px wide including insets and
border, and starts with ten items in a five-row viewport.
Both lists support multiple selection. Dragging a selected row captures all
selected keys; dragging an unselected row captures only that row. The preview
shows the number of items when dragging a group. A valid drop inserts the group
in source-list order at the indicated gap. A cross-list move selects only the
transferred rows in the destination, and activates and reveals the first inserted row.
Upper and lower row halves target the gaps before and after the row;
the hovered gap displays an insertion line. Targets use stable destination keys
so scrolling does not change their meaning. Dropping on unused surface space
appends the group. Both replacement snapshots and the destination's final
selection/active state are validated before source data is removed. Each list
reports its final state changes once, in the documented event order.
Same-list drops reorder the captured rows at the indicated gap, preserving
selection and the active row. The gap is adjusted for removed rows, including
when its anchor is part of the dragged group. Drops that leave the order unchanged
are no-ops. Foreign and stale drops do not change either list. Empty lists remain
drop targets; Escape cancels the active drag, and an outside release makes no
data changes. GPUI supplies the drag lifecycle; the host owns cross-list mutation.
The SDK scroll adapter provides opt-in edge auto-scroll for accepted drag payloads.
Holding the pointer inside the top or bottom edge continuously scrolls the hovered
list, with speed increasing toward the edge. The default maximum is 540 logical
pixels per second. Developers can customize each list with
`scroll.set_drag_auto_scroll_speed(720.0)`; zero disables movement, and invalid
negative or non-finite values restore the default. Measured bounds and limits determine
the scroll extent; leaving the edge, reaching an endpoint, dropping, or cancelling
stops scrolling. Gap targets update as rows scroll under a stationary pointer.
The same helper supports horizontal viewports. The exposition retains the
independent vertical and horizontal examples.

This is a **basic auto-scrolling function**: speed scales linearly with pointer
depth into the edge zone, and displacement is speed multiplied by elapsed frame
time. There is no temporal easing or inertia. The linear behavior is sufficient
for now. A future refinement should allow developers to supply a speed/easing
function while retaining the shared frame scheduling, cancellation, and bounds
handling. Only maximum speed is currently configurable.

Completed extraction:

- selection-to-drag capture in ListBox state;
- shared keyed sessions, gap targets, and host mutation proposals;
- typed notifications with shared completion/cancellation guards;
- nested interactive-child boundaries with headless dispatch regression coverage;
- Studio migrated to the SDK helpers while retaining domain mutation and visuals.

The extracted Studio wiring and subsequent migrated control/look-owned builder
are confirmed working by the user. The macro stays local for review.
A customizable auto-scroll response function remains an optional future extension.

### Phase 4: Uniform Virtualization — Implemented; Awaiting User Testing

Uniform-item virtualization is opt-in on `ListBoxControl` and `ListBoxBuilder`:

```rust
.virtualization(ListBoxVirtualization::Uniform { overscan: 2 })
```

`ListBoxVirtualization::Eager` remains the default. The existing scroll adapter is
extended; no separate public virtual-scroll-view control is introduced. The
implementation supports fixed-height vertical rows and fixed-width horizontal
cards, including spacing and viewport insets.

- The SDK computes the viewport range plus a configurable item buffer on each
  side. Leading/trailing spacers retain the full collection's scroll extent.
  Invalid uniform geometry falls back to eager rendering.
- The look-owned builder constructs templates, interaction bindings, and themed
  rows only for that range. Range iteration skips directly into the snapshot.
  Shadcn appearance remains entirely in the look.
- Templates remain ordinary render-time callbacks, including borrowed local
  captures. Durable state for interactive item content belongs in host models
  or retained entities; off-screen element instances are not retained.
- Selection, active keys, and range anchors still address the entire collection.
  Keyboard reveal calculates an off-screen item's position without requiring its
  element to exist. Resize, collection shrink, empty lists, focus-required wheel
  routing, and endpoint containment retain their existing semantics.
- DnD gaps resolve against the full collection, including the key after the last
  rendered row. Group payloads survive source-row unmounting; drag auto-scroll
  refreshes the render range and hit targets even in an unfocused destination.
- The same GPUI scroll handle retains position. Actual viewport measurements
  refine the range on the next scheduled frame at first layout and resizing;
  normal wheel updates construct the new range using the updated offset.

The existing vertical and horizontal exposition examples now each contain 1,000
items and opt into a two-item buffer. Their markup, content templates, dimensions,
selection controls, and inspectors are unchanged. The DnD pair also opts in while
retaining ten initial items per list and five visible rows.

A dedicated **Spectrum** example adds 10,000 typed color items in a 250px-wide,
five-row viewport. Its named content template combines a five-band color swatch,
a numbered label, hue/saturation metadata, and the selection checkmark. It uses
extended selection and focus-required scrolling, with Home/End navigation for
quick checks at both ends of the collection. The local markup macro and SDK
virtualization API are reused without additional scrolling logic.

The Spectrum example also shows live collection size, visible range (including
partial rows), constructed range (including the buffer), and actual template
calls for that render. The SDK scroll handle exposes a read-only snapshot via
`rendered_window()` after composition. Diagnostics reuse existing geometry and
rendering; they add no input listeners, event-tracker entries, or frame requests.
Headless tests verify the readout through aligned and partial-row wheel scrolling
and Home/End navigation. Manual validation of the readout remains pending.

Headless checks cover bounded template construction with 10,000 items on both
axes, off-screen extended selection/reveal, disabled-item navigation, resizing,
large wheel jumps, focus loss and endpoints, snapshot shrink/empty/repopulation,
same-list and cross-list group gap drops after rows leave the render range, and
auto-scroll mounting new destination targets without focus. Geometry unit tests
also cover 100,000 items, exact spacer extents, invalid inputs, and buffer overflow.
Verification passes 45 SDK ListBox tests, nine look tests, 19 Studio ListBox tests,
and two SDK doctests, plus Clippy across all three crates and workspace formatting.
Studio has not been launched by the agent; manual testing remains with the user.

Still deferred:

- horizontal content-sized width virtualization (vertical heights are implemented below);
- filtered/sorted projections (next priority);
- data paging: collection snapshots remain fully resident, and full-collection
  selection/transfer operations retain their existing costs.

### Phase 4b: Content-Sized Vertical Rows — Implemented; User Verified

Item templates determine height, and GPUI measures their outer row bounds at
layout time. `ListBoxFlow::VerticalContent { viewport_height, gap }` supplies an
explicit viewport height; it does not promise a fixed count of unequal rows.
The look-owned builder also exposes `.content_sized()`. Content-sized surfaces
fill the available width but do not force `h_full()` or a fixed row height.
Templates should use natural height; empty rows have a 1px minimum for progress.

The local markup keeps this choice next to the template:

```rust
scroll_view! { vertical;
    viewport_height = 320.0;
    vstack! {
        gap = 4.0;
        item_height = content;
        item_template = |model, _cx| note_template(model, look, expanded);
    }
}
```

- **Eager:** construct all rows, measure their actual heights, and use measured
  geometry for navigation/reveal and diagnostics.
- **Virtualized:** opt into
  `ListBoxVirtualization::Measured { estimated_height: 100.0, overscan: 2 }`.
  Offscreen rows use estimates until constructed. This remains rendering
  virtualization over a fully resident collection, without data fetching.
- The same scroll adapter retains a key-indexed height cache and a prefix-sum
  index with logarithmic height updates and offset lookup. Snapshot identity
  avoids scanning the whole collection on ordinary renders. Snapshot replacement
  rebuilds order and treats retained heights as estimates, since content under
  the same key may have changed. Removed keys are discarded.
- Actual bounds are read through the existing layout observer and scroll handle.
  No new mouse/hover listeners or per-item events are introduced. A corrective
  frame is requested only for changed measurements, geometry, or viewport size.
- Measurement corrections retain the top row key and its pixel offset. Pending
  keyboard reveal is resolved against the target's measured height. Rows taller
  than the viewport align their top; estimates and spacer extents converge as
  rows are measured. Width changes automatically invalidate cached measurements.
- Hosts call `control.invalidate_measurements(Some(&key), cx)` for external
  content changes or `None` for all rows, including typography/theme changes.
  This is unnecessary for snapshot replacement. Mounted rows are remeasured
  whenever they render; stale offscreen values remain estimates until revisited.
- DnD halves use parent-relative bounds for content-sized rows, preserving keyed
  before/after gaps and the existing auto-scroll adapter. Selection and domain
  transfer rules are unchanged.
- `rendered_window()` includes `measured_items` for content-sized flow, including
  eager mode. The uniform fixed-size path retains its existing arithmetic.
  Incompatible sizing/virtualization combinations fall back to eager composition;
  nonpositive/nonfinite height estimates use 48px.

Studio adds **Variable-height items** with two independently scrolling 1,000-note
lists using the same named template: eager and virtualized. The template combines
a selection mark, title, and naturally wrapping description. SDK checkboxes
expand details and narrow both lists from 250px to 180px. Both display collection
size, actual template calls, visible/constructed counts, and measured/estimated
counts. Shadcn appearance stays in the look; markup stays local.

Headless coverage checks real wrapping content in both modes, bounded virtual
construction, focus-required wheel routing, resize anchoring, Home/End and Return,
content expansion beyond the viewport and collapse at the collection end,
snapshot shrink/empty/reorder, settled
measurement frames, unequal-height DnD halves, and drag auto-scroll without focus.
The actual Studio comparison is also rendered headlessly to verify its template,
counts, and expansion/width subscriptions. The user confirmed this working locally;
the agent has not launched Studio. Verification for Phase 4b passed 48 SDK ListBox tests,
12 look tests, 20 Studio ListBox tests, Clippy across all three crates, and
workspace formatting.

### Phase 4c: Scroll Snapping — Removed

Removed at the user's request after local testing. The SDK snapping API,
settling timers/animation, alignment geometry, Studio selectors, and snapping-only
tests have been deleted. Normal wheel scrolling, keyboard reveal, focus routing,
drag auto-scroll, and user-verified variable-height support remain.

Filtering/view projections are the next planned work. Scroll snapping is no longer
part of the active implementation plan. SwiftUI examples above remain historical
layout references, not a promise of snapping support.

### Programmatic Item Positioning — Implemented; Smooth Variants Awaiting User Testing

The host explicitly chooses the item and when to move the viewport:

```rust
list.scroll_to(item_key, cx);        // Make this item visible.
list.scroll_to_center(item_key, cx); // Center this item in the viewport.

// Animate the same positioning requests.
list.scroll_to_smooth(item_key, cx);
list.scroll_to_center_smooth(item_key, cx);
```

All four methods are available on `ListBoxControl` and on the lower-level
`ListBoxScrollHandle`. They accept a stable item key, not a row index. Requests
are resolved against the current collection during layout; the latest scroll
request wins, and unknown or removed keys are ignored. They work without focus
and do not change selection, active item, or focus, or emit selection events.

`scroll_to` leaves fully visible items in place. An item larger than the viewport
is positioned with its top (vertical) or left edge (horizontal) visible.
`scroll_to_center` places the requested item's midpoint at the viewport midpoint
on the scrolling axis, including oversized items. Both clamp at collection
boundaries; exact centering near the beginning/end may therefore be impossible.

Fixed eager and virtualized lists support both axes. Content-sized vertical lists
use measured heights, first locating off-screen rows from estimates and correcting
placement when their actual heights arrive. Once completed, a request has no effect
on subsequent scrolling. The original methods remain immediate; the smooth
variants use the shared scrolling layer's frame-driven 200ms ease-out animation.
The shared `ScrollContainer` also exposes `set_vertical_offset_smooth(value, cx)`.
Animation follows measured destination corrections and clamps to current content
bounds. New positioning requests or user interaction interrupt it. This introduces
no snapping policy. Keyboard navigation retains its existing bring-into-view behavior.

Studio's **Variable-height items** comparison includes a **Target item** selector
(Note 1, 7, 50, 500, or 1000), **Make visible**, and **Center in viewport** buttons.
Both buttons apply to the eager and virtualized lists without changing selection.
The **Smooth scrolling** checkbox (off by default) chooses animated positioning
for both buttons. The user confirmed immediate positioning working locally;
the smooth variants await local testing.
Use the existing expansion and width controls to repeat the test after reflow.
The fixture uses SDK selectors/buttons with Shadcn styling, and its subscriptions
are covered by a headless test targeting off-screen notes in both lists.

Headless tests cover requests before first layout, unfocused positioning, unchanged
selection, fixed rows on both axes, measured off-screen and oversized rows, endpoint
clamping, missing keys, reordering, repeated requests, and scrolling afterward.
Smooth scrolling coverage checks intermediate offsets, exact final placement,
off-screen measurement corrections, request replacement, and wheel interruption.
The shared scroll container has separate clamping and interruption coverage.
Verification passed 85 ListBox tests across SDK, look, and Studio, plus 9 scroll
container tests, Clippy across all three crates, and workspace formatting.
The agent has not launched Studio.

## Explicit Non-Goals

The initial design does not require:

- a mandatory SDK visual shell (optional composition is provided by the look-owned builder);
- a default row renderer;
- a general-purpose injectable shell framework;
- an SDK-owned viewport or stack;
- a universal styling preset for arbitrary domain values;
- backward compatibility with the legacy visual ListBox API.

The legacy visual ListBox is removed, not retained as an optional shell for the
new model. Existing legacy APIs do not constrain naming or behavior here.

Repeated DnD composition justified the optional look-owned builder and local
markup adapter described above. These compose the existing behavior model without
replacing it or requiring other consumers to adopt a visual shell.

## Acceptance Criteria

Implemented composition criteria:

- the Luma Studio exposition appears in Selectors and visibly shows a scrolling
  vertical list;
- scoped inspectors remain available and reflect the new composition's actual
  styling and layout inputs;
- the list is built from `ListBoxState.visible_items()`;
- rows are composed by the host using `vstack!`;
- clicking a row updates selection through `ListBoxState` and changes its visual state;
- a `ListBoxEvent::SelectionChanged` appears in the event stream;
- the same state model feeds the independent horizontal `hstack!` composition;
- the SDK does not need to know what an arbitrary item looks like.

Implemented behavioral checks:

- duplicate snapshot keys and invalid selection/active replacements fail without
  partial mutation; repeated programmatic selection keys are deduplicated;
- snapshot replacement preserves stable-key selection and reconciles removal or
  disabled items, including empty and all-disabled collections;
- repeated clicks, programmatic updates, and reconciliation emit exactly the
  documented events, with no duplicate change events for no-op operations;
- keyboard input respects focus and nested controls, and reveals the active row
  through the host viewport in both vertical and horizontal compositions;
- in-repository consumers compile against the replacement, with no dependency
  on removed visual ListBox APIs; retained inspectors use the new composition's
  look and layout inputs.

Selection acceptance checks (unit-tested; user confirmed Studio behavior working):

- Shift ranges expand, contract, reverse, and skip disabled rows on either axis;
- Cmd/Ctrl toggles and additive ranges preserve other selections;
- runtime mode changes reconcile once and update keyboard handling;
- no-selection mode supports active navigation/activation without selecting;
- toggle-off never violates required selection; navigation-following is opt-in;
- range anchors survive reorder and reconcile removal/disable or explicit replacement.

Pending acceptance checks for planned features:

- invalid projections fail without partial mutation;
- filtering preserves hidden selection while reconciling active item and anchor;
- sorted/filtered range selection follows projected order;

Completed DnD extraction acceptance checks (automated; user confirmed Studio behavior working):

- selected-group capture is stable and excludes disabled/unknown drag origins;
- foreign/inactive sessions are rejected, and terminal notifications occur once;
- native row drags reach keyed before/after targets; outside release cancels;
- Escape cancels a drag and otherwise retains ordinary focus-scope behavior;
- nested child clicks/pointer gestures do not select or drag the row, while
  child-owned native drags remain functional;
- existing single/group transfers, reorders, no-op drops, and atomic rejection
  tests still pass with host-owned collection mutation.
