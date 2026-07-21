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
- optional later builder APIs for simple single-thumb cases

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
- future builders should use these names
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
- likely builder names

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

Instead of building multi-thumb capabilities first and handling plain sliders later, the incubation will target the two primary single-thumb use cases immediately:
- **`Slider`** (quantity fill progress)
- **`ColorSliderBuilder`** (static spectrum background, returning a standard `Slider` entity)

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

### Phase 3: Unify 1D Angular/Circular Dial Controls

To consolidate dial and ring sliders (e.g. `ColorArc`, `ColorRing`, and angular sliders) into the unified engine:
- Implement `Slider2InputStrategy` to define pointer mapping strategy:
  ```rust
  pub enum Slider2InputStrategy {
      Horizontal,
      Vertical,
      /// Calculates mouse angle relative to bounds center using atan2
      Angular {
          min_angle: f32, // e.g. -135.0
          max_angle: f32, // e.g. 135.0
      },
  }
  ```
- Translate screen points to angular percentages in `percentage_from_position`:
  ```rust
  // Angular percentage mapping:
  let center = bounds.origin + bounds.size / 2.0;
  let delta = position - center;
  let angle_rad = f32::atan2(delta.y.0, delta.x.0);
  let mut angle_deg = angle_rad.to_degrees();
  // Map angle_deg to 0.0..=1.0 relative to min_angle..=max_angle
  ```
- Add a `wrapping: bool` field to `Slider2Model` for continuous circular dials (like Hue wheels). When `wrapping` is true, the value bounds constraint uses modulo arithmetic rather than clamping:
  ```rust
  let span = range.end - range.start;
  let mut wrapped = (value - range.start) % span;
  if wrapped < 0.0 {
      wrapped += span;
  }
  let value = wrapped + range.start;
  ```
- Ensure the controller is generic enough to support rendering via custom dial templates (Arc/Ring templates) while reusing the core interaction state.

### Phase 4: Incubate multi-thumb state, selection, and stop editing ✅

After the single-thumb core is proven, expand the engine to support:
- Stable multi-thumb collections using `ThumbId`.
- Crossover sorting and drag-identity persistence.
- Dynamic stop insertion (clicking empty track) and deletion (keyboard).

**Delivered:**
- `Slider2ThumbPolicy` + `thumbs.rs` (hit testing, min distance, insert/remove)
- Builder: `.multi_stop()`, `.thumb_policy()`, `.thumb_values()`
- Events: `Change`/`Release` include `thumb_id`; `ThumbAdded`, `ThumbRemoved`, `ThumbSelected`
- Linear template renders all thumbs with per-thumb drag and active focus ring
- Keyboard: Delete/Backspace removes active thumb (`RemoveValue` on RangeValue profile)
- Gallery: multi-stop fill-track demo

### Phase 5: Replace and Migrate Legacy Slider to Slider2 ✅

Migrate all legacy single-thumb `Slider` usages in the codebase to use the new unified `Slider2` engine:
- Delete the old `crates/sdk/src/controls/slider/` module.
- Move and rename `crates/sdk/src/controls/slider2/` to `crates/sdk/src/controls/slider/`.
- Re-export `Slider`, `SliderEvent`, `SliderBuilder`, etc. for backward compatibility.
- Update `look-shadcn` control extension and template binding methods.
- Update and merge the old `pane.rs` and `slider2_pane.rs` gallery pages.
- Ensure all other client preference panels and demos are adapted and compile.
- Confirm zero impact on the isolated `ColorSlider` controls.

**Done:** Legacy control deleted; implementation lives flat under `slider/` (with `theme.rs` + `template/`); `slider/mod.rs` re-exports legacy names; `look.slider()` binds the new linear template; gallery unified to one Slider page; luma-studio, neumorphic custom templates, and event handlers updated; ColorSlider untouched.

### Phase 6: Add Color-Capable Track Rendering Hooks & ColorSlider Migration ✅

Migrate the legacy `ColorSlider` control to the unified template-driven engine to validate linear color editing:

#### 1. Core Engine Extensions (DomainTrackRenderer Hook & Runtime Modification)
- Extend the `DomainTrackRenderer` trait in `domain.rs` to support querying preview colors:
  ```rust
  pub trait DomainTrackRenderer: Send + Sync {
      fn paint(&self, bounds: Bounds<Pixels>, orientation: Slider2Orientation, reversed: bool, window: &mut Window);
      fn get_color_at_position(&self, position: f32) -> Option<Hsla> { None }
  }
  ```
- In `SliderControl::set_thumb_position_internal` (`control.rs`), automatically invoke `get_color_at_position` using the new thumb position whenever it changes, updating `thumb.preview = Some(color)`.
- **Dynamic Track Updates**: Add a runtime setter `set_domain_track` to `SliderControl` to allow updating the track renderer dynamically at runtime (critical for Saturation/Alpha track updates when Hue/base color changes):
  ```rust
  pub fn set_domain_track(&mut self, renderer: Arc<dyn DomainTrackRenderer>, cx: &mut Context<Self>) {
      self.model.presentation = TrackPresentation::Domain;
      self.model.domain_track = Some(renderer);
      cx.notify();
  }
  ```

