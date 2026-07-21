# Issue #1: Reactive Flex Layout for Card Surfaces

## Description

Luma Studio's current Cards tab is an absolute-positioned demo board. That is useful for manual arrangement, but it is the wrong behavior for the "tweakcn-style" card gallery use case where cards should react to the available horizontal space.

The target behavior is:

* Cards fill each row horizontally.
* Cards shrink and grow within a sensible width range.
* Rows reflow automatically as the work panel width changes.
* The layout should remain lookless and reusable across SDK consumers.

This is a strong case for a real SDK flex layout surface rather than only `hstack!` / `vstack!` helpers.

---

## Desired Behavior

Given a container width:

* The layout places items left-to-right.
* Items wrap onto new rows when they cannot remain above their minimum width.
* Items on a row expand to consume leftover horizontal space.
* Each card respects a configured width policy:
  * preferred width or flex basis,
  * minimum width,
  * maximum width,
  * grow / shrink behavior.

This is not "3 columns, then 2, then 1" as a hard grid rule. The column count is an emergent result of the available width and each item's size constraints.

---

## Why Flex, Not Grid, For This Seam

Grid is a better fit when the app wants explicit tracks.

This seam is different:

* The user expectation is "fill the row."
* Cards should react continuously to width changes, not only at breakpoints.
* The number of cards per row should be derived from available width.
* Some card families may want slightly different preferred widths while still participating in the same flowing gallery.

That points to wrapped flex layout with item basis, grow, shrink, and min/max width constraints.

---

## Why The Current Cards Tab Is Not This

The live Luma Studio Cards tab currently renders an absolute-positioned board:

* [`apps/luma-studio/src/studio/content/board.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/apps/luma-studio/src/studio/content/board.rs)
* [`apps/luma-studio/src/studio/panel_layout.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/apps/luma-studio/src/studio/panel_layout.rs)
* [`apps/luma-studio/src/studio/panel_layout_config.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/apps/luma-studio/src/studio/panel_layout_config.rs)

That surface persists explicit coordinates and is effectively a draggable board mode.

The reactive gallery layout should be treated as a separate layout mode or a separate surface, not as a small tweak to the current persisted-position board.

---

## Proposed SDK Shape

Add a lookless layout primitive under the SDK layout surface:

```text
crates/sdk/src/layouts/flex_layout.rs
```

Export it from:

* `crates/sdk/src/layout.rs`
* optionally `crates/sdk/src/macros.rs` if we later want macro sugar

The primitive should stay layout-only. It does not need LMTP control structure unless we later add interaction, persistence, or measurement callbacks. For now this is a plain reusable layout component/builder.

---

## Minimal Public API Sketch

### Container

```rust
pub enum FlexDirection {
    Row,
    Column,
}

pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

pub struct FlexLayout {
    direction: FlexDirection,
    wrap: FlexWrap,
    gap_x: f32,
    gap_y: f32,
    align_items: Option<FlexAlign>,
    justify_content: Option<FlexJustify>,
    children: Vec<FlexChild>,
}
```

### Child Sizing Policy

```rust
pub struct FlexItemSize {
    pub basis: Option<f32>,
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub grow: f32,
    pub shrink: f32,
}

pub struct FlexChild {
    pub element: AnyElement,
    pub size: FlexItemSize,
    pub align_self: Option<FlexAlign>,
}
```

### Builder Surface

```rust
FlexLayout::new()
    .direction(FlexDirection::Row)
    .wrap(FlexWrap::Wrap)
    .gap(16.0)
    .child(
        card,
        FlexItemSize {
            basis: Some(320.0),
            min_width: Some(260.0),
            max_width: Some(420.0),
            min_height: None,
            max_height: None,
            grow: 1.0,
            shrink: 1.0,
        },
    )
```

The key capability is not the container row/column toggle. The key capability is item sizing policy.

---

## Convenience Layer For Card Galleries

If the raw flex builder feels too low-level, add a narrow helper on top rather than bloating the base layout primitive.

Example:

```rust
pub struct ReactiveCardFlow {
    item_min_width: f32,
    item_preferred_width: f32,
    item_max_width: f32,
    gap_x: f32,
    gap_y: f32,
}
```

This would compile to `FlexLayout`, not replace it.

That keeps the base primitive general while still making the Luma Studio and Gallery use case easy to express.

---

## Example Target Usage

```rust
FlexLayout::new()
    .direction(FlexDirection::Row)
    .wrap(FlexWrap::Wrap)
    .gap_x(16.0)
    .gap_y(16.0)
    .children(cards.into_iter().map(|card| {
        FlexChild::new(card).size(FlexItemSize::card_fill(
            320.0, // preferred
            260.0, // min
            420.0, // max
        ))
    }))
```

Intended result:

* wide panel: more cards per row, cards grow toward preferred/max width
* medium panel: fewer cards per row, cards shrink as needed
* narrow panel: one card per row once min width forces wrap

---

## Sizing Authoring Model

In practice, developers usually do hand-author these dimensions at first, but they should not keep doing that card-by-card forever.

The normal pattern is:

* start with a few hand-tuned `min / preferred / max` ranges,
* resize the host surface until wrapping feels right,
* collapse repeated values into named presets,
* keep one-off values only for true outliers.

So the SDK should support per-item sizing, but app code should usually converge on a small width vocabulary rather than a long list of bespoke numbers.

### Expected Preset Shape

```rust
pub enum CardWidthClass {
    Compact,
    Normal,
    Wide,
}

impl CardWidthClass {
    pub fn size_policy(self) -> FlexItemSize {
        match self {
            Self::Compact => FlexItemSize::card_fill(340.0, 300.0, 380.0),
            Self::Normal => FlexItemSize::card_fill(380.0, 320.0, 420.0),
            Self::Wide => FlexItemSize::card_fill(560.0, 420.0, 760.0),
        }
    }
}
```

### Likely Mapping For Current Luma Studio Cards

Based on the current board widths in [`apps/luma-studio/src/studio/panel_layout.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/apps/luma-studio/src/studio/panel_layout.rs):

