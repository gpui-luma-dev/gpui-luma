# Issue #5: Drag and Drop Design for SDK Sortable Collections

## Description

Applications need a first-class SDK way to drag and drop items within a collection and between nearby collections without inventing app-local gesture code, preview layers, or drop targeting logic.

The immediate working example is a collection of small cards or panels that can be dragged:
1. Within the same row to reorder.
2. Between rows to relocate.
3. With a modern floating drag preview rather than only moving the live source element.

This should land as an SDK control family, not as a one-off app behavior. It must follow the normal SDK boundaries:
1. **Model** for static configuration.
2. **Control** for live drag state and event handling.
3. **Template** for row, item, placeholder, and preview rendering.
4. **Theme** for all visual treatment such as hover, insertion, lift, opacity, and shadows.

The SDK should own interaction state and hit testing, while the application continues to own the actual data model and decides whether a proposed move is accepted.

---

## Design Goals

### Functional Goals

* Reorder items within a row.
* Move items between rows in the same collection surface.
* Emit semantic events instead of mutating app state directly.
* Support disabled items, drop restrictions, and rejected targets.
* Work with card-like content, not only plain text rows.

### UX Goals

* Do not start dragging immediately on pointer down; preserve normal click behavior via an activation threshold.
* Keep the source location visible with a placeholder while dragging.
* Show a floating preview that follows the pointer and stays theme-correct.
* Show a clear insertion affordance at the current drop location.
* Leave room for auto-scroll near container edges.

### Architectural Goals

* Do not turn `list_view` or `selection_panel` into a catch-all drag framework.
* Reuse existing GPUI drag payload mechanics already used by controls such as splitters and sliders.
* Reuse existing overlay/deferred window-host patterns already present in popup and overlay controls.
* Keep the first SDK abstraction narrow: sortable collections of items grouped by rows.

---

## Why a Dedicated Sortable Collection Control

The repo already has useful ingredients:

* Lightweight drag payloads and `on_drag` / `on_drag_move` gesture handling in controls such as dock splitters and sliders.
* Deferred overlay rendering in popup and overlay-window controls.
* Card templating and look resolution in the card control family.

Those ingredients are necessary but not sufficient. Reordering cards across rows needs additional behavior that does not belong inside existing controls:

* Source/target identity tracking.
* Drop index resolution.
* Placeholder management.
* Preview rendering.
* Drop acceptance rules.
* Cross-row move semantics.

That is enough surface area to justify a dedicated control family instead of scattering drag state across apps.

---

## Proposed Control Family

Create a new control family under:

```text
crates/sdk/src/controls/sortable_collection/
  mod.rs
  model.rs
  control.rs
  template.rs
  theme.rs
```

This family should represent a collection composed of rows, where each row contains ordered items.

### Public Entry Point

```rust
pub type SortableCollection<TItem, TRowMeta = ()> = Entity<SortableCollectionControl<TItem, TRowMeta>>;
```

### Builder

```rust
SortableCollection::new(id)
```

The builder should configure:

* Rows and row metadata.
* Items per row.
* Item identity extraction.
* Drag eligibility.
* Drop eligibility.
* Item template.
* Placeholder template.
* Drag preview template.

---

## Data Model Shape

The first version should be explicit and typed around rows and items.

### Identity Types

```rust
pub type DragItemId = SharedString;
pub type DropRowId = SharedString;
pub type DropContainerId = SharedString;
```

If later we want generic typed ids, that can be layered on after the core behavior proves out.

### Row Model

```rust
pub struct SortableRow<TItem, TRowMeta = ()> {
    pub id: DropRowId,
    pub meta: TRowMeta,
    pub items: Vec<TItem>,
}
```

### Collection Model

```rust
pub struct SortableCollectionModel<TItem, TRowMeta = ()> {
    pub id: DropContainerId,
    pub rows: Vec<SortableRow<TItem, TRowMeta>>,
    pub size: ControlSize,
    pub enabled: bool,
    pub activation_distance_px: f32,
    pub template: Arc<dyn SortableCollectionTemplate<TItem, TRowMeta>>,
    pub theme: Arc<dyn SortableCollectionTheme>,
}
```

### App-Supplied Policy Hooks

