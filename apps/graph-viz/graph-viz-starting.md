# Garmin Ride Activity Visualizer - Getting Started

This guide outlines the plan to build the ride activity visualizer inside the `apps/graph-viz` application. It details the data formats, parsing libraries, data models, GPUI architectural integration, and custom widget design to ensure complete alignment with the `gpui-luma` SDK and theming system.

---

## 1. Data Formats & Sample Assets

We have sample activity data for a single ride stored in three formats under [apps/graph-viz/assets/garmin-data/](file:///Users/scg/Developer/GitHub/gpui-luma/apps/graph-viz/assets/garmin-data):
*   `23386792539_ACTIVITY.fit` (Binary FIT format) — **Primary data source**
*   `activity_23386792539.gpx` (XML GPX format)
*   `activity_23386792539.tcx` (XML TCX format)

### Recommendation
Use **FIT** as the primary import source. It is the native format produced by Garmin devices and provides complete telemetry data (heart rate, cadence, power, temperature, speed, elevation, and GPS coordinates) in a compact size. Use **GPX** as a secondary option for simple route line imports.

---

## 2. Parsing Dependencies

To parse these formats in Rust, we will leverage the following crates:

```toml
[dependencies]
# For parsing Garmin .fit binary files
fitparser = "0.7"
# For parsing XML-based .gpx files
gpx = "0.8"
# Serde is required for serialization/deserialization integration
serde = { version = "1.0", features = ["derive"] }
```

---

## 3. Data Abstraction Layer

To isolate parsing details from our UI code, we will introduce an app-local data abstraction in `apps/graph-viz/src/graph/activity/`:

```rust
// activity/model.rs
use gpui::Point;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RideSummary {
    pub total_distance_meters: f32,
    pub total_duration_seconds: f32,
    pub elevation_gain_meters: f32,
    pub avg_speed_mps: f32,
    pub avg_heart_rate: Option<f32>,
    pub avg_power: Option<f32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TelemetryPoint {
    pub timestamp_seconds: f32, // Offset from ride start
    pub location: Option<Point<f32>>, // Latitude & Longitude projected coords
    pub altitude_meters: Option<f32>,
    pub speed_mps: Option<f32>,
    pub heart_rate_bpm: Option<u8>,
    pub cadence_rpm: Option<u8>,
    pub power_watts: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct RideActivity {
    pub summary: RideSummary,
    pub points: Vec<TelemetryPoint>,
}
```

---

## 4. UI Architecture & SDK Alignment

The `graph-viz` app will structure its user interface strictly around the `gpui-luma` SDK and lookless styling boundaries.

### 4.1 Composing SDK Controls
All interface widgets must be created using the `ShadcnLook` extension builders:
*   **Vertical Scrolling Layout:** A tabbed interface can come later. For now, stack all graph panels, summary metrics, and maps vertically (`vstack!`) inside a scrollable main window viewport.
*   **Selections:** Use `Selector` in the sidebar or control header to choose or filter which telemetry metrics (e.g., Heart Rate, Elevation, Power) are plotted in the stack.
*   **Timeline Scrubbing:** Use a unified `Slider` to scrub through the ride timeline, syncing the highlight crosshairs/tooltips across all stacked graphs simultaneously.

### 4.2 Layout Macro Enforcement
In compliance with the workspace rules, do not write raw multi-child flex chains. All layouts must use `vstack!`, `hstack!`, or `wrappanel!`:

```rust
// Correct: Vertical layout of summary cards
let summary_layout = vstack! {
    gap=12.0 align=stretch;
    distance_card,
    elevation_card,
    speed_card,
};
```

---

## 5. Theming & Custom Graphs

To create visual graphs (line charts, maps) that theme correctly:
1.  **Read Colors from Theme:** Custom elements must query their palette colors directly from the active `ShadcnLook` theme.
2.  **No Hardcoded Color Constants:** Use theme tokens for paths, gridlines, text, and backgrounds:

```rust
// Query look details during render/paint
let chrome = look.chrome();
let border_color = chrome.border; // Grid lines color
let accent_color = look.token_color("primary").unwrap_or(chrome.title_text); // Graph stroke color
```

### 5.1 Telemetry Chart Component (Element)
For high-performance rendering of lines and routes, implement a custom GPUI `Element`:

```rust
pub struct TelemetryChartElement {
    points: Vec<Point<f32>>,
    stroke_color: Hsla,
    grid_color: Hsla,
}

impl gpui::Element for TelemetryChartElement {
    type RequestLayoutState = ();
    type PaintState = ();

    fn request_layout(
        &mut self,
        _id: Option<&gpui::ElementId>,
        cx: &mut gpui::WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        // Compute dimensions and request layout bounding box
        let layout_id = cx.request_layout(&gpui::Style::default(), None);
        (layout_id, ())
    }

    fn paint(
        &mut self,
        _id: Option<&gpui::ElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _paint_state: &mut Self::PaintState,
        cx: &mut gpui::WindowContext,
    ) {
        // 1. Draw background panel & grid lines using `grid_color`
        // 2. Scale telemetry points (time -> x, value -> y) into `bounds`
        // 3. Draw visual stroke path using `stroke_color`
    }
}
```

---

## 6. Promotion to SDK

By isolating the graphing/charting logic inside self-contained GPUI elements (`TelemetryChartElement`, `RouteMapElement`) and utilizing lookless tokens, the graphing controls will complement the SDK cleanly. 

Once these visual components are validated within the `graph-viz` app shell, they can easily graduate to a generic charting suite in `crates/sdk/src/controls/charts/` by separating the Garmin domain models from the primitive line-drawing logic.
