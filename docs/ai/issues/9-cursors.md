# Issue #9: Splitter Cursor States on macOS

## Description

When dragging or hovering resize splitters in the app shell, the current implementation uses the generic platform resize cursors via:

- `cursor_col_resize()`
- `cursor_row_resize()`

On macOS, these render as the normal black-filled resize cursor. This differs from some native or semi-native split views where the cursor visually communicates constraints: when a panel can no longer shrink or expand in one direction, the arrow on that side appears dimmed/gray.

The current `DockSplitter` / `ResizablePanels` splitters do **not** provide that directional constraint feedback.

## Current State

- The splitter visuals themselves are custom GPUI controls.
- The cursor is the standard platform resize cursor.
- Width/height constraints are enforced in app/control logic.
- The OS cursor does not know about those app-level min/max bounds, so it cannot automatically gray one side.

## Problem Summary

We want cursor feedback that matches panel constraints more closely, especially on macOS:

- normal resize when movement is possible both ways
- reduced / disabled-looking cursor direction when the panel is at min or max bounds

With the current custom splitter approach, the platform only sees "show a resize cursor," not a native constrained split view.

## Why This Is Hard

The gray-arrow behavior appears to come from native platform splitters that integrate:

- divider position
- container bounds
- min/max constraints
- system cursor rendering

Our splitters are custom controls, so:

- layout constraints live in Rust state
- cursor rendering lives in the platform
- there is no automatic bridge between them

## Possible Solutions

### 1. Keep the standard platform resize cursor

**Simplest and current behavior.**

Pros:
- No extra implementation complexity
- Uses expected OS resize cursors
- Works cross-platform

Cons:
- Does not communicate one-sided constraint state
- Less polished than native splitters in some apps

### 2. Add custom cursor assets per constraint state

Detect splitter state and swap cursor images dynamically.

Possible cursor variants:
- horizontal resize
- horizontal at min
- horizontal at max
- vertical resize
- vertical at min
- vertical at max

Pros:
- Can approximate native constrained-cursor feedback
- Full control over visuals

Cons:
- Requires custom cursor support in GPUI/platform layer
- Requires image assets + hotspot definitions
- Must handle platform differences explicitly
- Will still be an approximation, not a true native system splitter

### 3. Add stronger in-control visual feedback instead of cursor changes

Keep the generic OS cursor, but make the splitter itself communicate constraints better.

Examples:
- dim the splitter when pinned at a bound
- change splitter thumb color at min/max
- show a subtle end-stop affordance on hover/drag

Pros:
- Fully implementable in current custom control architecture
- No custom cursor API required
- Portable across platforms

Cons:
- Does not change the cursor itself
- Different feel than native constrained cursors

### 4. Use native platform splitters where available

Replace custom splitters with a true native host control on platforms that support it.

Pros:
- Best chance of getting exact native cursor behavior
- Potentially better platform fidelity overall

Cons:
- Much more complex architecturally
- Harder to keep cross-platform parity
- Pushes layout behavior out of the current GPUI/custom-control model
