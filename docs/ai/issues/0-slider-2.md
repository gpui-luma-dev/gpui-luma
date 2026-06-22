# Slider 2: Additive Segmented Multi-Thumb Slider Architecture

## Goal

Create a new slider architecture that supports:

- segmented tracks
- domain-style spectrum tracks
- multiple thumbs with stable identity
- color-specific stop editing use cases such as gradient pickers
- future reuse for non-color sliders where that reuse is real

This work must be **additive**, not a rewrite of the current slider stack.

In particular:

- **Incubate with existing main use-cases**: Create the new architecture to support the two main existing single-thumb sliders first: `Slider` (quantity/fill) and `ColorSlider` (domain/spectrum).
- **No initial multi-thumb/crossover mix-in**: Defer multi-thumb gradient-stop mixers to a later step so Step 1 stays clean and focused.

---

## Why This Issue Exists

The current controls split into three different shapes:

- `Slider` is a single-value control with horizontal, vertical, and angular input strategies in [crates/sdk/src/controls/slider/model.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slider/model.rs).
- `RangeSlider` is really a single-value slider constrained by allowed intervals, with segmented sibling track rendering in [crates/sdk/src/controls/range_slider/model.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/range_slider/model.rs) and [crates/sdk/src/controls/range_slider/template.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/range_slider/template.rs).
- `ColorSlider` has the most mature color rendering behavior, including delegate-driven track drawing, interpolation choices, thumb-shape behavior, and color-aware thumb fill logic in [crates/sdk/src/controls/color/color_slider/model.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/color/color_slider/model.rs) and [crates/sdk/src/controls/color/color_slider/surface.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/color/color_slider/surface.rs).

The current problem is not “all sliders should become one control immediately.”

The actual problem is narrower:

- the current `ColorSlider` does not have a value/interactions model for multiple thumbs, stop identity, add/remove, or segmented constraints
- the current `RangeSlider` is not the desired long-term shape and is already considered expendable
- the current `Slider` is too simple to serve as the architectural center for gradient-stop and spectrum editing

So the new work should produce a **new shared engine** for segmented and multi-thumb behavior, while preserving the existing color slider family until the new path is genuinely better.

---

## Non-Negotiable Requirements

### 1. Additive rollout

- No in-place rewrite of `crates/sdk/src/controls/color/color_slider/`.
- No forced migration of existing gallery color slider demos during the first implementation pass.
- Existing `ColorSliderPane` and `ColorSliderRevealedPane` must remain valid reference surfaces during development:
  - [apps/gallery/src/gallery/panes/color/slider_pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/color/slider_pane.rs)
  - [apps/gallery/src/gallery/panes/color/slider_revealed_pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/color/slider_revealed_pane.rs)

### 2. Color-first incubation

The first real consumer of the new architecture should be a color use case that the current `ColorSlider` cannot handle well:

- segmented OKLCH spectrum tracks
- multiple gradient stops
- stop insertion and removal
- stop selection and focused editing

Plain fill sliders are a possible later reuse target, not the first design center.

### 3. Stable stop identity

The new control must model thumbs/stops with stable identifiers rather than positional indices alone.

This is required for:

- dragging across neighboring stops
- selection persistence while sorting by position
- external editor state keyed by a specific stop

### 4. Segments must be first-class render data

The track cannot be treated as only “one filled region plus one unfilled region.”

The render model must support:

- contiguous sibling segments across `0.0..=1.0`
- domain segments that show the underlying spectrum continuously
- inactive or occluded segments
- optional highlighted ranges or selected spans layered on top of a domain track

This should retain the sibling-rendering approach already proven in `RangeSliderTemplate`, rather than going back to overlap-plus-`overflow_hidden()` tricks.

### 5. Preserve mature color-slider visuals

The current color slider has already solved a lot of non-trivial behavior:

- delegate-driven background generation
- interpolation selection
- thumb shapes
- edge-to-edge behavior
- color-filled thumb rendering when appropriate

Those capabilities should not be thrown away. The new path should either:

- reuse those concepts directly, or
- adapt them behind a new interface

But it should not require rebuilding color behavior from scratch just to get multi-thumb support.

### 6. Keyboard and pointer behavior must be explicit

The new control needs a defined interaction contract for:

- selecting the nearest thumb on pointer down
- dragging a chosen thumb without identity jitter
- snapping to legal positions
- stepping over forbidden gaps
- adding a thumb from empty track space
- removing a thumb via gesture or keyboard

### 7. Keep LMTP seams clear

The new family should still follow the repo’s LMTP split:

- model
- control
- template
- theme or rendering adapters

Do not bury interaction policy inside ad hoc render closures.

