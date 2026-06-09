# Unified Stylesheet Mapping Architecture (`style.toml`)

> [!NOTE]
> **Status:** Architecture Design Document & Proposal  
> **Workspace Edition:** Rust 2024  
> **Objective:** Define the decoupled theming system for `look-shadcn` to replace hardcoded Rust styling resolvers and scattered look tables with a single, centralized stylesheet.

---

## 1. Rationale: Why Decouple Styling?

To achieve stylesheet-like discoverability, styling systems should separate **what a value is** (design tokens) from **how a control maps to it** (semantic mappings). 

Hardcoding layout structures, hover colors, and paddings in Rust files creates friction. By moving mappings out of code and into a structured stylesheet configuration (`style.toml`), we establish a **3-Tier Design System**:

```text
┌──────────────────────────────────────────────┐
│  Tier 1: Design Tokens (CSS Catalog)         │ <-- Theme Skin (Jarvis, Retro Arcade)
│  --primary: hsl(330 64% 52%); --radius: 6px   │     (Parsed from raw CSS files)
└──────────────────────┬───────────────────────┘
                       │ resolves variables
┌──────────────────────▼───────────────────────┐
│  Tier 2: Semantic Mappings (style.toml)      │ <-- SOURCE OF TRUTH
│  button.primary.default.background = "primary"│     (Defines component layout & style rules)
└──────────────────────┬───────────────────────┘
                       │ evaluates
┌──────────────────────▼───────────────────────┐
│  Tier 3: Control Templates (Rust Engine)      │ <-- Layout & Event Logic
│  Paints component using resolved variables   │     (Zero hardcoded styles, zero macros)
└──────────────────────────────────────────────┘
```

### Key Benefits
* **Theme Swapping without Recoding**: Changing the look from *Retro Arcade* to *Jarvis* is done by changing the loaded CSS design tokens file. The core mapping rules (`style.toml`) remain identical.
* **Component-Level Autonomy**: Tweaking padding scales, border widths, shadows, or corner-radii is done by editing `style.toml`, keeping the Rust controls clean.
* **No Compile Bottlenecks**: Modifying styles does not require compiling macro crates or running cargo builds. Mappings can be reloaded dynamically at runtime in Theme Studio.

---

## 2. Proposed Mappings Schema (`style.toml`)

The schema maps a 5-part selector address directly to output visual and metric properties.

### A. Selector Address Structure
```toml
# Selector address components:
# [component_type.style_variant.interaction_layer.theme_mode.selection_state]
```

### B. Mappings Example (`style.toml`)
```toml
# =====================================================================
# Button Styling Rules
# =====================================================================

# Primary Default State
[button.primary.default.any.any]
background = "primary"
foreground = "primary-foreground"
border = "primary"
height = "metrics.control.md"
padding_horizontal = "16.0"
font_size = "14.0"
icon_size = "16.0"
corner_radius = "radius"
shadow = "none"

# Outline Default State (Light Mode)
[button.outline.default.light.false]
background = "transparent"
foreground = "foreground"
border = "border"
corner_radius = "radius"
shadow = "shadow-sm"

# Outline Hovered State (Dark Mode)
[button.outline.hover.dark.false]
background = "input/50"
foreground = "accent-foreground"
border = "input"
corner_radius = "radius"
shadow = "shadow-sm"

# Selected Toggles (Default State)
[button.outline.default.any.true]
background = "primary"
foreground = "primary-foreground"
border = "primary"
corner_radius = "radius"
shadow = "shadow-sm"
```

---

## 3. Implementation Checklist & Considerations

To implement this layout model, we must resolve several key architectural backlog items:

### 1. Non-Color Parsing (`ResolveValue<T>`)
The `LookResolver` must parse more than just color variables. We need trait-based parsing implementations to resolve different cell types from the CSS catalog:
* **`ResolveValue<f32>`**: Resolves dimensions like heights and border radii (converting `--radius: 0.25rem` into raw pixel values, e.g. `4.0`).
* **`ResolveValue<Shadow>`**: Parses shadow variables (e.g. `--shadow-sm: 2px 2px 4px 0px hsl(...)`) and maps them to GPUI shadows.
* **`ResolveValue<SharedString>`**: Resolves font families (e.g. `--font-sans: Outfit`).

### 2. Standardizing the Rust Control Resolvers
The control resolvers (like `button_palette` in `button.rs`) must be thinned down. They should be simple data-piping functions that:
1. Gather state inputs (`layer`, `mode`, `selected`).
2. Query the parsed `style.toml` catalog for the component.
3. Apply standard adorners (focus rings) and build the final palette struct.

### 3. Dynamic Theme Reloading
For the Theme Studio to be truly interactive:
* The loader must monitor changes to `theme.toml` and token CSS files (e.g. using `notify` or simple reload hooks).
* Modifying the styling file instantly repaints the GPUI preview matrix in real-time, matching standard web development feedback loops.