* `Compact`
  * `CreateAccount` (`340`)
  * `Chat` (`360`)
  * `TreeView` (`360`)
* `Normal`
  * `UpgradeSubscription` (`380`)
  * `TeamMembers` (`380`)
  * `CookieSettings` (`380`)
  * `ReportIssue` (`380`)
  * `ShareDocument` (`380`)
  * `DatePickerRange` (`380`)
  * `Accordion` (`380`)
  * `SystemPreferences` (`380`)
* `Wide`
  * `Payments` (`720`)

The exact preset values may move a bit once the layout is live, but the key point is that most cards should share a small set of policies.

That keeps:

* the primitive flexible,
* the app code readable,
* the tuning effort manageable,
* the visual rhythm more coherent across the gallery surface.

---

## Scope Guardrails

This should not become a giant Box clone or CSS translation layer.

Keep v1 focused on:

* direction
* wrap
* gap / gap_x / gap_y
* align / justify
* per-child basis
* per-child grow / shrink
* per-child min / max width
* optional per-child min / max height

Avoid in v1:

* responsive breakpoint syntax
* margin/padding/position/inset kitchen sink props
* look-owned defaults hidden inside the layout primitive
* mixing this with persisted draggable board behavior

---

## Luma Studio Implication

Luma Studio likely wants two distinct card surfaces:

1. **Board mode**
   Persisted positions, drag arrangement, explicit card placement.

2. **Reactive gallery mode**
   Wrapped flex fill layout for seeing how cards behave as a responsive surface.

Trying to make one surface serve both jobs will blur the seam and produce awkward behavior.

---

## Verification Plan

### Manual Verification

Add a temporary Luma Studio or Gallery surface showing:

* 12 cards with shared min / preferred / max widths
* live resizing of the host pane
* clear transitions from many-per-row to single-column

Confirm:

* rows always fill horizontally
* cards do not collapse below minimum width
* cards do not stretch past maximum width
* wrapping feels continuous rather than breakpoint-snapped

### Automated Verification

Add small builder/layout tests for:

* flex child metadata preservation
* no-panic rendering with wrap enabled
* size policy propagation to child wrappers

---

## Recommendation

For the tweakcn-style card surface, the SDK should add a real lookless flex layout primitive centered on wrapped row flow and item sizing policy.

Do not frame this as "we need a Flex because web users know Flex."

Frame it as:

* Luma Studio has a real reactive card-fill layout use case.
* Wrapped flex with width constraints is the correct behavioral seam.
* Familiarity is a bonus, not the architectural reason.