#### 2. ColorSlider Migration (Wrapper-Free, Builder-Based)
- **Eliminate `ColorSlider` wrapper component**: Remove the legacy `ColorSlider` struct/component class completely, avoiding any runtime wrapper boilerplate.
- **Implement a dedicated `ColorSliderBuilder`**: Create a `ColorSliderBuilder` in the SDK that returns a standard `Slider` control (`Entity<SliderControl>`) configured with:
  - `TrackPresentation::Domain`
  - A custom `DomainTrackRenderer` delegate corresponding to the requested color dimension.
- **Strict type safety (no magic strings)**: The `ColorSliderBuilder` will expose explicit methods for configuring different spectrum/dimension options (such as `.hue()`, `.saturation(color)`, `.alpha(color)`, `.lightness(color)`) instead of using string keys or generic wrappers.
- **Adapt Track Renderers**: Implement `DomainTrackRenderer` directly on the existing `ColorSliderDelegate` concrete types (or new unified types like `HueDelegate`, `GradientDelegate`, `AlphaDelegate`, `ChannelDelegate`).
- Port vector/raster gradient and checkerboard rendering math from the delegates' old `style_background` methods into their respective `DomainTrackRenderer::paint` implementations.

**Done:** `DomainTrackRenderer` + thumb preview sync + `set_domain_track`/`set_range` on unified slider; wrapper-free `ColorSliderBuilder` → `Entity<SliderControl>` with typed factories; legacy `ColorSlider`/`ColorSliderState`/`surface.rs`/`model.rs` removed; all gallery color-slider consumers migrated (`slider_pane`, `slider_revealed_pane`, HSV plane, color picker, multimixer); dynamic sat/alpha delegate sync via `ColorSliderDomainRenderer` + `update_domain_delegate`/`refresh_color_slider`.

### Phase 7: Unify Radial Color Controls - Part 1 (ColorArc) ✅

Migrate the legacy `ColorArc` control to standard `Slider` entities configured with angular input strategy and angular template, avoiding wrapping component classes:
- **`ColorArc` Migration**:
  - Expose a `ColorArcBuilder` returning a standard `Slider` (`Entity<SliderControl>`) configured with angular input strategy (`.angular(min_angle, max_angle)`) and using `ColorArcTemplate`.
  - Delegate the radial gradient/spectrum rendering math from its old delegates directly to `DomainTrackRenderer::paint` inside the dial canvas.
- Ensure pointer tracking and snapping/intervals delegate to the unified slider core.

**Done:** Wrapper-free `ColorArcBuilder` → `Entity<SliderControl>` with typed hue/sat/lightness factories and vector/raster renderer selection; `ColorArcDomainRenderer` + `ColorArcTemplate` with dynamic dial geometry; legacy `ColorArcState`/`ColorArc`/`surface.rs`/`model.rs` removed; `DomainTrackRenderer::raster_image` hook for bitmap arcs; gallery `arc_pane` and `hue_ring_sl_arcs` migrated.

### Phase 8: Unify Radial Color Controls - Part 2 (ColorRing) ✅

Migrate the legacy `ColorRing` control to standard `Slider` entities configured with circular input strategy and ring template, avoiding wrapping component classes:
- **`ColorRing` Migration**:
  - Expose a `ColorRingBuilder` returning a standard `Slider` (`Entity<SliderControl>`) configured with angular input strategy (`.angular(0.0, 2.0 * PI).wrapping(true)`) and using `ThemedCircularRingTemplate`.
  - Delegate the circular gradient/spectrum rendering to `DomainTrackRenderer::paint` within the ring canvas.
- Ensure pointer tracking, wrapping boundary seam logic, and snapping/intervals delegate to the unified slider core.

**Done:** Expose a `ColorRingBuilder` returning standard unified `Slider` entities, angular wrapping strategy, themed ring template, and circular gradient rendering delegated to track canvas/domain renderer.

### Phase 9: Build new gallery prototype surfaces [Skipped]

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

### Phase 10: Prove one real color migration ✅

After the prototype feels right, migrate one real color use case that the old `ColorSlider` cannot express cleanly.

Best candidates:

- an OKLCH constrained spectrum slider
- a gradient stop editor

This phase must preserve old color slider demos so the new control is compared against working reference behavior, not developed in a vacuum.

**Done:** OKLCH constrained spectrum slider validated in `MultiMixerState` using allowed intervals and dynamic channel tracking.

### Phase 11: Finalize cleanup of old controls ✅

Only after Phase 10 succeeds:

- Deprecate or remove remaining legacy files and tests.
- Ensure all radial controls and linear spectrum controls are fully powered by the unified `Slider` control.

**Done:** All legacy `ColorRing` files (`ring.rs`, `surface.rs`, `model.rs`, `control.rs`, `factory.rs`) deleted from `crates/sdk/src/controls/color/color_ring/`. Disallowed old ring pane page references removed from the gallery.

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
