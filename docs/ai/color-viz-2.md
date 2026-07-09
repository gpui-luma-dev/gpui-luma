# Color Viz 2: Tabbed Gradient and Mesh Execution Plan

## Purpose

This document lays out a concrete implementation plan for the next `apps/color-viz` expansion:

1. Add a tabbed shell so stop-based gradients and mesh do not share one forced layout.
2. Add radial gradient support.
3. Add angular gradient support.
4. Leave the mesh tab intentionally empty in phase 1.
5. Add a second-phase mesh-gradient editor with a manually movable `3 x 3` point grid.
6. Leave generalized mesh dimensions for a later phase 3.

This is an app-local plan for `apps/color-viz`, not an SDK-generalization plan.

## Desired Outcome

The target experience is a gradient playground where:

1. The app first splits into two top-level tabs:
   - `Gradients`
   - `Mesh`
2. The `Gradients` tab contains `Linear`, `Radial`, and `Angular` using the same stop editor, stop list, color picker, renderer diagnostics, and preview shell.
3. Radial does not expose angle, since angle is not meaningful there.
4. Angular uses the same angle control already present in the app.
5. In phase 1, the `Mesh` tab exists but is intentionally empty.
6. In phase 2, mesh mode presents a visible `3 x 3` connected grid of control points like the supplied reference image.
7. As a user drags any mesh point, the visible lattice remains connected and the preview re-rasterizes from the updated geometry.

## Constraints From Current GPUI

The current GPUI checkout only exposes native background support for:

1. Solid fills
2. Linear gradients
3. Pattern slash
4. Checkerboard

See:

- [`/Users/scg/.cargo/git/checkouts/zed-a70e2ad075855582/d08d98f/crates/gpui/src/color.rs:657`](/Users/scg/.cargo/git/checkouts/zed-a70e2ad075855582/d08d98f/crates/gpui/src/color.rs:657)
- [`/Users/scg/.cargo/git/checkouts/zed-a70e2ad075855582/d08d98f/crates/gpui/src/color.rs:777`](/Users/scg/.cargo/git/checkouts/zed-a70e2ad075855582/d08d98f/crates/gpui/src/color.rs:777)

Implication:

1. Native GPUI can remain a useful path for some linear previews.
2. Radial, angular, and mesh must be app-side rasterized in `color-viz`.
3. This work should reuse the app's existing `RenderImage` preview path, not attempt to force new gradient primitives into SDK or GPUI.

## Current App State

`apps/color-viz` already has the right high-level architecture for this work.

### Existing strengths

1. A dedicated gradient builder entity already owns editing state:
   - [`apps/color-viz/src/gradient_builder/builder.rs`](/Users/scg/Developer/GitHub/gpui-luma/apps/color-viz/src/gradient_builder/builder.rs)
2. The preview already supports:
   - direct painting
   - synchronous raster generation
   - asynchronous raster generation
3. The preview already caches raster results and invalidates on size / stop / angle changes:
   - [`apps/color-viz/src/gradient_builder/builder.rs:515`](/Users/scg/Developer/GitHub/gpui-luma/apps/color-viz/src/gradient_builder/builder.rs:515)
4. The paint layer is already app-local:
   - [`apps/color-viz/src/gradient_builder/paint.rs`](/Users/scg/Developer/GitHub/gpui-luma/apps/color-viz/src/gradient_builder/paint.rs)
5. Stop data is already explicit `(position, color)` rather than implied spacing:
   - [`crates/sdk/src/controls/color/color_slider/delegates.rs:228`](/Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/color/color_slider/delegates.rs:228)

### Current limitations

1. `GradientType` is currently only `Linear`:
   - [`apps/color-viz/src/gradient_builder/paint.rs:11`](/Users/scg/Developer/GitHub/gpui-luma/apps/color-viz/src/gradient_builder/paint.rs:11)
2. The type selector only exposes `Linear`:
   - [`apps/color-viz/src/gradient_builder/builder.rs:123`](/Users/scg/Developer/GitHub/gpui-luma/apps/color-viz/src/gradient_builder/builder.rs:123)
   - [`apps/color-viz/src/gradient_builder/builder.rs:935`](/Users/scg/Developer/GitHub/gpui-luma/apps/color-viz/src/gradient_builder/builder.rs:935)