---

## 4. The Complexity of Metrics (The "Box Model" Seams)

Unlike color replacement (which is a static 1:1 token lookup), metrics determine the **physical layout bounds and constraints** (the Box Model). When moving metrics to `style.toml`, the following structural intersections must be addressed:

### A. The CSS Math Engine (`calc()`)
CSS variables frequently use calculated layouts:
- `--radius-md: calc(var(--radius) - 2px);`
If the style parser only does static token matches, it will fail to compile. The `ResolveValue<f32>` resolver must implement a simple expression evaluator capable of:
1. Stripping and resolving nested variables `var(...)`.
2. Handling basic unit arithmetic (rems to pixels, addition, subtraction).

### B. Positional Aspect-Ratio Constraints (The Icon Button Sizing Seam)
Standard buttons have horizontal padding (e.g. `16px`). However, circular or square icon buttons must have `width = height` and `padding_horizontal = 0`.
* **The Seam**: The stylesheet parser must support conditional metrics based on component layout constraints (e.g. `role = "Icon"` overriding `padding_x = 0.0` or setting `aspect_ratio = "square"`).

### C. Pixel Snapping & High-DPI Scaling
GPUI aligns sizes with physical device pixels relative to `window.scale_factor()`.
* **The Seam**: Mapped float parameters (like radii and border widths) must be evaluated as logical pixels and cleanly mapped into the GPUI canvas rendering context without creating layout shifts.

### D. Typographical Center Alignment
If font size and line height are configured independently, the label will clip or center off-axis inside the button track.
* **The Seam**: Line height metrics must align with the button box's internal height. The typography system must calculate a baseline offset dynamically to ensure text remains perfectly centered regardless of the custom font family.

---

## 5. Bootstrapping the System: "We Only Have a Button"

Since the button control is our primary visual proof-of-concept, we can bootstrap the entire stylesheet-driven configuration focusing exclusively on button styles and dimensions. Below is the specification for what a button-only `style.toml` looks like and how we parser-resolve it in Rust.

### A. The Button Stylesheet (`style.toml`)

To support flexible overrides (such as `"any"` fallback wildcards), the stylesheet defines metrics structures and an ordered list of color mapping rules. This matches CSS rule selector matching:

```toml
# style.toml - Unified Button Mappings

# ---------------------------------------------------------------------
# 1. Layout Metrics (Dimension configurations by ControlSize)
# ---------------------------------------------------------------------
[button.metrics.sm]
height = "metrics.control.sm"
padding_horizontal = 12.0
font_size = 12.0
icon_size = 14.0
corner_radius = "radius"
shadow = "shadow-xs"

[button.metrics.md]
height = "metrics.control.md"
padding_horizontal = 16.0
font_size = 14.0
icon_size = 16.0
corner_radius = "radius"
shadow = "shadow-sm"

[button.metrics.lg]
height = "metrics.control.lg"
padding_horizontal = 20.0
font_size = 16.0
icon_size = 18.0
corner_radius = "radius"
shadow = "shadow-md"

# ---------------------------------------------------------------------
# 2. Color Rules (Evaluated in order; first match wins)
# ---------------------------------------------------------------------
# Selectors filter by: style, layer, mode, and selected.
# Any omitted filter or filter set to "any" matches any value.

[[button.color_rules]]
style = "primary"
layer = "default"
background = "primary"
foreground = "primary-foreground"

[[button.color_rules]]
style = "primary"
layer = "hover"
background = "primary-hover"
foreground = "primary-foreground"

[[button.color_rules]]
style = "primary"
layer = "pressed"
background = "primary-pressed"
foreground = "primary-foreground"

[[button.color_rules]]
style = "primary"
layer = "disabled"
background = "muted"
foreground = "muted-foreground"

[[button.color_rules]]
style = "secondary"
layer = "default"
background = "secondary"
foreground = "secondary-foreground"

[[button.color_rules]]
style = "secondary"
layer = "hover"
background = "secondary-hover"
foreground = "secondary-foreground"

# Outline Toggle Buttons (Selected = true)
[[button.color_rules]]
style = "outline"
selected = true
background = "primary"
foreground = "primary-foreground"
border = "primary"

# Outline Buttons (Selected = false / Default)
[[button.color_rules]]
style = "outline"
layer = "default"
selected = false
background = "transparent"
foreground = "foreground"
border = "border"

# Light Mode Hover Outline
[[button.color_rules]]
style = "outline"
layer = "hover"
mode = "light"
selected = false
background = "accent"
foreground = "accent-foreground"
border = "border"

# Dark Mode Hover Outline (with 50% opacity suffix)
[[button.color_rules]]
style = "outline"
layer = "hover"
mode = "dark"
selected = false
background = "input/50"
foreground = "accent-foreground"
border = "input"

# Ghost Toggles (Selected = true)
[[button.color_rules]]
style = "ghost"
selected = true
background = "primary"
foreground = "primary-foreground"

# Ghost Buttons (Selected = false / Default)
[[button.color_rules]]
style = "ghost"
layer = "default"
selected = false
background = "transparent"
foreground = "foreground"

[[button.color_rules]]
style = "ghost"
layer = "hover"
mode = "light"
selected = false
background = "accent"
foreground = "accent-foreground"

[[button.color_rules]]
style = "ghost"
layer = "hover"
mode = "dark"
selected = false
background = "accent/50"
foreground = "accent-foreground"

[[button.color_rules]]
style = "ghost"
layer = "disabled"
background = "transparent"
foreground = "muted-foreground"

# Universal Fallback Rule
[[button.color_rules]]
background = "transparent"
foreground = "foreground"
```

