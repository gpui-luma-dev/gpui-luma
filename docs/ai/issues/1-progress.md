# Progress Controls Architecture: Linear Progress

## Status

**Proposed Architecture & Design Document**

This document outlines the architectural plan for extending the `gpui-luma` SDK progress control subsystem (`crates/sdk/src/controls/progress/`) to support static **Linear Progress** indicators. 

> [!NOTE]
> For the discrete multi-step stepper control, see [1-stepper.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/ai/issues/1-stepper.md).

---

## 1. Problem Statement & Objectives

Currently, `crates/sdk/src/controls/progress/` provides a single circular canvas-based progress ring (`ThemedProgressTemplate`). Downstream applications require a static **Linear Progress** bar with:

- Static progress bar indicator (non-interactive, zero focus/hover/drag handlers).
- Filled circle thumb option (`show_thumb: bool`).
- Clean template separation: distinct templates (`LinearProgressTemplate` vs `CircularProgressTemplate`) rather than conditional `if/else` branching within a single template.
- Full 4-axis direction support: Left-to-Right (LTR), Right-to-Left (RTL), Top-to-Bottom (TTB), and Bottom-to-Top (BTT).
- Horizontal and Vertical orientations.

---

## 2. Architectural Design (LMTP Split)

Adhering strictly to section 2.1 of `docs/architecture.md`, the design maintains a strict separation across Model, Control, Template, and Theme boundaries.

```
┌─────────────────────────────────────────────────────────────┐
│                        MODEL                                │
│   ProgressModel (Value, Range, Direction, Show Thumb)      │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│                       CONTROL                               │
│   ProgressControl (GPUI Entity)                             │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│                       TEMPLATE                              │
│   CircularProgressTemplate / LinearProgressTemplate          │
│   (Stateless Div / Canvas Builders)                         │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│                        THEME                                │
│   ProgressTheme / ProgressLook (Look Resolution)            │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Component Specifications

### 3.1 Direction & Orientation Data Types

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ProgressOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ProgressDirection {
    #[default]
    LeftToRight,
    RightToLeft,
    BottomToTop,
    TopToBottom,
}

impl ProgressDirection {
    pub fn orientation(self) -> ProgressOrientation {
        match self {
            Self::LeftToRight | Self::RightToLeft => ProgressOrientation::Horizontal,
            Self::BottomToTop | Self::TopToBottom => ProgressOrientation::Vertical,
        }
    }

    pub fn is_reversed(self) -> bool {
        matches!(self, Self::RightToLeft | Self::TopToBottom)
    }
}
```

### 3.2 Render Model (`ProgressRenderModel`)

```rust
pub struct ProgressRenderModel<'a> {
    pub id: &'a SharedString,
    pub range: ControlRange,
    pub value: f32,
    pub percentage: f32,
    pub size: ControlSize,
    pub enabled: bool,
    pub direction: ProgressDirection,
    pub show_thumb: bool,
}
```

### 3.3 Builder API Extensions (`ProgressBuilder`)

```rust
impl ProgressBuilder {
    pub fn direction(mut self, direction: ProgressDirection) -> Self;
    pub fn orientation(mut self, orientation: ProgressOrientation) -> Self;
    pub fn show_thumb(mut self, show_thumb: bool) -> Self;
    pub fn linear(self) -> Self; // Convenience to assign LinearProgressTemplate
    pub fn circular(self) -> Self; // Convenience to assign CircularProgressTemplate
}
```

---

## 4. Progress Template Separation

To ensure modularity and avoid conditional branching inside template rendering:

1. **`CircularProgressTemplate`** (`crates/sdk/src/controls/progress/template/circular.rs`):
   - Paints circular progress rings using GPUI canvas arc paths.

2. **`LinearProgressTemplate`** (`crates/sdk/src/controls/progress/template/linear.rs`):
   - Renders a linear track container, filled progress bar segment, and optional static circular thumb badge.
   - Zero event listeners or focus handlers attached (purely static indicator).

```rust
pub trait ProgressTemplate: Send + Sync {
    fn render(&self, model: &ProgressRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<Div>;
}
```

### Layout & Geometry Matrix for `LinearProgressTemplate`

| `ProgressDirection` | Layout Flow | Fill Edge Origin | Thumb Alignment Offset |
| :--- | :--- | :--- | :--- |
| **`LeftToRight`** | `hstack` / `w_full` | Starts at Left `0%` | `left = percentage * track_length - thumb_radius` |
| **`RightToLeft`** | `hstack` / `w_full` | Starts at Right `100%` | `right = percentage * track_length - thumb_radius` |
| **`BottomToTop`** | `vstack` / `h_full` | Starts at Bottom `0%` | `bottom = percentage * track_length - thumb_radius` |
| **`TopToBottom`** | `vstack` / `h_full` | Starts at Top `100%` | `top = percentage * track_length - thumb_radius` |

---

## 5. Theme & Look Integration

### 5.1 Theme Token Extensions (`crates/sdk/src/controls/progress/theme.rs`)

```rust
#[derive(Clone, Copy, Debug)]
pub struct ProgressLook {
    pub track_color: Hsla,
    pub progress_color: Hsla,
    pub thumb_color: Hsla,
    pub track_height: f32,
    pub thumb_size: f32,
    pub size: f32,
    pub stroke_width: f32,
}
```

### 5.2 Look Crate Integration (`crates/look-shadcn`)
Extend `ShadcnLookControlExt` and stylesheet configuration to resolve semantic metrics and colors:
- Track colors resolve from stylesheet `muted` and `primary` variables.
- Track height and thumb metrics scale gracefully across `ControlSize::Sm`, `ControlSize::Md`, and `ControlSize::Lg`.

---

## 6. File & Module Structure

```
crates/sdk/src/controls/progress/
├── mod.rs                # Exports Progress public API
├── model.rs              # ProgressModel, ProgressBuilder, ProgressRenderModel
├── control.rs            # ProgressControl entity
├── theme.rs              # ProgressTheme, ProgressLook
└── template/
    ├── mod.rs            # ProgressTemplate trait definition
    ├── circular.rs       # Canvas arc ring template
    └── linear.rs         # Linear track + fill + optional thumb template
```

---

## 7. Verification & Validation Plan

### 7.1 Automated Testing (`cargo test`)
- **Range & Clamping**: Verify values under `0.0` or over `100.0` clamp cleanly across all 4 directions.
- **Direction Inversion**: Assert percentage position formulas produce correct values for LTR (`val`), RTL (`1 - val`), BTT (`val`), and TTB (`1 - val`).

### 7.2 Visual Gallery Demos (`apps/gallery`)
Add a new exposition card in `apps/gallery` featuring:
- **Linear Progress Matrix**: Grid showing horizontal (LTR, RTL) and vertical (BTT, TTB) bars with toggles for `.show_thumb(true/false)`.