3. CSS output only formats `linear-gradient(...)`:
   - [`apps/color-viz/src/gradient_builder/color.rs:60`](/Users/scg/Developer/GitHub/gpui-luma/apps/color-viz/src/gradient_builder/color.rs:60)
4. The raster path only knows how to sample a linear projection:
   - [`apps/color-viz/src/gradient_builder/paint.rs:203`](/Users/scg/Developer/GitHub/gpui-luma/apps/color-viz/src/gradient_builder/paint.rs:203)

## Reference Reuse Inside This Repo

### Reuse target 1: color-viz preview cache and raster seam

The current preview design is the exact seam to extend:

1. `GradientBuilder` owns:
   - gradient type
   - angle
   - preview renderer selection
   - cached `RenderImage`
2. `paint.rs` owns:
   - painting helpers
   - raster generation
   - stop sorting and sampling

That seam should stay intact.

### Reuse target 2: Gallery color harmonies point overlay

The Gallery color harmonies pane already renders multiple markers on top of a background field:

- [`apps/gallery/src/gallery/panes/color_compositions/compositions/color_harmonies.rs:440`](/Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/color_compositions/compositions/color_harmonies.rs:440)
- [`apps/gallery/src/gallery/panes/color_compositions/compositions/color_harmonies.rs:446`](/Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/color_compositions/compositions/color_harmonies.rs:446)

That code is useful as a structural example for:

1. absolute marker placement
2. marker sizing
3. layering markers over a rendered color surface

It is not directly reusable as a control, but it shows the app-local composition pattern we want for mesh points.

### Reuse target 3: SDK color field drag interaction

The SDK color field already has the drag pattern needed for movable mesh points:

- [`crates/sdk/src/controls/color/color_field/field/view.rs:202`](/Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/color/color_field/field/view.rs:202)

This is the main interaction reference for:

1. pointer-down capture
2. drag state activation
3. continuous updates during drag
4. release handling

For mesh editing, reuse the gesture style, not the entire control.

## Scope Decision

This work should remain app-local in `apps/color-viz` for now.

Reasons:

1. GPUI does not natively expose radial, angular, or mesh background primitives in this checkout.
2. The existing `color-viz` feature is explicitly experimental.
3. Mesh gradients are still highly exploratory and platform-sensitive.
4. The UI and data model will likely evolve quickly once manual control-point editing exists.

Do not broaden this into an SDK abstraction during the first implementation.

## Feature Breakdown

## Phase 1: Tabbed Shell, Radial, and Angular

### Tab shell requirements

1. Introduce two top-level tabs immediately:
   - `Gradients`
   - `Mesh`
2. Put all stop-based gradient work inside `Gradients`.
3. Make `Mesh` a deliberate placeholder during phase 1 rather than forcing mesh-specific layout into the stop editor shell.
4. Reuse existing SDK/navigation controls for the tabs instead of inventing app-local fake tabs.

### Why tabs belong in phase 1

The stop-based modes and the future mesh editor do not share the same editing model.

`Gradients` is centered on:

1. ordered 1D stops
2. one active stop
3. stop list management
4. type-specific angle behavior

`Mesh` is centered on:

1. 2D point topology
2. connected guide lines
3. geometry editing
4. point list and selection instead of stop list

### User-facing requirements

1. Add the top-level tabs before mesh implementation.
2. Add `Radial` and `Angular` to the existing gradient type selector inside the `Gradients` tab.
3. Preserve the current stop editor workflow.
4. Preserve current color stop insertion/removal/editing behavior.
5. Preserve current diagnostics panel and renderer selector.
6. Hide or disable angle when type is `Radial`.
7. Keep angle available for `Linear` and `Angular`.
8. Show the `Mesh` tab as an intentional placeholder until phase 2 begins.

### Data model changes

Extend the app-local type enum in `paint.rs`:

```rust
pub enum GradientType {
    Linear,
    Radial,
    Angular,
    Mesh,
}
```

