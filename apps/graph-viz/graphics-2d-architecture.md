# Composable 2D Graphics Architecture for GPUI-Luma

This document defines a composable, lookless architectural proposal for 2D graphing and visual metrics in the `gpui-luma` workspace. It draws inspiration from the mathematical abstraction layer of Luma's color system (`FieldDomain2D`, `Arc`) and standardizes on the LMTP (Model, Control, Template, Theme) split.

---

## 1. Core Abstractions: The Coordinate Pipeline

To decouple raw telemetry data from physical display layouts, the rendering pipeline separates coordinate mapping into two mathematical layers.

```text
  Data Coordinates             Normalized Space (UV)            Window Coordinates
  [(x_data, y_data)]   ──────► [0.0..1.0, 0.0..1.0]   ──────►   [Bounds<Pixels>]
                       (Domain)                       (Viewport)
```

> [!NOTE]
> **Y-Axis Inversion Rule:**
> `PlotDomain2D::to_uv` maps low data values to low UV coordinates (bottom of domain, i.e., UV Y = 0.0). `Viewport2D::uv_to_pixels` then handles the inversion to GPUI coordinate space where high screen pixels represent the bottom of the window. Implementers must be careful not to double-invert the Y-axis.

### 1.1 PlotDomain2D (Data to UV)
Represents the numeric boundaries of the data space. All data coords are defined as generic tuples `(f32, f32)` representing `(x_data, y_data)` to avoid confusion with physical GPUI layout points.

```rust
pub struct PlotDomain2D {
    pub x_range: std::ops::RangeInclusive<f32>,
    pub y_range: std::ops::RangeInclusive<f32>,
}

impl PlotDomain2D {
    /// Projects a data point (x, y) into a normalized UV coordinate (0.0..1.0)
    pub fn to_uv(&self, x: f32, y: f32) -> gpui::Point<f32> {
        let x_min = *self.x_range.start();
        let x_max = *self.x_range.end();
        let y_min = *self.y_range.start();
        let y_max = *self.y_range.end();

        gpui::point(
            (x - x_min) / (x_max - x_min.max(f32::EPSILON)),
            (y - y_min) / (y_max - y_min.max(f32::EPSILON)),
        )
    }
}
```

### 1.2 Viewport2D (UV to Pixels)
Transforms normalized UV coordinates into concrete pixel positions within a layout bounding box.

```rust
pub struct Viewport2D {
    pub bounds: gpui::Bounds<gpui::Pixels>,
}

impl Viewport2D {
    /// Maps a normalized UV point (0.0..1.0) to screen pixel coordinates
    pub fn uv_to_pixels(&self, uv: gpui::Point<f32>) -> gpui::Point<gpui::Pixels> {
        let x = self.bounds.origin.x + gpui::px(uv.x * self.bounds.size.width.as_f32());
        // Invert Y coordinate so UV Y = 0.0 is the bottom of the physical viewport bounds
        let y = self.bounds.origin.y + gpui::px((1.0 - uv.y) * self.bounds.size.height.as_f32());
        gpui::point(x, y)
    }
}
```

---

## 2. Composable Canvas Stack Pattern

Rather than custom `Element` implementations, visual graphics are built using GPUI's standard `canvas` element wrapper. Primitives are stacked as absolute layers inside a relative container.

```rust
// Drawing logic type signature
pub type GraphicDrawFn = Box<dyn Fn(&PlotDomain2D, &Viewport2D, &mut gpui::Window) + 'static>;
```

### 2.1 Domain Scales: Shared X, Per-Series Y
For multi-metric timelines (e.g., stacking heart rate, speed, and elevation charts):
*   **Shared X Domain:** All graphs share the same `x_range` (representing time elapsed).
*   **Per-Series Y Domain:** Each visualization layer/panel maintains its own distinct `y_range` domain limits.
*   **Scrub Synchronization:** Global interactions (like timelines or cursors) operate in normalized UV X coordinates (`0.0..1.0`), ensuring alignment across different Y scales.

### 2.2 Functional Drawing Abstractions

#### 1. GridLines (Ticks, Axes, Grid)
Paints axes and reference tick grids.
```rust
pub struct GridLinesLayer {
    pub x_ticks: usize,
    pub y_ticks: usize,
    pub grid_color: gpui::Hsla,
}
```

#### 2. LinePath (Line graphs)
Connects data coordinates with straight lines.
```rust
pub struct LinePathLayer {
    pub points: Vec<(f32, f32)>, // Original data coordinates (x_data, y_data)
    pub stroke_width: gpui::Pixels,
    pub color: gpui::Hsla,
}
```

