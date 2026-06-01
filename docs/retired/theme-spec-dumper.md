# Design Spec: HTML Theme & Metrics Dumper (Switch Pilot)

This document details the architecture for the HTML Theme & Metrics Spec Dumper, scoped strictly to the **Switch** control as the initial pilot implementation. The design prioritizes **zero metadata drift**, **strict modularity**, and **minimal boilerplate** by combining a co-located specification definition, linker-based compiler registration (`linkme`), and dynamic `serde` JSON serialization.

---

## 1. The Unified Spec Contract: `LumaControlSpec`

Instead of separating visual documentation from layout metrics, all specifications are unified under a single trait. This ensures that the Switch control describes both its styling mappings (for theme resolution) and its spatial metrics (for sizing visual verification).

```rust
// theme/spec.rs

pub trait LumaControlSpec: Send + Sync {
    /// The component's human-readable name (returns "Switch")
    fn name(&self) -> &'static str;

    /// The static Radix/Shadcn CSS variable mapping list (representing visual parts)
    fn theme_usages(&self) -> &'static [ThemePartUsage];

    /// Serializes the control's layout metrics for a given size into JSON values
    fn export_metrics(
        &self, 
        size: ControlSize, 
        tokens: &ThemeTokens, 
        scale_factor: f32
    ) -> serde_json::Value;
}
```

---

## 2. Linker-Based Registration (`linkme`)

To completely eliminate the maintenance cost of a central registry file, the dumper utilizes **linker-section registration** via the `linkme` crate. This collects specifications at compile-time automatically.

### Global Registry Hook
The registry declares a distributed slice. When the `SwitchSpec` is compiled, it automatically registers itself to this slice:

```rust
// theme/spec/registry.rs

#[linkme::distributed_slice]
pub static ALL_CONTROL_SPECS: [&'static dyn LumaControlSpec];
```

---

## 3. Reference Co-located Implementation: Switch Spec

The spec implementation lives directly inside the Switch-specific styling file (**`theme/radix/switch.rs`**), right below its Markdown documentation table and color resolvers.

### Associated Switch Styling Structs
To decouple structural layouts, we define `SwitchPalette` (for stateful colors) and `SwitchScale` (for physical proportions):

```rust
// controls/switch/theme.rs

/// Stateful visual assets returned by the theme engine
#[derive(Clone, Debug)]
pub struct SwitchPalette {
    pub track_background: Hsla,
    pub track_border: Hsla,
    pub thumb_background: Hsla,
    pub thumb_border: Hsla,
    pub thumb_shadow: Vec<BoxShadow>,
    pub label_color: Hsla,
    pub adorner: Option<AdornerSpec>,
    pub label_typography: LumaTextStyle,
    pub label_font_family: SharedString,
}

/// Dynamic physical dimensions computed by the SDK layout formulas
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct SwitchScale {
    pub width: f32,
    pub height: f32,
    pub thumb_size: f32,
    pub padding: f32,
    pub gap: f32,
    pub radius: f32,
    pub label_baseline_shift: f32,
}
```

### Spec Implementation Block
```rust
// theme/radix/switch.rs

use crate::theme::spec::{LumaControlSpec, ThemePartUsage, ALL_CONTROL_SPECS};
use crate::theme::{ControlSize, ThemeTokens};
use crate::controls::switch::theme::SwitchPalette;
use crate::controls::switch::layout::SwitchScale;

pub struct SwitchSpec;

impl LumaControlSpec for SwitchSpec {
    fn name(&self) -> &'static str {
        "Switch"
    }

    fn theme_usages(&self) -> &'static [ThemePartUsage] {
        &[
            part("off track", "input", &["off"], &["SwitchPalette.track_background"]),
            part("track border", "border", &["off", "disabled"], &["SwitchPalette.track_border"]),
            part("on track", "primary", &["on"], &["SwitchPalette.track_background", "SwitchPalette.track_border"]),
            part("off thumb", "background", &["off"], &["SwitchPalette.thumb_background"]),
            part("thumb border", "border", &["off"], &["SwitchPalette.thumb_border"]),
            part("on thumb", "primary-foreground", &["on"], &["SwitchPalette.thumb_background", "SwitchPalette.thumb_border"]),
            part("focus ring", "ring", &["focused"], &["SwitchPalette.adorner"]),
        ]
    }

    fn export_metrics(
        &self, 
        size: ControlSize, 
        tokens: &ThemeTokens, 
        scale_factor: f32
    ) -> serde_json::Value {
        let scale = SwitchScale::compute(size, &tokens.metrics, scale_factor);
        serde_json::to_value(scale).unwrap_or(serde_json::Value::Null)
    }
}

// Compile-time registration: pushes the static SwitchSpec reference into the distributed slice
#[linkme::distributed_slice(ALL_CONTROL_SPECS)]
static SWITCH_SPEC_REGISTRATION: &'static dyn LumaControlSpec = &SwitchSpec;
```