`Mesh` can land in the enum early even if phase 1 only fully implements the first three modes. Doing that now reduces later branching churn in `builder.rs`.

### Builder changes

Primary file:

- [`apps/color-viz/src/gradient_builder/builder.rs`](/Users/scg/Developer/GitHub/gpui-luma/apps/color-viz/src/gradient_builder/builder.rs)

Required updates:

1. Add top-level tab state:
   - `Gradients`
   - `Mesh`
2. Render a tab strip at the top of the builder shell.
3. Route the existing stop-based controls into the `Gradients` tab body.
4. Render a placeholder body for `Mesh`.
5. Extend `type_items()` to include:
   - `linear`
   - `radial`
   - `angular`
   - optional early placeholder `mesh`
6. Extend `handle_type_event()` to switch over all gradient-tab types.
7. Make `gradient_spec()` dispatch by `gradient_type`.
8. Include `gradient_type` in preview cache invalidation.
9. When switching to `Radial`, skip angle display in the editor panel.
10. When switching to `Angular`, keep the angle control and treat it as the rotation origin for conic sampling.

### CSS / textual output changes

Primary file:

- [`apps/color-viz/src/gradient_builder/color.rs`](/Users/scg/Developer/GitHub/gpui-luma/apps/color-viz/src/gradient_builder/color.rs)

Add formatting helpers:

1. `format_css_radial_gradient(...)`
2. `format_css_angular_gradient(...)`
3. `format_css_gradient(...)` dispatcher

Recommended output style:

1. Linear:
   - `linear-gradient(90deg, ...)`
2. Radial:
   - `radial-gradient(circle, ...)`
3. Angular:
   - `conic-gradient(from 90deg, ...)`

This keeps the preview and the exported string aligned with common CSS mental models.

### Preview strategy changes

The existing renderer menu currently has:

1. `Quads`
2. `Render (Sync)`
3. `Render (Async)`

Recommended behavior by type:

1. `Linear`
   - keep all three modes
   - native / quads path still useful
2. `Radial`
   - force render path
   - `Quads` should either be disabled or transparently resolved to render
3. `Angular`
   - force render path
   - `Quads` should either be disabled or transparently resolved to render
4. `Mesh`
   - force render path

The diagnostics panel should continue to distinguish:

1. requested renderer
2. active renderer actually used

That matters even more once only `Linear` has a true non-raster path.

### Rendering implementation for radial

Primary file:

- [`apps/color-viz/src/gradient_builder/paint.rs`](/Users/scg/Developer/GitHub/gpui-luma/apps/color-viz/src/gradient_builder/paint.rs)

Recommended first-pass radial definition:

1. Gradient center fixed at preview center.
2. Radius derived from furthest corner from center.
3. Stop positions interpreted along `0.0..1.0` of that radius.
4. Sample value:
   - `distance(sample, center) / max_radius`

That is enough to ship a useful radial mode without introducing focal-point controls.

Implementation shape:

1. Add `rasterize_radial_gradient_preview(size, stops) -> Option<Arc<RenderImage>>`
2. Add `paint_radial_gradient_preview(...)`
3. Keep direct paint path simple:
   - likely route radial through raster even in sync mode

Important note:

Do not spend time trying to map radial into GPUI `Background`; that API does not exist in this checkout.

### Rendering implementation for angular

Recommended first-pass angular definition:

1. Gradient center fixed at preview center.
2. Angle slider offsets the starting angle.
3. Stop positions interpreted around the full turn:
   - `0.0` = start angle
   - `1.0` = one full revolution

Sampling:

1. Compute delta from center to sample point.
2. Compute `atan2(dy, dx)`.
3. Normalize to `0.0..1.0`.
4. Apply rotation offset from the angle slider.
5. Sample stop color with wrap semantics.

This is the main new color-math behavior for phase 1.

### Shared sampler direction

The current helper `color_at_position(...)` assumes non-wrapping linear sampling. That still works for linear and radial, but angular needs wrap-aware interpolation.

Recommended addition:

1. keep `color_at_position(...)` for clamped sampling
2. add `color_at_wrapped_position(...)` for angular

