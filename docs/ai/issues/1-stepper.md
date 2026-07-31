# Stepper Architecture & Specification

## Status

**Proposed Architecture & Design Document**

This document defines the architecture and specification for the discrete multi-step process visualization control (`StepperControl` / `Stepper`) in `crates/sdk/src/controls/stepper/`.

---

## 1. Problem Statement & Objectives

Complex user workflows (setup wizards, multi-stage checkout, onboarding tasks, multi-step forms) require a visual progress indicator that displays discrete steps.

### Requirements
- **Configurable Step Count**: Caller initializes the control with total steps (`step_count`).
- **3-State Step Resolution**:
  - `StepState::Complete` (Circle badge with Lucide Checkmark icon, primary/accent fill).
  - `StepState::InProgress` (Circle badge with step number, primary background).
  - `StepState::Incomplete` (Circle badge with step number, outlined/muted border).
- **Direction & Layout**: Support for Left-to-Right (LTR), Right-to-Left (RTL), Bottom-to-Top (BTT), and Top-to-Bottom (TTB).
- **Optional Step Labels**: Supporting titles/subtitles per step (`labels: Vec<SharedString>`).

---

## 2. Core Architectural Design (LMTP Split)

Following `docs/architecture.md` (LMTP Split):

```
┌─────────────────────────────────────────────────────────────┐
│                        MODEL                                │
│   StepperModel (State, Step Count, Step States, Labels)    │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│                       CONTROL                               │
│   StepperControl (GPUI Entity owned by app context)         │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│                       TEMPLATE                              │
│   StepperTemplate (Stateless Div Builders)                  │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│                        THEME                                │
│   StepperTheme / StepperLook (Token & Metric Resolver)      │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Data Types & Component Specification

### 3.1 Step States & Data Types
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StepState {
    Complete,
    #[default]
    InProgress,
    Incomplete,
}

#[derive(Clone, Debug)]
pub struct StepperItem {
    pub index: usize,
    pub state: StepState,
    pub label: Option<SharedString>,
}
```

### 3.2 Stepper Model & Render Model
```rust
#[derive(Clone)]
pub struct StepperModel {
    pub(crate) id: SharedString,
    pub(crate) step_count: usize,
    pub(crate) current_step: usize,
    pub(crate) step_states: Vec<StepState>,
    pub(crate) labels: Vec<SharedString>,
    pub(crate) direction: ProgressDirection,
    pub(crate) size: ControlSize,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn StepperTemplate>,
}

pub struct StepperRenderModel<'a> {
    pub id: &'a SharedString,
    pub step_count: usize,
    pub current_step: usize,
    pub step_states: &'a [StepState],
    pub labels: &'a [SharedString],
    pub direction: ProgressDirection,
    pub size: ControlSize,
    pub enabled: bool,
}
```

---

## 4. Visual Layout & Geometry Matrix

### 4.1 Step Circle Badges & Connecting Tracks
Each step node consists of:
1. **Circle Badge**:
   - `Complete`: Primary background color + `LucideIcon::Check` foreground.
   - `InProgress`: Primary background color + step number text (e.g. `"2"`).
   - `Incomplete`: Muted background + subtle border + muted step number text (e.g. `"3"`).
2. **Connector Track Segment**:
   - Connects step $i$ to step $i+1$.
   - Rendered with active primary progress color if step $i$ is complete or in progress; otherwise muted track color.

### 4.2 Layout Direction

| `ProgressDirection` | Connector Orientation | Node Alignment | Step Order |
| :--- | :--- | :--- | :--- |
| **`LeftToRight`** | Horizontal line | `hstack` | Left $\rightarrow$ Right |
| **`RightToLeft`** | Horizontal line | `hstack` reversed | Right $\rightarrow$ Left |
| **`BottomToTop`** | Vertical line | `vstack` reversed | Bottom $\rightarrow$ Top |
| **`TopToBottom`** | Vertical line | `vstack` | Top $\rightarrow$ Bottom |

---

## 5. Builder API Surface (`StepperBuilder`)

```rust
pub fn stepper(id: impl Into<SharedString>, step_count: usize) -> StepperBuilder;

impl StepperBuilder {
    pub fn current_step(mut self, current: usize) -> Self;
    pub fn step_state(mut self, step_index: usize, state: StepState) -> Self;
    pub fn labels(mut self, labels: Vec<impl Into<SharedString>>) -> Self;
    pub fn direction(mut self, direction: ProgressDirection) -> Self;
    pub fn size(mut self, size: ControlSize) -> Self;
    pub fn template(mut self, template: Arc<dyn StepperTemplate>) -> Self;
    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<StepperControl>;
}
```

---

## 6. Theme & Look Specs (`crates/sdk/src/controls/stepper/theme.rs`)

```rust
#[derive(Clone, Copy, Debug)]
pub struct StepperLook {
    pub complete_bg: Hsla,
    pub complete_fg: Hsla,
    pub in_progress_bg: Hsla,
    pub in_progress_fg: Hsla,
    pub incomplete_bg: Hsla,
    pub incomplete_border: Hsla,
    pub incomplete_fg: Hsla,
    pub track_active_color: Hsla,
    pub track_muted_color: Hsla,
    pub step_badge_size: f32,
    pub track_thickness: f32,
}

pub trait StepperTheme: Send + Sync {
    fn resolve(&self, enabled: bool, size: ControlSize) -> StepperLook;
}
```

---

## 7. File & Module Structure

```
crates/sdk/src/controls/
└── stepper/
    ├── mod.rs        # Exports `Stepper`, `StepperControl`, `stepper()`
    ├── model.rs      # StepperModel, StepperBuilder, StepperRenderModel
    ├── control.rs    # StepperControl entity
    ├── template.rs   # StepperTemplate trait & default themed template
    └── theme.rs      # StepperTheme & StepperLook
```

---

## 8. Verification & Test Plan

1. **Unit Tests**:
   - Verify step state calculation for boundary step counts (`current_step = 0`, `current_step = step_count - 1`).
   - Validate explicit custom `step_state` overrides.
2. **Gallery Integration (`apps/gallery`)**:
   - Interactive Wizard prototype with `Previous` and `Next` buttons stepping through a 4-step workflow.