```rust
pub type CanDragFn<TItem, TRowMeta> =
    Arc<dyn Fn(&DragContext<'_, TItem, TRowMeta>) -> bool + Send + Sync>;

pub type CanDropFn<TItem, TRowMeta> =
    Arc<dyn Fn(&DropProposal<'_, TItem, TRowMeta>) -> bool + Send + Sync>;
```

The app stays in charge of domain rules:

* Whether an item may be dragged.
* Whether a target row may accept that item.
* Whether a proposed insertion index is valid.

The SDK should not infer domain legality from layout alone.

---

## Runtime State

The control owns only transient UI state.

### Drag Session

```rust
pub struct DragSession {
    pub item_id: DragItemId,
    pub source_row_id: DropRowId,
    pub source_index: usize,
    pub pointer_origin: Point<Pixels>,
    pub pointer_current: Point<Pixels>,
    pub preview_offset: Point<Pixels>,
}
```

### Active Drop Location

```rust
pub struct DropLocation {
    pub row_id: DropRowId,
    pub index: usize,
}
```

### Control State

```rust
pub struct SortableCollectionState {
    pub pressed_item: Option<PressedItemState>,
    pub drag_session: Option<DragSession>,
    pub active_drop: Option<DropLocation>,
    pub hovered_row: Option<DropRowId>,
}
```

Important boundary: this state does not own the reordered item list. The app still applies the reorder after receiving a semantic drop event.

---

## Event Model

The control should emit semantic events rather than directly editing the collection data.

```rust
pub enum SortableCollectionEvent {
    DragStarted {
        item_id: DragItemId,
        source_row_id: DropRowId,
        source_index: usize,
    },
    DragHovered {
        location: Option<DropLocation>,
    },
    DropCommitted {
        item_id: DragItemId,
        source_row_id: DropRowId,
        source_index: usize,
        target_row_id: DropRowId,
        target_index: usize,
    },
    DragCancelled,
}
```

This mirrors the rest of the SDK event philosophy:

* Controls own gesture handling.
* Controls emit semantic events.
* Parent entities subscribe and update app state.

---

## Gesture Lifecycle

### 1. Press

Pointer down on a drag handle records:

* Source row id.
* Source index.
* Pointer origin.
* Candidate dragged item id.

At this point the control is still in a pressed state, not a drag session.

### 2. Activation Threshold

Drag begins only after the pointer moves beyond a small threshold such as `activation_distance_px = 4.0` or `6.0`.

This is important because card surfaces often contain clickable content. Without a threshold, drag and click semantics fight each other.

### 3. Active Drag

Once the threshold is crossed:

* Emit `DragStarted`.
* Promote the pressed state into a `DragSession`.
* Render the source item as lifted/ghosted in place.
* Render a placeholder at the source slot until a target slot is resolved.
* Render a floating drag preview in an overlay layer.
* Continuously recompute `active_drop`.

### 4. Hover and Target Resolution

While dragging:

* Resolve the nearest row under or nearest to the pointer.
* Resolve insertion index from item slot bounds within that row.
* Validate the proposal through `can_drop`.
* Update visual insertion affordances and emit `DragHovered`.

### 5. Commit or Cancel

On mouse up:

* If `active_drop` is valid, emit `DropCommitted`.
* If not valid, emit `DragCancelled`.
* Clear drag session, placeholder state, and preview layer.

---

## Drop Target Resolution

This is the core functional problem and should be an explicit subsystem, not buried in templates.

### Drop Registry

The control should maintain a registry of measured geometry for:

* Row bounds.
* Item bounds.
* Optional leading and trailing insertion slots.

Conceptually:

```rust
pub struct DropRegistry {
    pub rows: Vec<RowDropGeometry>,
}
```

Each row entry should contain:

* Row id.
* Row bounds.
* Ordered item slot bounds.
* Optional content direction metadata if horizontal and vertical rows both become necessary later.

### Resolution Strategy

For the initial cards-in-rows case:

* Determine the best row from pointer location.
* Within that row, resolve the insertion index by comparing pointer position to item midpoints.

For a horizontal row of cards:

* Pointer before first midpoint => index `0`
* Pointer between item `i` and `i + 1` => index `i + 1`
* Pointer after last midpoint => append at end