That avoids overloading one helper with conflicting assumptions.

### Cache key changes

`PreviewImageCacheKey` currently keys on:

1. width
2. height
3. rotation
4. stops

For the next phase it must also include:

1. `gradient_type`
2. any mesh geometry when mesh is active

Without that, switching between linear / radial / angular with the same stop set will incorrectly reuse stale rasters.

### Detailed phase 1 file plan

#### `apps/color-viz/src/gradient_builder/paint.rs`

Add:

1. `GradientType::{Radial, Angular, Mesh}`
2. radial raster helper
3. angular raster helper
4. wrapped angular stop sampler
5. top-level raster dispatcher by `GradientType`

Keep:

1. existing linear helpers
2. existing direct-paint fast paths for linear

#### `apps/color-viz/src/gradient_builder/builder.rs`

Change:

1. type selector items
2. type event handling
3. preview-cache key generation
4. preview generation dispatcher
5. diagnostics text
6. stop-editor panel layout so angle hides in radial mode

#### `apps/color-viz/src/gradient_builder/color.rs`

Change:

1. add radial formatter
2. add angular formatter
3. add gradient-type dispatcher

## Phase 2: Mesh Grid

Phase 2 starts inside the already-existing `Mesh` tab.

That means mesh work can focus on:

1. model
2. overlay
3. rasterization
4. point editing

It does not need to renegotiate the top-level layout at the same time.

## Product definition

The requested mesh mode is not "mesh gradients as a platform primitive." It is:

1. a rectangular preview surface
2. a visible `3 x 3` lattice of connected points
3. manual point dragging
4. colors assigned to points
5. a re-rasterized preview as geometry changes

The reference image suggests:

1. all points remain visibly connected by row and column guide lines
2. the mesh deforms continuously
3. colors remain attached to the points
4. the preview is the interpolated fill across the deformed cells

That should be the implementation target.

## Mesh architecture

### Keep mesh separate from stop-based gradients

Do not try to force mesh into the same exact model as linear/radial/angular stops.

Linear/radial/angular model:

1. ordered 1D stop positions
2. one color at each position

Mesh model:

1. 2D point positions
2. one color per point
3. fixed topology
4. deformed geometry

The UI shell can stay shared, but the data model should not.

### Recommended app-local mesh types

Add a dedicated mesh module under `apps/color-viz/src/gradient_builder/`, for example:

```text
mesh.rs
mesh_paint.rs
mesh_view.rs
```

Suggested model:

```rust
pub struct MeshPoint {
    pub id: MeshPointId,
    pub row: usize,
    pub col: usize,
    pub u: f32,
    pub v: f32,
    pub color: Hsla,
}

pub struct MeshGrid {
    pub rows: usize,
    pub cols: usize,
    pub points: Vec<MeshPoint>,
}
```

Initial invariant:

1. `rows == 3`
2. `cols == 3`

This should remain fixed for the first implementation.

### Initial mesh seed

Initialize the grid as a regular lattice:

1. top row at `v = 0.0`
2. middle row at `v = 0.5`
3. bottom row at `v = 1.0`
4. columns at `u = 0.0`, `0.5`, `1.0`

That directly matches the reference structure.

### Initial mesh colors

Good default:

1. top row dark / black
2. middle row blue
3. bottom row green

This mirrors the new example image and gives immediate visual proof that the grid is working.

Recommended default deformation:

1. leave the outer eight points on the regular lattice
2. offset the center point away from the exact center
3. use that center offset as the initial proof that geometry and color interpolation are both live

This matches the example more closely than a perfectly regular center point.

### Rendering model

There are two plausible first-pass rasterization approaches.

#### Option A: Bilinear interpolation per cell

Each quad cell is bounded by four corner points:

1. top-left
2. top-right
3. bottom-left
4. bottom-right

For each pixel:

1. find which logical cell it belongs to in mesh space
2. solve local `(s, t)` inside that cell
3. bilinearly interpolate point color from the four corners

Pros:

1. visually appropriate for a rectangular mesh
2. matches the "connected grid" mental model
3. simpler than trying to imitate Apple mesh shading exactly