### B. Rust Parser Data Structures

We map the TOML stylesheet structures directly to deserializable Rust models:

```rust
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct StylesheetConfig {
    pub button: ButtonStylesheet,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ButtonStylesheet {
    pub metrics: HashMap<String, ButtonMetricsRule>,
    pub color_rules: Vec<ButtonColorRule>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ButtonMetricsRule {
    pub height: String,
    pub padding_horizontal: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub corner_radius: String,
    pub shadow: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ButtonColorRule {
    pub style: Option<String>,
    pub layer: Option<String>,
    pub mode: Option<String>,
    pub selected: Option<bool>,

    pub background: String,
    pub foreground: String,
    pub border: Option<String>,
}

impl ButtonColorRule {
    /// Evaluates if the current lookup parameters match this rule.
    pub fn matches(&self, style: &str, layer: &str, mode: &str, selected: bool) -> bool {
        self.style.as_deref().map_or(true, |s| s == "any" || s == style)
            && self.layer.as_deref().map_or(true, |l| l == "any" || l == layer)
            && self.mode.as_deref().map_or(true, |m| m == "any" || m == mode)
            && self.selected.map_or(true, |s| s == selected)
    }
}
```

### C. Integrating and Resolving at Runtime (`button.rs`)

The loaded stylesheet config is accessed through the `AppearanceContext` and evaluated. The custom procedural macro `declare_look_table!` acts as an intermediate compiler cache or code-gen reference, but the ultimate stylesheet loader queries the TOML rules sequentially:

```rust
pub(crate) fn button_palette(
    ctx: &AppearanceContext,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ControlSize,
) -> ButtonFamilyPalette {
    let state = ctx.state;
    let layer = state.layer().to_string(); // "default", "hover", "pressed", "disabled"
    let theme_mode = ctx.theme_mode.to_string(); // "light", "dark"
    let selected = matches!(role, ButtonFamilyRole::Toggle { selected: true });
    
    let style_str = match style {
        ShadcnButtonStyle::Primary => "primary",
        ShadcnButtonStyle::Secondary => "secondary",
        ShadcnButtonStyle::Outline => "outline",
        ShadcnButtonStyle::Ghost => "ghost",
    };

    // 1. Resolve colors from the stylesheet config rules
    let colors = ctx.look().stylesheet()
        .find_color_rule(style_str, &layer, &theme_mode, selected)
        .map(|rule| ButtonColorPalette {
            background: ctx.resolver().resolve_decl(&rule.background).unwrap_or_default(),
            foreground: ctx.resolver().resolve_decl(&rule.foreground).unwrap_or_default(),
            border: rule.border.as_ref().map(|b| ctx.resolver().resolve_decl(b).unwrap_or_default()),
        })
        .unwrap_or_else(|| ButtonColorPalette::fallback());

    // 2. Resolve metrics configurations by size key ("sm", "md", "lg")
    let size_key = match size {
        ControlSize::Sm => "sm",
        ControlSize::Md => "md",
        ControlSize::Lg => "lg",
    };
    let metrics = ctx.look().stylesheet()
        .get_metrics("button", size_key)
        .unwrap_or_else(|| ButtonMetrics::fallback());

    // 3. Set conditional outer focus ring
    let adorner = state.focused.then(|| {
        AdornerSpec::FocusRing(FocusRingAdornerSpec {
            color: ctx.palette().focus_ring,
            placement: match style {
                ShadcnButtonStyle::Ghost => AdornerPlacement::Inset,
                _ => AdornerPlacement::Oversize,
            },
            distance: ctx.metrics().border_width.default + ctx.metrics().focus.width,
            width: ctx.metrics().focus.width,
        })
    });

    // 4. Assemble the final style palette for the button rendering template
    ButtonFamilyPalette {
        background: colors.background.hsla(),
        foreground: colors.foreground.hsla(),
        border: colors.border.map(|c| c.hsla()).unwrap_or(colors.background.hsla()),
        adorner,
        typography: ctx.typography().text.label.with_size(metrics.font_size),
        font_family: ctx.typography().font.sans.family.clone().into(),
    }
}
```