This gives a simple, modern insertion model without needing collision-heavy geometry tricks.

---

## Modern Drag Preview Design

This is the most important UX requirement and should be handled as a first-class SDK feature.

### Principle

The drag preview should not be the live source card itself. It should be a separate render artifact driven by a read-only preview model.

Reasons:

* The source slot still needs to exist for layout stability and placeholder rendering.
* The preview often needs different styling such as scale, opacity, elevation, and pointer offset.
* The preview should render in a higher overlay layer, not inside the normal row flow.
* Reusing the live entity would couple drag behavior to ordinary card lifecycle and focus behavior.

### Preview Layer

Implement a dedicated floating preview layer rendered through the existing deferred overlay pattern already used elsewhere in the repo.

The layer should:

* Render above the collection.
* Follow the pointer smoothly.
* Ignore hit testing.
* Stay theme-correct in light and dark modes.

This should be conceptually similar to the repo's popup and overlay-host approach, but specialized for drag previews rather than menus or dialog content.

### Preview Template

```rust
pub trait DragPreviewTemplate<TItem, TRowMeta>: Send + Sync {
    fn render(
        &self,
        model: &DragPreviewRenderModel<'_, TItem, TRowMeta>,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}
```

### Preview Render Model

```rust
pub struct DragPreviewRenderModel<'a, TItem, TRowMeta> {
    pub item: &'a TItem,
    pub item_id: &'a DragItemId,
    pub source_row_id: &'a DropRowId,
    pub source_index: usize,
    pub pointer_position: Point<Pixels>,
    pub preview_position: Point<Pixels>,
    pub accepted: bool,
    pub size: ControlSize,
    pub row_meta: Option<&'a TRowMeta>,
}
```

### Default Preview Behavior

The default SDK preview should:

* Reuse the same content shape as the card/item template when possible.
* Apply a stronger shadow.
* Optionally scale to around `1.02` to `1.04`.
* Apply slight opacity reduction if the current target is rejected.
* Offset from the pointer so the drop indicator remains visible beneath it.

The preview should look intentional and lifted, not like a screenshot clone with no state.

---

## Placeholder and Insertion Affordances

The dragged source item should leave behind a placeholder instead of collapsing immediately.

### Placeholder

The placeholder should preserve:

* Width
* Height
* Border radius
* Row spacing

This avoids layout jitter while dragging.

It can render as:

* A transparent spacer.
* A faint dashed card shell.
* A themed empty slot.

That visual choice belongs in theme/template, not control logic.

### Insertion Affordance

The target position should also be visible independently of the floating preview. For example:

* A vertical insertion line between cards.
* A glowing slot gap.
* A placeholder card snapping into the predicted target index.

For rows of small cards, the best first approach is a real placeholder card at the target index. It is easier to read than a thin rule when items have varied widths.

---

## Template Architecture

The collection needs separate template seams for different parts of the experience.

### Collection Template

```rust
pub trait SortableCollectionTemplate<TItem, TRowMeta>: Send + Sync {
    fn render(
        &self,
        model: &SortableCollectionRenderModel<'_, TItem, TRowMeta>,
        handlers: SortableCollectionTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}
```

### Sub-Templates

The collection template may delegate to:

* `row_template`
* `item_template`
* `placeholder_template`
* `preview_template`

This mirrors the rest of the SDK's template composition style. The control should compute state and identity. The template decides how those states render.

### Item Template Guidance

For the user's first use case, item templates should compose normal SDK card controls or card-like templates rather than custom app-local raw `div` chrome.

That keeps the design aligned with the SDK/app boundary already documented in the architecture guide.

---

## Theme Responsibilities

The theme should define semantic drag states rather than hardcoded visual constants in the control.

### Suggested Theme Surface

```rust
pub struct SortableCollectionLook {
    pub row_gap: f32,
    pub item_gap: f32,
    pub lift_shadow: gpui::BoxShadow,
    pub preview_scale: f32,
    pub source_opacity: f32,
    pub placeholder_border: Hsla,
    pub placeholder_background: Hsla,
    pub insertion_emphasis: Hsla,
    pub rejected_opacity: f32,
}
```

The exact fields can evolve, but these states should not live in `control.rs`.