Cons:

1. requires inverse mapping if cells are deformed non-affinely

#### Option B: Split each cell into two triangles

For each cell:

1. split into two triangles
2. use barycentric interpolation within each triangle

Pros:

1. easier point-in-triangle math
2. stable for a first implementation

Cons:

1. can show a diagonal seam inside each cell
2. looks less like a smooth mesh

Recommendation:

1. Ship triangle subdivision first if speed of implementation matters most.
2. Move to bilinear cell interpolation if the diagonal seam looks too crude.

Given the visual goal, bilinear is the better long-term direction, but triangle split is the fastest validation pass.

### Visible mesh overlay

Mesh mode needs a dedicated overlay layer above the preview.

That overlay should render:

1. horizontal guide segments between neighboring points
2. vertical guide segments between neighboring points
3. point handles at every mesh point

This overlay is not just decorative. It is the editing affordance.

Recommended rendering structure:

1. preview raster as the bottom layer
2. guide-line layer
3. handle layer

Guide lines can be light translucent white like the reference image.

### Point handles

Handle styling should follow the same basic composition as the Gallery harmony markers:

1. outer ring
2. inner color fill
3. absolute placement over the preview

But mesh handles need larger hit targets than pure decorative markers.

Recommended split:

1. visible dot: small
2. hit target: larger invisible or translucent box

### Interaction plan

Each mesh point needs:

1. hit testing
2. selection
3. drag state
4. live position update
5. clamp to preview bounds

Recommended behavior:

1. pointer down on handle selects point
2. drag immediately updates that point's normalized `(u, v)`
3. on move, point remains inside preview rect
4. on release, preview stays at final geometry

For the first pass, do not try to preserve topological ordering constraints beyond bounds clamping.

That means:

1. points may cross each other if dragged aggressively
2. cells may invert

This is acceptable only for the first internal experiment if we want speed. If visual stability is more important, add row/column monotonic constraints:

1. each point must stay between its left/right neighbors
2. each point must stay between its top/bottom neighbors

Recommendation:

1. enforce monotonic ordering from the beginning

Why:

1. the reference image implies a stable connected lattice
2. inverted cells will make the rasterizer and UI much harder to reason about
3. monotonic clamping is cheap compared with debugging self-crossing mesh geometry

### Monotonic constraint rules

For a point at `(row, col)`:

1. its `u` cannot move left of the point to its left plus epsilon
2. its `u` cannot move right of the point to its right minus epsilon
3. its `v` cannot move above the point above plus epsilon
4. its `v` cannot move below the point below minus epsilon

Edge points still clamp to `0.0..1.0`.

This preserves the connected-grid mental model and prevents invalid cell flips.

### Mesh color editing

The simplest compatible plan is:

1. selecting a mesh point routes the existing color picker to that point's color
2. the stop list panel becomes a point list in mesh mode
3. add no new color editing widget for phase 2

This keeps the interface familiar.

Suggested mesh side panel shape:

1. top-level tabs remain unchanged
2. renderer selector
3. point list
4. selected point metadata:
   - row
   - col
   - `u`
   - `v`
   - color
5. optional reset button for the grid

### Mesh CSS output

There is no stable cross-platform CSS equivalent for the macOS mesh feature you described that maps cleanly to this implementation.

Recommendation:

1. do not fake a CSS export in phase 2
2. show a human-readable mesh summary instead

For example:

1. grid size
2. selected point
3. point positions
4. point colors

If later you want a serializable format, define an app-local `mesh-gradient(...)` debug string or JSON export.

## Phase 3: Generalized Mesh Dimensions

Phase 3 is where mesh topology becomes configurable.

That phase can introduce:

1. arbitrary `rows`
2. arbitrary `cols`
3. point insertion or removal rules
4. generalized panel UI for larger meshes

Do not pull that into phase 2.

Reasons:

1. `3 x 3` is enough to validate the full interaction loop
2. the interpolation and dragging constraints are easier to debug on fixed topology
3. the side-panel UX is much simpler when the point count is known in advance
4. generalized meshes will likely change the data model and validation rules

