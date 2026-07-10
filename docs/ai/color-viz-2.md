# Color Viz 2: Mesh Gradient Scope Reset and Execution Plan

## Purpose

This document replaces the earlier mesh-gradient plan with a tighter scope based on the live `apps/color-viz` prototype.

The linear, radial, and angular work is already in place. The next planning problem is the mesh editor: what behavior matters next, what should stay fixed, and what should explicitly wait.

This remains app-local work in `apps/color-viz`. It is not an SDK generalization project.

## Current State

The app already has:

1. top-level `Gradients` and `Mesh` tabs
2. linear, radial, and angular stop-based gradients
3. a mesh preview path with draggable points
4. a tessellated mesh raster path that is usable but intentionally approximate
5. a current mesh UI built around a vertically tall point list

The current mesh prototype is good enough to continue from, but not finished enough to generalize blindly.

## Progress Snapshot: 2026-07-10

The document below started as a forward plan. Parts of that plan are now done and should be treated as current behavior, not future intent.

Completed so far:

1. the app has a separate `Mesh` tab instead of forcing mesh controls into the same surface as stop-based gradients
2. linear, radial, and angular gradients are already working in the `Gradients` tab
3. the mesh tab has draggable preview handles with live rerasterization
4. mesh dragging now releases correctly instead of getting stuck on mouse or trackpad interaction
5. the mesh scene has a first-class background color
6. the mesh renderer clears to that background before drawing the mesh fill
7. border points can move away from the frame edge without exposing transparent or unpainted output
8. the left-side color picker flow is hooked up for both mesh point colors and mesh background color
9. the preview handles no longer open the picker directly; color editing is intentionally routed through the left panel
10. a reset icon button restores the default startup state for the active mesh preset
11. the mesh tab supports concrete grid presets: `3 x 4`, `2 x 2`, `3 x 3`, `4 x 4`
12. the mesh tab supports aspect presets: `Fill`, `9:19`, `3:4`, `1:1`, `2:3`
13. preview framing now accounts for handle space instead of sizing the frame as if handles consumed no room
14. the most recent `Fill` sizing bug on the right edge was fixed by fitting against the inner preview content box rather than the outer padded shell

Still true:

1. the mesh raster path is still an approximation, not Apple-parity mesh shading
2. the point list is still vertically expensive
3. the current work is still app-local to `apps/color-viz`
4. this is still not a general mesh-topology editor

## Scope Reset: 2026-07-10

The next work should not be framed as "make arbitrary mesh dimensions."

That jumps over the real product issues:

1. edge-point behavior
2. rectangular fill behavior
3. supported grid presets
4. aspect-ratio framing
5. mesh-panel compaction

The immediate plan should solve those first.

## Refined Requirements

### 1. Edge-point fill behavior

Requirement:

1. A point on the edge of the grid should not visually clip.
2. The output should remain a fully filled rectangle.

Implication:

1. If border points are allowed to move inward, the mesh alone no longer guarantees full rectangular coverage.
2. The renderer therefore needs an explicit background-fill model.

Decision:

1. Add a mesh background color.
2. Clear the preview/image to that background color before rendering the mesh fill.
3. Render the mesh over that background.
4. Keep guide lines and handles as a separate overlay above both.

This is the simplest correct model for movable border points. It is also easy to explain.

### 2. Grid support should be preset-based, not fully arbitrary

Supported mesh grids for the next phase:

1. `2 x 2`
2. `3 x 3`
3. `4 x 4`

Do not jump to arbitrary row/column counts yet.

Why:

1. preset grids cover the immediate product need
2. drag constraints remain understandable
3. raster logic stays debuggable
4. UI density can be solved against known point counts

### 3. Aspect ratio is now part of the feature

Supported aspect ratios:

1. `9:19`
2. `3:4`
3. `1:1`
4. `2:3`

This affects:

1. preview framing
2. raster size policy
3. point placement normalization
4. how preset meshes are seeded and displayed

This should be treated as mesh configuration, not as a visual afterthought.

### 4. Mesh controls need UI compaction

Current problem:

1. the mesh point rows consume too much vertical space
2. the panel becomes scroll-heavy too early
3. higher point counts will make this worse immediately

Requirement:

1. compact the live mesh controls before or alongside preset expansion

Recommended first pass:

1. reduce row padding
2. collapse point metadata into a denser one-line or two-line layout
3. shorten repeated labels
4. remove large descriptive prose from the live panel

## Product Definition

The target mesh experience is:

1. a rectangular preview surface
2. a connected draggable lattice
3. colors attached to points
4. a background color attached to the mesh scene
5. continuous rerasterization while dragging
6. fixed supported grid presets
7. fixed supported aspect-ratio presets

This is not yet:

1. arbitrary topology editing
2. arbitrary row/column counts
3. a platform-native mesh implementation
4. a general export format

## Recommended Data Model

The next model should be explicit about mesh configuration.

Suggested app-local types:

```rust
pub struct MeshPoint {
    pub row: u8,
    pub col: u8,
    pub u: f32,
    pub v: f32,
    pub color: Hsla,
}

pub struct MeshGrid {
    pub rows: usize,
    pub cols: usize,
    pub points: Vec<MeshPoint>,
    pub background: Hsla,
    pub preset: MeshGridPreset,
    pub aspect_ratio: MeshAspectRatio,
}

pub enum MeshGridPreset {
    TwoByTwo,
    ThreeByThree,
    FourByFour,
}

pub enum MeshAspectRatio {
    NineByNineteen,
    ThreeByFour,
    OneByOne,
    TwoByThree,
}
```

Notes:

1. `rows` and `cols` can remain stored, but they should be derived from the preset for now.
2. `background` must be first-class if border points can move inward.
3. `u` and `v` should remain normalized to the preview frame.

## Rendering Policy

### Inner mesh rendering

Do not restart the internal mesh math from scratch unless necessary.

The current tessellated patch approach is adequate as the working baseline.

The next renderer step should focus on the outer-fill problem, not on chasing perfect Apple parity.

### Outer fill behavior

Required policy:

1. clear preview raster to `mesh.background`
2. render the deformed mesh on top
3. paint guide lines after the fill
4. paint handles in the UI overlay

This avoids clipping and removes ambiguity when a border point leaves the frame edge.

### Overlay behavior

The mesh overlay should continue to show:

1. horizontal connections
2. vertical connections
3. draggable handles

The overlay is part of the editing model, not decoration.

## Interaction Rules

### Dragging

Each point still needs:

1. selection
2. pointer-down capture
3. drag updates
4. release cleanup
5. live preview invalidation

### Ordering constraints

Monotonic row/column constraints should remain in place:

1. a point cannot cross its left/right neighbors in `u`
2. a point cannot cross its top/bottom neighbors in `v`
3. edge points clamp to valid outer ranges

That preserves a stable connected lattice and keeps the raster logic sane.

### Border points

Border points are allowed to move, but the product must decide whether that means:

1. they slide only along the border, or
2. they may move inward freely

Based on the latest requirement, the plan should assume:

1. border points may move inward
2. uncovered outer area is filled by `mesh.background`

That is the harder requirement, but it is also the clearer one.

## UI Shape

### Mesh controls panel

Recommended order:

1. grid preset selector
2. aspect ratio selector
3. renderer/diagnostics
4. background color row
5. compact point list
6. selected point details
7. reset button if needed

### Point list compaction

The point list should be redesigned around density.

Recommended row content:

1. point id like `P11`
2. color swatch
3. inline coordinates like `u 0.52  v 0.31`
4. selected-state styling

Avoid:

1. tall stacked labels
2. repeated verbose captions
3. large empty padding bands

## File-Level Direction

### `apps/color-viz/src/gradient_builder/builder.rs`

Likely responsibilities:

1. mesh preset selection state
2. mesh aspect-ratio selection state
3. mesh background color editing
4. compact point-list rendering
5. preview cache invalidation across preset/aspect/background changes

### `apps/color-viz/src/gradient_builder/paint.rs`

Likely responsibilities:

1. apply background fill before mesh rasterization
2. use current mesh points against the active aspect ratio
3. keep guide-line painting separate from the fill pass

### Optional app-local split

If the file grows too large, break mesh logic into:

1. `mesh.rs`
2. `mesh_paint.rs`
3. `mesh_view.rs`

Do that only if it reduces confusion. Do not split files just to satisfy symmetry.

## Revised Phase Plan

This phase list is updated to reflect what has already landed.

### Phase 2A: Mesh edge-fill stabilization

Status: Complete

Goal:

1. make border-point behavior correct and explainable

Work:

1. add mesh background color to the model
2. clear the raster to background before mesh rendering
3. verify border-point movement does not expose clipping or unpainted output
4. keep current grid fixed while this lands

Exit criteria:

1. dragging edge points never reveals clipping
2. uncovered areas fill with background color
3. the interaction remains stable

### Phase 2B: Preset grids and aspect ratios

Status: Complete

Goal:

1. support the concrete shapes you actually want without opening arbitrary topology work

Work:

1. add grid presets for `2 x 2`, `3 x 3`, `4 x 4` while keeping the earlier `3 x 4` sample preset
2. add aspect-ratio presets for `Fill`, `9:19`, `3:4`, `1:1`, `2:3`
3. define deterministic default point layouts and colors per preset
4. ensure cache keys and drag math update correctly
5. keep preview handles visible by budgeting space for them inside preview framing

Exit criteria:

1. preset switches rebuild the mesh predictably
2. aspect-ratio switches reframe the preview correctly
3. dragging remains correct across presets

### Phase 2C: Mesh UI cleanup

Status: Next active phase

Goal:

1. make the panel usable for the supported presets

Work:

1. compress the point-list rows
2. move verbose copy out of the live panel
3. ensure more points fit on screen without excessive scrolling
4. reduce wasted vertical space around preset/aspect/reset controls
5. decide whether selected-point details stay inline or move to a smaller dedicated row

Exit criteria:

1. the mesh controls remain usable at `4 x 4`
2. the selected point is still easy to identify
3. the panel is denser without becoming cryptic

### Phase 3: Rendering refinement and panel cleanup follow-through

After the preset/aspect work, the next practical phase is not arbitrary topology. It is quality and usability cleanup on the fixed supported feature set.

Goal:

1. improve the shipped mesh experience without reopening core scope

Work:

1. tighten panel density so `4 x 4` remains comfortable without heavy scrolling
2. clean up any remaining preview-edge spacing inconsistencies between `Fill` and fixed aspect presets
3. reduce obvious raster artifacts where possible without throwing away the current mesh renderer
4. make default preset seeding look intentional across all supported grids

Exit criteria:

1. the mesh tab feels stable and compact on all supported presets
2. handle framing is visually correct for `Fill` and fixed ratios
3. remaining render artifacts are acceptable for current product scope

### Phase 4: True generalization

Only after the preset-based version is stable should the app consider:

1. arbitrary rows
2. arbitrary columns
3. non-square presets beyond the initial list
4. dynamic point creation/removal
5. richer serialization/export behavior

## Validation Plan

### Manual checks

1. Drag a top, side, and corner border point inward and confirm the preview remains fully filled.
2. Change mesh background color and confirm the outer uncovered regions update correctly.
3. Switch between `2 x 2`, `3 x 3`, and `4 x 4` and confirm point indexing, selection, and drag constraints remain correct.
4. Switch between `9:19`, `3:4`, `1:1`, and `2:3` and confirm the preview reframes without breaking normalized drag behavior.
5. Confirm the compact panel still makes point selection and color editing obvious.

## Recommended Next Step

The next implementation step should be Phase 2C.

Why:

1. the correctness and configuration work is already in place
2. the biggest remaining product issue is control density and panel usability
3. this is smaller and safer than reopening renderer math immediately

Concrete next coding slice:

1. compact the mesh left panel
2. reduce vertical padding around point rows and info rows
3. keep preset, aspect, reset, and background controls visible without excessive scrolling
4. verify `4 x 4` remains usable after compaction

Do not combine that with arbitrary-grid generalization in the same pass.