---

## Accessibility and Input Policy

The first implementation can be pointer-first, but the design should not block later accessibility work.

Future capabilities should include:

* Keyboard reorder within a row.
* Keyboard move between rows.
* Focus-visible drag handles.
* Announced source and target positions.

These do not need to ship in MVP, but the event model and template structure should leave room for them.

---

## Auto-Scroll Strategy

Dragging near the edges of a scrollable container should eventually auto-scroll the active row or collection viewport.

This should be an explicit subsystem such as:

```rust
pub struct AutoScrollController { ... }
```

It should be deferred until after the base cards-in-rows interaction is solid. The first pass can assume fully visible rows if that keeps the control smaller and reviewable.

---

## What This Should Not Do Initially

To keep the first version tight, it should not try to solve:

* Arbitrary cross-window drag and drop.
* OS-native file drags.
* Copy/link/alias semantics.
* Multi-select drag.
* Nested tree drag/drop.
* Freeform canvas movement.

Those are different problems and would distort the control shape too early.

---

## Recommended MVP

### Phase 1: Sortable Rows of Cards

Support:

* Multiple rows in one collection.
* Reorder within a row.
* Move between rows.
* Activation threshold.
* Floating preview.
* Source placeholder.
* Target placeholder or insertion slot.
* Semantic event emission.

Do not include:

* Auto-scroll.
* Keyboard drag/reorder.
* Cross-collection transfer.

### Why This Scope

This directly serves the intended working example while exercising every meaningful seam:

* Typed identity
* Per-row drop resolution
* Preview layer
* Placeholder behavior
* Theme-aware visuals
* App-owned reorder state

If this shape feels clean in Gallery, it can later become the base for richer drag/drop behavior.

---

## Implementation Plan

### Phase 1: SDK Core

- [ ] Add `crates/sdk/src/controls/sortable_collection/mod.rs`.
- [ ] Implement typed row/item models and builder configuration in `model.rs`.
- [ ] Implement transient drag session state and semantic event emission in `control.rs`.
- [ ] Implement row/item/placeholder composition in `template.rs`.
- [ ] Implement theme resolution in `theme.rs`.
- [ ] Export the control from `crates/sdk/src/controls/mod.rs`.

### Phase 2: Preview Layer

- [ ] Add a deferred floating preview layer owned by the control.
- [ ] Implement default `DragPreviewTemplate`.
- [ ] Ensure preview uses theme-aware shadow, scale, and accepted/rejected states.
- [ ] Keep preview hit testing disabled.

### Phase 3: Gallery Validation

- [ ] Add a Gallery pane showing several rows of small cards/panels.
- [ ] Include same-row reorder and cross-row moves.
- [ ] Keep one pane focused on drag/drop behavior only.
- [ ] Surface diagnostics such as current source row, target row, and target index during development if needed.

---

## Verification Plan

### Automated Tests

- [ ] Activation threshold does not start a drag on simple click.
- [ ] Drag session starts after threshold is crossed.
- [ ] Drop resolution within a row returns the correct insertion index.
- [ ] Cross-row drop resolution returns the correct row and index.
- [ ] Rejected targets do not emit `DropCommitted`.
- [ ] Valid drops emit `DropCommitted` with the expected source and target ids.
- [ ] Cancelling clears drag session and active placeholder state.

### Manual Verification

- [ ] Open the Gallery drag/drop pane.
- [ ] Drag a card within a row and verify the placeholder and final reorder.
- [ ] Drag a card to a different row and verify the target placeholder updates continuously.
- [ ] Verify the floating preview stays above the collection and matches the active theme.
- [ ] Verify light/dark mode changes still render the preview and placeholder correctly.
- [ ] Verify clicking a card without crossing the threshold still behaves as a normal click.
- [ ] Verify rejected targets render a visibly invalid state and do not commit.

---

## Key Architectural Decision

The important decision is to treat drag preview as its own overlay-rendered template artifact rather than as the live dragged item.

That gives us:

* Stable layout at the source slot.
* Cleaner control/template separation.
* Theme-correct preview rendering.
* Room for modern lift, shadow, and insertion behavior.
* A path to richer drag/drop later without turning existing list or card controls into a monolith.