## Execution Sequence

## Milestone 1: Introduce Tabbed Shell and Preserve Linear

Goal:

1. split the app into `Gradients` and `Mesh`
2. keep current linear behavior intact
3. leave `Mesh` intentionally empty

Tasks:

1. add top-level tab state
2. render tab controls using existing SDK/navigation patterns
3. move the existing gradient UI under `Gradients`
4. render a placeholder `Mesh` body
5. verify existing linear mode still behaves the same

Exit criteria:

1. the app has two top-level tabs
2. `Gradients` behaves like the current app
3. `Mesh` is present but empty by design

## Milestone 2: Refactor for type-based preview dispatch

Goal:

1. introduce `GradientType` branching cleanly
2. keep linear behavior unchanged

Tasks:

1. expand `GradientType`
2. add formatter dispatch
3. add raster dispatch
4. add cache-key `gradient_type`
5. update diagnostics wording

Exit criteria:

1. linear still behaves exactly as before
2. switching type changes preview/spec pipeline cleanly

## Milestone 3: Radial support

Goal:

1. working radial preview
2. radial CSS string
3. angle hidden or disabled

Tasks:

1. add radial type selector item
2. add radial formatter
3. add radial raster sampler
4. disable native/quads path for radial
5. verify stop insertion / deletion / recolor still work

Exit criteria:

1. radial preview updates on stop moves
2. radial preview updates on color changes
3. angle control is absent or inert in radial mode

## Milestone 4: Angular support

Goal:

1. working conic-like preview
2. angle rotates the starting seam

Tasks:

1. add angular type selector item
2. add wrapped stop sampler
3. add angular raster sampler
4. add angular formatter
5. ensure seam rotation tracks angle slider

Exit criteria:

1. angular preview rotates when angle changes
2. stop order wraps correctly around the circle
3. no stale cache reuse between linear/radial/angular

## Milestone 5: Mesh data model and static render

Goal:

1. mesh mode with a default `3 x 3` grid
2. static mesh raster
3. visible overlay lines and handles

Tasks:

1. add mesh model types
2. add mesh raster code
3. add mesh preview overlay
4. add mesh point list panel
5. route color picker to selected point

Exit criteria:

1. mesh mode renders a colored `3 x 3` field
2. overlay points and connecting lines appear correctly

## Milestone 6: Mesh dragging

Goal:

1. all mesh points draggable
2. preview rerenders continuously
3. grid stays connected

Tasks:

1. add pointer hitboxes for points
2. add selected point state
3. add drag lifecycle
4. add monotonic row/column clamping
5. invalidate preview cache during drag

Exit criteria:

1. moving a point updates overlay and preview immediately
2. neighboring segments stay connected
3. cells do not invert

## File-Level Plan

### `apps/color-viz/src/gradient_builder/builder.rs`

Responsibilities after this project:

1. top-level tab state:
   - gradients
   - mesh
2. top-level gradient type state inside the gradients tab:
   - linear
   - radial
   - angular
3. shared shell composition
4. type-specific subpanels
5. preview cache ownership
6. selected stop or selected mesh point ownership

Planned additions:

1. `selected_tab`
2. `selected_mesh_point`
3. `mesh_grid`
4. type-specific preview dispatch
5. type-specific details panel rendering

### `apps/color-viz/src/gradient_builder/paint.rs`

Responsibilities after this project:

1. shared sampling helpers
2. linear preview helpers
3. radial preview helpers
4. angular preview helpers
5. top-level raster dispatch

This file should not absorb mesh editing UI logic.

### `apps/color-viz/src/gradient_builder/color.rs`

Responsibilities after this project:

1. format per-type CSS or debug output
2. keep formatting separate from raster math

### New file: `apps/color-viz/src/gradient_builder/mesh.rs`

Suggested responsibilities:

1. mesh data types
2. default grid construction
3. point lookup helpers
4. clamping and ordering helpers

### New file: `apps/color-viz/src/gradient_builder/mesh_paint.rs`

Suggested responsibilities:

1. mesh raster generation
2. guide-line paint helpers
3. point placement helpers