#### 3. AreaFill & FillBetween (Envelope fills)
Fills the envelope between two boundary curves (mimicking Matplotlib's `fill_between` plot type).
```rust
pub struct AreaFillLayer {
    pub points: Vec<(f32, f32)>,
    pub fill_color: gpui::Hsla,
    pub baseline: f32,
}

pub struct FillBetweenLayer {
    pub points: Vec<(f32, f32, f32)>, // (x, y1_top, y2_bottom)
    pub fill_color: gpui::Hsla,
}
```

#### 4. Bar & BarLabel (Discrete / Time-binned charts)
Renders vertical bars and text labels corresponding to data buckets (mimicking Matplotlib's `bar` and `bar_label` features).
```rust
pub struct BarLayer {
    pub points: Vec<(f32, f32)>, // (x_center, y_value)
    pub bar_width: f32,          // Width in data X-axis units
    pub fill_color: gpui::Hsla,
    pub border_color: Option<gpui::Hsla>,
}

pub struct BarLabelLayer {
    pub points: Vec<(f32, f32)>,
    pub labels: Vec<String>,
    pub text_color: gpui::Hsla,
}
```

#### 5. InteractionCursor (Highlight & Tooltip)
Renders a vertical line at the active normalized scrub position.
```rust
pub struct InteractionCursorLayer {
    pub active_fraction: Option<f32>, // Shared normalized X position (0.0..1.0)
    pub cursor_color: gpui::Hsla,
}
```

---

## 3. Interaction Architecture: Controls Split

To cleanly coordinate user gestures, interaction responsibilities are split between local mouse events and global timeline progression:

```text
  Global TimelineControl (SDK Slider) ──► syncs: scrub_fraction (0.0..1.0)
                                                 │
                                                 ▼
  Local ChartControl (Hover/Hits)     ◄── updates tooltips/highlight markers
```

### 3.1 Local ChartControl
Owns localized chart-specific cursor states. Projects physical screen cursor coordinates to the local X data coordinate for tooltips:
```rust
pub struct ChartControl {
    domain: PlotDomain2D,
    hovered_point: Option<(f32, f32)>,
}

impl ChartControl {
    pub fn x_from_pointer(&self, pointer_x: gpui::Pixels, bounds: gpui::Bounds<gpui::Pixels>) -> f32 {
        let x_norm = ((pointer_x - bounds.origin.x).as_f32() / bounds.size.width.as_f32()).clamp(0.0, 1.0);
        let x_min = *self.domain.x_range.start();
        let x_max = *self.domain.x_range.end();
        x_min + x_norm * (x_max - x_min)
    }
}
```

### 3.2 Global TimelineControl
Reuses the SDK's existing `Slider` control. Dragging the slider publishes a `scrub_fraction: f32` (or time offset) to the workspace state, synchronizing the `InteractionCursorLayer` across all stacked charts simultaneously.

---

## 4. Performance & Downsampling

With larger data sets (e.g., 4,000+ points recorded over a multi-hour ride), drawing every raw point directly onto the canvas causes rendering bottlenecks.

### Downsampling Strategy
*   **Resolution-based Decimation:** Downsample data coordinates to match the screen's horizontal pixel width (e.g. 1 sample per pixel bounds).
*   **MinMax Binning:** Divide the X domain into bins corresponding to viewport columns, retaining only the minimum and maximum values per bin to prevent aliasing.
*   **Pre-layer execution:** Apply decimation to the data vectors *before* feeding coordinates to `LinePathLayer` or `BarLayer` to minimize paint path overhead.

---

## 5. Pluggable Theme Injection (App-Side Extension)

To ensure **zero impact** on the shared SDK and `look-shadcn` crates during development, the graphing visual states are resolved using a local extension trait implemented on `ShadcnLook`. 

This local mapping queries existing, standard design tokens from the theme to resolve graph colors:

```rust
// In apps/graph-viz/src/graph/theme.rs (or controls/theme.rs)
use gpui::Hsla;
use gpui_luma_look_shadcn::ShadcnLook;

pub struct ChartTheme {
    pub grid_line: Hsla,
    pub cursor_line: Hsla,
    pub heartrate_stroke: Hsla,
    pub speed_stroke: Hsla,
    pub elevation_stroke: Hsla,
}

pub trait GraphVizThemeExt {
    fn resolve_chart_theme(&self) -> ChartTheme;
}

impl GraphVizThemeExt for ShadcnLook {
    fn resolve_chart_theme(&self) -> ChartTheme {
        let chrome = self.chrome();
        ChartTheme {
            // Resolve using standard, existing fallback variables in look-shadcn
            grid_line: self.token_color("border").unwrap_or(chrome.border),
            cursor_line: self.token_color("ring").unwrap_or(chrome.title_text),
            heartrate_stroke: self.token_color("destructive").unwrap_or(chrome.title_text),
            speed_stroke: self.token_color("primary").unwrap_or(chrome.title_text),
            elevation_stroke: self.token_color("muted-foreground").unwrap_or(chrome.border),
        }
    }
}
```

---

## 6. Graduation Path to SDK

When the graphing system is promoted to the core SDK:
1.  **Move Primitives:** Move `PlotDomain2D`, `Viewport2D`, and standard graphic drawing closures to `crates/sdk/src/controls/charts/`.
2.  **Declare SDK Theme Trait:** Define a unified `ChartTheme` trait in the SDK.
3.  **Implement in Look Crate:** Implement `ChartTheme` in `crates/look-shadcn` by matching layout variables dynamically inside the look's `style.toml`.
4.  **Zero Drawing Logic Changes:** The drawing code in `GraphicLayer` remains identical because it operates only on raw `Hsla` values passed by the theme.
