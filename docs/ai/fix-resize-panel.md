# Design Spec: Resizable Panels Upgrade (Mixed Sizing & Visual DSL)

This document describes the current architectural limitations of the `ResizablePanels` control and outlines the design for upgrading it to support mixed sizing (pixels + proportional weights) and a clean, visual builder/macro usage surface.

---

## 1. Current Architectural Issues

The current implementation of `ResizablePanels` under [`crates/sdk/src/controls/resizable_panels/`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/resizable_panels/) has several design gaps that affect usability and layout predictability:

1. **Purely Percentage-Based Sizing (`0.0..=100.0`)**:
   Every panel is sized as a percentage float. The layout engine normalizes all panel sizes to sum to `100.0` (see [`math.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/resizable_panels/math.rs#L22-L24)). 
2. **Unpredictable Window Resizing**:
   For standard application shells (like the Theme Studio sidebar split), a fixed-pixel width is desired for the sidebar. Because `ResizablePanels` only supports percentages, resizing the main window causes the sidebar to scale proportionally, leading to unwanted stretching or squishing.
3. **High Sizing Drift & Boilerplate**:
   Developers must manually ensure that all sibling panels' sizes add up to exactly 100.0. If one panel's default size is updated, all adjacent panel declarations must be edited in tandem to prevent the normalization engine from distorting the intended widths.
4. **Opaque constraints**:
   Defining minimum and maximum boundaries as percentages (e.g., `.min_size(18.0)`) is highly unintuitive compared to expressing constraints in pixels (e.g., `min: px(200.0)`).

---

## 2. Target Design: Mixed Sizing & Proportional Weights

To combine the layout power of React's `react-resizable-panels` with the readability of WPF's Grid systems, Luma's resizable layout engine should support **mixed sizing modes** concurrently in the same pane strip.

### Sizing Primitives
Replace raw `f32` size properties with a strongly typed enum:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PanelSize {
    /// Exact width or height in pixels
    Absolute(gpui::Pixels),
    /// Proportional "star" weight (shares remaining space after absolute panes)
    Weight(f32),
}
```

### The Two-Pass Layout Solver
The layout calculation in [`template.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/resizable_panels/template.rs) is updated to a two-pass algorithm:

```
┌────────────────────────────────────────────────────────┐
│                      Total Width                       │
├───────────────┬───┬────────────────────────────────────┤
│   Sidebar     │ ║ │            Workspace               │
│   (280 px)    │ ║ │             (Fill)                 │
└───────┬───────┴─┬─┴──────────────────┬─────────────────┘
        │         │                    │
        ▼         ▼                    ▼
     Pass 1    Pass 1                Pass 2
   (Subtract) (Subtract)     (Allocates remaining)
```

1. **Pass 1 (Absolute & Spacer allocation)**: 
   Read total available width/height. Subtract the widths of all `PanelSize::Absolute(px)` panels and all interactive handles (`handle_size`).
2. **Pass 2 (Weighted distribution)**:
   Sum all `PanelSize::Weight` tokens in the pane list. Allocate the remaining pixels proportionally to each weighted panel.

---

## 3. Usage Surface: The Fluent Builder

The builder API is redesigned to allow defining sizing rules directly on the parent builder chain, eliminating the need to manually build separate `ResizablePanelSpec` vectors.

```rust
// Proposed Fluent Builder API:
let main_split = ResizablePanels::horizontal("theme-studio-main-split")
    .orientation(ResizablePanelsOrientation::Horizontal)
    
    // Panel 0: Left Sidebar (pixel-constrained)
    .pane(sidebar_view)
        .size(px(280.0))
        .min(px(200.0))
        .max(px(400.0))
        
    .split_handle() // Draggable divider separating the adjacent panels
    
    // Panel 1: Main Content (fills all remaining space)
    .pane(board_view)
        .weight(1.0)
        
    .spawn(cx);
```

---

## 4. Usage Surface: The Visual DSL Macro

To give developers a crystal-clear, schematic representation of the layout structure, we introduce a declarative `resizable_panels!` macro. This maps visual syntax to the underlying builder blocks.

### The Macro Syntax
```rust
resizable_panels! {
    cx,
    id: "theme-studio-main-split",
    layout: Horizontal,
    panes: [
        // Format: View => Size, [Constraints]
        sidebar_view => px(280.0), min: px(200.0), max: px(400.0),
        
        | // Draggable separator handle
        
        board_view => weight(1.0)
    ]
}
```

### Composing Nested Split Layouts:
Nesting horizontal and vertical resizable splits becomes highly readable and matches the physical structure of the interface:

```rust
resizable_panels! {
    cx,
    id: "main-split",
    layout: Horizontal,
    panes: [
        // Left Column: Navigation Sidebar
        sidebar_view => px(250.0), min: px(180.0), max: px(350.0),
        |
        // Right Column: Split editor/terminal workspace
        resizable_panels! {
            cx,
            id: "workspace-split",
            layout: Vertical,
            panes: [
                editor_pane => weight(2.0),
                |
                terminal_pane => weight(1.0)
            ]
        } => weight(1.0)
    ]
}
```

---

## 5. Verification Plan & Test Strategy

To verify the mixed-sizing mathematical solver and visual macro correctness, we will port the layout pages from the Opal alpha as full-viewport integration testbeds inside the Luma gallery.

### Automated Tests
* Implement unit tests in `math.rs` validating mixed `Absolute(px)` and `Weight(f32)` distribution under varying window sizes.
* Add edge-case tests verifying that `min` and `max` constraints (both pixel and percentage-based) clamp correctly during split drag-move calculations.

### Manual Verification (App Shell Demos)
Port the following Opal gallery layouts to `apps/gallery/src/gallery/panes/resizable_panels/` as full-size workspace templates:

1. **Layered Inset Shell**:
   * Uses an `Absolute(px)` nav pane split and an inset rounded content canvas.
   * **Target**: Verify that margins and frame widths remain stable during window scaling.
2. **Detached / Floating Shell**:
   * Separates the sidebar and content pane with a visible visual gap.
   * **Target**: Verify that drag handles align correctly in the gap and maintain target spacing.
3. **Icon Rail Collapse**:
   * A collapsible split pane that collapses to an `Absolute(px(56.0))` icon rail rather than hiding.
   * **Target**: Verify that the sidebar content renders correctly at narrow icon-rail widths.
4. **Nested Workspace (IDE Shell)**:
   * Uses the visual `resizable_panels!` macro to define a 3-pane layout (file explorer sidebar, center editor, bottom terminal).
   * **Target**: Verify nested horizontal/vertical propagation and handle alignment.