### Optional new file: `apps/color-viz/src/gradient_builder/mesh_view.rs`

Suggested responsibilities:

1. overlay element construction
2. point-handle UI helpers
3. point-list row rendering

This file is optional, but likely useful once mesh mode grows.

## Validation Plan

## Manual checks for phase 1

### Linear

1. existing linear mode still works
2. quads/render/render-async still behave as expected
3. CSS string still matches preview direction

### Radial

1. switching to radial updates preview immediately
2. moving stops updates radial bands correctly
3. adding stops works
4. deleting stops works
5. angle control is hidden or disabled

### Angular

1. switching to angular updates preview immediately
2. angle rotates the seam
3. adding a stop near `0%` and another near `100%` wraps correctly
4. render sync and async both work

## Manual checks for mesh

1. mesh mode shows `3 x 3` points
2. lines between adjacent points stay visible
3. dragging a corner point deforms the preview
4. dragging a center point deforms the preview
5. selected point color edits update the fill
6. points cannot leave the preview bounds
7. points cannot cross enough to invert the grid if monotonic clamps are enabled

## Suggested tests

The app is heavily visual, but some pure logic should still be unit tested.

### Add unit tests for

1. radial sampler edge cases
2. angular wrap sampler edge cases
3. preview cache key equality with differing gradient types
4. mesh monotonic clamp logic
5. mesh default-grid construction
6. mesh center-point offset seed behavior

### Do not over-invest in

1. snapshotting every raster result
2. brittle pixel-perfect image assertions for the whole preview

The highest-value tests are around geometry and sampling invariants.

## Performance Notes

1. Linear already has a useful split between direct paint and raster.
2. Radial, angular, and mesh should lean on the raster paths.
3. Mesh will be the heaviest mode and should be treated as the path most likely to need progressive rendering later.

If performance becomes an issue:

1. snapshot inputs on the main thread
2. raster raw image data off-thread
3. swap `RenderImage` on the main thread
4. use generation/version checks to drop stale work

That matches the existing direction already discussed around `color-viz` preview rendering.

## Risks

### Risk 1: trying to unify mesh and stops too early

Avoid this.

The shell can be shared, but the model should stay separate.

### Risk 2: allowing mesh points to cross

This will complicate everything:

1. guide-line drawing
2. cell lookup
3. interpolation
4. user expectations

Use monotonic constraints early.

### Risk 3: hiding the actual active renderer

The diagnostics panel should stay explicit about:

1. requested renderer
2. actual renderer used

That matters once more modes are raster-only.

### Risk 4: treating Apple mesh gradients as a fidelity target

The requested behavior is the editable connected-grid experience, not pixel-identical parity with Apple's proprietary implementation.

Ship:

1. connected grid
2. per-point color interpolation
3. stable dragging

Do not block the feature on exact platform-matching shading behavior.

## Recommended Implementation Order

1. Add the top-level tabs and keep `Mesh` empty.
2. Generalize type-based dispatch without changing linear behavior.
3. Add radial.
4. Add angular.
5. Add mesh data model.
6. Add mesh static preview.
7. Add mesh overlay.
8. Add mesh dragging.
9. Add mesh constraints tuning.

This order keeps each step reviewable and preserves a working app between milestones.

## Final Recommendation

Treat this as an app-local rendering/editor project, not a shared-control project.

The right seam is:

1. add the tabbed shell in phase 1
2. keep the `Gradients` tab focused on stop-based modes
3. extend the preview raster pipeline for radial and angular
4. add a separate mesh model and overlay editor in the `Mesh` tab for phase 2

The most important implementation decisions are:

1. add the tabbed shell in phase 1 instead of waiting for mesh
2. keep GPUI-native logic only for linear
3. force radial/angular/mesh through app-side raster generation
4. keep mesh topology fixed at `3 x 3` for phase 2
5. defer configurable mesh dimensions to phase 3
6. enforce monotonic point constraints so the grid remains connected and sane while dragging

If this plan is followed, `color-viz` can grow into a much stronger gradient playground without forcing unstable gradient abstractions into the SDK too early.