---

## Scope Boundaries

### In scope

- a brand-new slider family under `crates/sdk/src/controls/`
- segmented track rendering
- multi-thumb value models
- color-first track rendering hooks
- gallery validation surfaces for gradient and spectrum editing
- optional later wrappers for simple single-thumb cases

### Out of scope for the first pass

- replacing the existing `Slider`
- preserving `RangeSlider` as a strategic control family
- migrating every color slider story immediately
- expanding into radial or ring controls
- broad theme-system redesign

---

## User-Facing Taxonomy

Names matter here. The public control vocabulary should be explicit and stable even if the internal engine evolves underneath it.

### Control names

- `Slider` — one movable thumb selecting a value
- `IntervalSlider` — two thumbs selecting one contiguous interval
- `MultiSlider` — multiple independently movable thumbs
- `Scrubber` — slider specialized for navigating time or ordered content

These names should be treated as the preferred user-facing API and gallery vocabulary.

That means:

- gallery panes and demos should use these names
- future builders or wrappers should use these names
- docs should describe the family in these terms

### Track presentation names

- `Fill` — selected quantity fills part of the track
- `Domain` — the whole track represents the value space; the thumb marks a position
- `Segments` — the track is divided into discrete regions
- `Ranges` — multiple highlighted intervals appear on the track

These names also fit the intended public API and documentation surface well.

### Important architectural note

The public names above do **not** require the internal implementation to start as one large enum-driven model.

The recommended approach in this document is still:

- build a lower-level shared engine around thumbs, segments, and interaction
- expose semantic user-facing controls on top of that engine

So these names should be considered:

- primary public names
- primary documentation names
- likely wrapper or builder names

They do not need to be the very first internal storage model if that would make the core harder to evolve.

---

## Architectural Direction

## 1. New control family, separate from existing controls

Create a new control family instead of mutating:

- `crates/sdk/src/controls/slider/`
- `crates/sdk/src/controls/range_slider/`
- `crates/sdk/src/controls/color/color_slider/`

Suggested working location:

- `crates/sdk/src/controls/slider2/`

The name can change, but the separation should remain: this must be a new incubation surface with its own API.

## 2. Core should be about thumbs + segments, not “all sliders”

The new core should be designed around three things:

- thumb collection and identity
- track segment generation
- interaction state and gesture policy

That is the real shared engine.

It should not overcommit on day one to a giant taxonomy for every future slider variant.

## 3. Color rendering should plug into the core

The new track model should allow color-specific renderers to provide:

- domain backgrounds for spectrum-like tracks
- per-segment backgrounds
- checkerboard underlays where needed
- thumb preview colors

This keeps color-specific sophistication without forcing it into the generic state machine.

## 4. Prioritize single-thumb quantity and color sliders first

Instead of building multi-thumb capabilities first and wrapping plain sliders later, the incubation will target the two primary single-thumb use cases immediately:
- **`Slider`** (quantity fill progress)
- **`ColorSlider`** (static spectrum background)

This ensures the core architecture snaps perfectly to Luma's main slider workloads before introducing the added complexity of multi-thumb selection, insertion/deletion, and crossover sorting.

---

## Proposed Data Model

This is the shape to aim for, not a frozen API:

### Thumb model

- `ThumbId`
- `SliderThumbValue { id, position, preview, role }`

Notes:

- `position` should be normalized or easily convertible to normalized space
- `preview` can carry optional color or semantic thumb styling hints
- `role` can support future min/max/stop semantics without hard-coding them into the base type

### Value model

Prefer a small collection-first model over a large enum-first taxonomy.

Example direction:

- one-thumb sliders use a one-element thumb list
- gradient editors use many thumbs
- interval-style consumers can use two thumbs with semantic roles

This is a better fit for color-stop editing than centering the base model on `Single | Interval | Multi`.

### Track model

- `TrackSegment`
- start and end positions
- segment kind
- optional segment payload for rendering

Likely segment kinds:

- `Domain`
- `Active`
- `Inactive`
- `Blocked`
- `SelectedRange`

The exact names can change, but the core idea is that segments are render data, not an implementation detail of one template.

### Constraint model

Keep constraints layered rather than crammed into one enum immediately.

Likely concerns:

- overall bounds
- step policy
- allowed intervals
- minimum thumb distance
- crossover allowed or not
- minimum and maximum thumb count

These may compose better as a small struct than as one catch-all enum.

---

## Detailed Plan

### Phase 1: Scaffold the new isolated slider engine supporting Slider and ColorSlider

Create a new control family (e.g. `crates/sdk/src/controls/slider2/`) with its own:
- `model.rs`
- `control.rs`
- `template.rs`