---

## 4. Exporter Resolution Pipeline

The exporter runs as a command within the CLI utility (`crates/luma-theme`). Since HSL colors are runtime-dependent (varying by theme and mode), the exporter resolves tokens dynamically at export time before serialization.

```
┌────────────────────────────────────────────────────────┐
│                   Exporter CLI / Script                │
├────────────────────────────────────────────────────────┤
│  1. Loads target Radix CSS catalog or native tokens    │
│  2. Iterates over static `ALL_CONTROL_SPECS` slice     │
│  3. Resolves static CSS token names to HSL hex colors  │
│  4. Runs Switch metrics math for Sm/Md/Lg sizes        │
│  5. Embeds output JSON into standalone HTML styleguide │
└───────────────────────────┬────────────────────────────┘
                            │ Outputs
                            ▼
┌────────────────────────────────────────────────────────┐
│                   styleguide.html                      │
└────────────────────────────────────────────────────────┘
```

### Export Resolution Logic
```rust
// theme/spec/exporter.rs

pub fn generate_spec_dump(theme: &RadixTheme, scale_factor: f32) -> serde_json::Value {
    let mode_tokens = theme.mode_tokens();
    let mut dump = serde_json::Map::new();

    // Iterate over compile-time linked specs automatically (only Switch registers initially)
    for spec in ALL_CONTROL_SPECS {
        let mut comp_data = serde_json::Map::new();

        // 1. Resolve Colors at Export Time
        let mut resolved_usages = Vec::new();
        for part in spec.theme_usages() {
            let resolved_color = theme.token_color(part.token)
                .map(|color| format_compact_hsla(color))
                .unwrap_or_else(|_| "transparent".to_string());

            resolved_usages.push(json!({
                "part": part.part,
                "token": format!("--{}", part.token),
                "states": part.states,
                "color": resolved_color, // Actual HSL resolved at export time
                "target_fields": part.appearance_fields,
            }));
        }
        comp_data.insert("usages".to_string(), serde_json::Value::Array(resolved_usages));

        // 2. Resolve Metrics at Export Time
        let mut metrics_data = serde_json::Map::new();
        for size in &[ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
            let json_metrics = spec.export_metrics(*size, mode_tokens, scale_factor);
            metrics_data.insert(format!("{:?}", size), json_metrics);
        }
        comp_data.insert("metrics".to_string(), serde_json::Value::Object(metrics_data));

        dump.insert(spec.name().to_string(), serde_json::Value::Object(comp_data));
    }

    serde_json::Value::Object(dump)
}
```

---

## 5. HTML Document Layout & Dynamic SVG Previews

Because the exporter dumps the fully resolved Switch colors and size parameters into a single JSON object inside the HTML file, the styleguide requires no server backend and remains extremely responsive.

### Dynamic SVG Renderer
Inside the styleguide page, standard browser JavaScript maps JSON values directly to SVG parameters. For example, rendering the Switch dynamically:

```html
<!-- Renders a mathematically accurate Switch preview based on current theme scale -->
<div class="control-preview">
  <svg :width="metrics.active.width" :height="metrics.active.height">
    <!-- Switch Track -->
    <rect 
      :width="metrics.active.width" 
      :height="metrics.active.height" 
      :rx="metrics.active.radius" 
      :fill="colors.track_background" 
      :stroke="colors.track_border"
      stroke-width="1"
    />
    <!-- Switch Thumb -->
    <circle 
      :cx="checked ? (metrics.active.width - metrics.active.thumb_size/2 - metrics.active.padding) : (metrics.active.thumb_size/2 + metrics.active.padding)" 
      :cy="metrics.active.height / 2" 
      :r="metrics.active.thumb_size / 2" 
      :fill="colors.thumb_background"
      :stroke="colors.thumb_border"
      stroke-width="1"
    />
  </svg>
</div>
```

---

## 6. Verification & Validation Workflow

* **Visual Review:** Allows designers to instantly spot color mapping errors or sizing mismatches on the Switch without checking out code or running desktop window compiles.
* **Typographic Baseline Check:** Renders the text label next to the Switch SVG preview using the `label_baseline_shift` parameters to verify alignment.
* **Regression Protection:** Running a structural JSON diff on the exporter output block during refactoring makes it simple to ensure no metric calculations or layouts were broken in the process.