Step 1 milestone:
- Support standard single-thumb **`Slider`** (dynamic quantity fill track).
- Support standard single-thumb **`ColorSlider`** (static spectrum gradient track).
- Verify single-thumb interaction, LTR/RTL layout mapping, and rendering isolation.
- Completely remove/revert the legacy `RangeSlider` crate.

### Phase 2: Implement sibling segment rendering & corner rounding correctness

Ensure visual presentation is 100% correct without relying on `overflow_hidden()` hacks:
- Implement sibling-level track segment partition mapping.
- Apply outer-edge-only corner rounding.
- Port orientation-aware rendering layout from previous designs.

### Phase 3: Incubate multi-thumb state, selection, and stop editing (Later Phase)

After the single-thumb core is proven, expand the engine to support:
- Stable multi-thumb collections using `ThumbId`.
- Crossover sorting and drag-identity persistence.
- Dynamic stop insertion (clicking empty track) and deletion (drag-off).

### Phase 4: Add color-capable track rendering hooks

Introduce rendering hooks that can express:

- a continuous color domain track
- segmented spectrum bands
- per-thumb preview color
- checkerboard underlays for alpha-like cases

This is the point where the new family becomes useful for:

- OKLCH spectrum sliders
- gradient stop pickers
- richer color editors

The hook surface should be rich enough to preserve current color sophistication, but not so broad that all color logic moves into the generic core.

### Phase 5: Build new gallery prototype surfaces

Add a dedicated gallery validation pane for the new family instead of immediately replacing existing panes.

That pane should include at least:

- single-thumb segmented domain example
- blocked-gap example
- multi-thumb gradient-stop example
- add/remove stop behavior
- selected-stop inspection state

Good nearby validation surfaces:

- existing slider gallery area under [apps/gallery/src/gallery/panes/slider/](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/slider/)
- existing color and color-composition panes under [apps/gallery/src/gallery/panes/color/](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/color/) and [apps/gallery/src/gallery/panes/color_compositions/](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/color_compositions/)

The multi-mixer surface is especially relevant as a later consumer:

- [apps/gallery/src/gallery/panes/color_compositions/multi_mixer.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/color_compositions/multi_mixer.rs)

### Phase 6: Prove one real color migration

After the prototype feels right, migrate one real color use case that the old `ColorSlider` cannot express cleanly.

Best candidates:

- an OKLCH constrained spectrum slider
- a gradient stop editor

This phase must preserve old color slider demos so the new control is compared against working reference behavior, not developed in a vacuum.

### Phase 7: Decide what to do with old controls

Only after Phase 6 succeeds:

- decide whether `RangeSlider` should be deprecated or removed
- decide whether plain `Slider` should remain independent or get a thin wrapper over the new family
- decide whether some `ColorSlider` delegates should be adapted into the new control family

This decision should be based on actual reuse, not on an up-front desire for conceptual purity.

---

## Migration Strategy

### Keep as-is initially

- `crates/sdk/src/controls/color/color_slider/`
- `crates/sdk/src/controls/slider/`
- current gallery slider and color panes

### Treat as disposable or non-strategic

- `crates/sdk/src/controls/range_slider/`
- range-slider-specific gallery stories, except where they are useful as a regression reference for segment rendering

### New work goes here

- new SDK control family under `crates/sdk/src/controls/`
- new gallery prototype pane under `apps/gallery/src/gallery/panes/slider/` or a clearly related color-composition prototype surface

---

## Acceptance Criteria

### Architectural

- A new slider family exists without breaking current slider or color-slider consumers.
- The new family has explicit model/control/template seams.
- The new family supports segmented sibling track rendering without relying on `overflow_hidden()` for rounded edge correctness.

### Interaction

- Multiple thumbs can be selected and dragged with stable identity.
- Thumb ordering does not cause active-thumb jitter.
- Pointer and keyboard interaction behavior is defined and testable.
- Gap or blocked regions can be skipped or clamped intentionally.

### Color capability

- The new family can render a spectrum-like domain track.
- The new family can render and edit multiple color stops.
- The new family can surface thumb preview color without regressing generic slider behavior.

### Rollout safety

- Existing color slider demos remain functional throughout incubation.
- The new gallery prototype makes side-by-side validation possible before any migration decision.

---

## Recommended First Concrete Deliverable

The first meaningful deliverable should be:

- a new gallery prototype pane powered by the new slider family
- one segmented spectrum example
- one multi-thumb gradient-stop example
- no migration of the current `ColorSliderPane`

That gets the architecture tested against the real problem without risking the mature existing color slider work.
