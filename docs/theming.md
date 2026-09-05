# GPUI-Luma Theming & Style System

This guide outlines the 3-Tier Luma design system, global theme tokens, stylesheet mappings (`style.toml`), look extensions, typography policies, and instructions for styling new controls.

---

## 1. The 3-Tier Design System

GPUI-Luma completely decouples control behavior from the styling system through a three-layer pipeline:

```text
┌──────────────────────────────────────────────┐
│  Tier 1: Design Tokens (CSS Catalog)         │ <-- Theme skin CSS supplied by the app
│  --primary: hsl(330 64% 52%); --radius: 6px   │     (`ShadcnLook::from_css_str`). Demo packs
│                                              │     live in `apps/common`.
└──────────────────────┬───────────────────────┘
                       │ resolves variables
┌──────────────────────▼───────────────────────┐
│  Tier 2: Semantic Mappings (style.toml)      │ <-- SOURCE OF TRUTH
│  button.primary.default.background = "primary"│     (Maps design variables to controls)
└──────────────────────┬───────────────────────┘
                       │ evaluates
┌──────────────────────▼───────────────────────┐
│  Tier 3: Control Templates (Rust SDK)        │ <-- Structural Layout & Painting
│  Paints component using resolved variables   │     (Zero hardcoded colors, zero macros)
└──────────────────────────────────────────────┘
```

---

## 2. Tier 1: Design Tokens & Metric Scales

Design tokens represent global style primitives (colors, metrics, typography, shadows):
*   **Palette Colors**: Mapped using `Hsla` types and state color models (hovered, pressed, focused, disabled).
*   **Metrics & Spacing**: Resolved from standard metric configurations.
*   **Metric Scales**: Controls fetch margins, height steps, border widths, and radii from shared scales cached in the `LumaLayoutCache`. These include `StandardBoxScale` (for fields and triggers) and `ListRowScale` (for lists and rows) to snap visuals to pixel grids cleanly depending on the active `window.scale_factor()`.

---

## 3. Tier 2: Semantic Mappings (`style.toml`)

`style.toml` maps component states directly to Tier-1 tokens.

### Selector Address Structure
Rules are evaluated sequentially using a five-part selector address:
```toml
# Selector address components:
# [component_type.style_variant.interaction_layer.theme_mode.selection_state]
```

### Color Mapping Rules Example
```toml
# style.toml - Mappings Example
[[button.color_rules]]
style = "primary"
layer = "default"
background = "primary"
foreground = "primary-foreground"

[[button.color_rules]]
style = "outline"
layer = "hover"
mode = "dark"
selected = false
background = "input/50"     # Supports opacity suffix (evaluated at runtime)
foreground = "accent-foreground"
border = "input"
```

### Metrics Configurations
```toml
[button.metrics.md]
height = "metrics.control.md"
padding_horizontal = 16.0
font_size = 14.0
icon_size = 16.0
corner_radius = "radius"
shadow = "shadow-sm"
```

---

## 4. Typography Guidelines

Luma provides theme-aware text scale resolvers to prevent manual `text_size + line_height + font_weight` repetitions:

### App-level Guidelines (`apps/*`)
Apps may use GPUI's built-in text helpers (`text_xs()`, `text_sm()`, `text_base()`, `text_lg()`, `text_xl()`, `text_2xl()`) when simple fixed-scale sizing is sufficient and theme-driven line-heights are not required.

### SDK / Look-level Guidelines (`crates/sdk`, `crates/look-shadcn`)
SDK and look crates must **never** use GPUI's text helpers directly. Instead, they must query typography from `ShadcnLook` or apply resolved `LumaTextStyle` values using:
*   **Semantic Role Helpers**: `text_h1()`, `text_h2()`, `text_h3()`, `text_h4()`, `text_p()`.
*   **Themed Style Resolver**: `.typography_style(look.typography_scale(ShadcnTextSize::Sm))`.

---

## 5. Look-Specific Extensions

Design-language specifics (like Shadcn variants, styled card containers, and badges) reside entirely inside the look crate (`crates/look-shadcn`), leaving the core SDK look-agnostic. Shared provenance shapes live in `crates/look-core`. A second look stub lives in `crates/look-radix` (`RadixLook` + `RadixLookControlExt`) and implements the same SDK `*Theme` traits without Shadcn token names.

These extensions are implemented via trait extension on the active theme:
```rust
pub trait ShadcnLookControlExt {
    fn button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn card(&self, id: impl Into<SharedString>) -> ShadcnCardBuilder;
    fn badge(&self, label: impl Into<SharedString>) -> Badge;
    // ... maps specialized controls directly to Shadcn templates
}
```
 Downstream applications call these methods (e.g. `look.primary_button("save")`, `look.card("panel")`) to spawn controls or look-layer styled containers pre-bound to Shadcn tokens.

To author another look without editing the SDK: implement the needed `*Theme` traits, expose a look-local factory ext, and keep source interpretation (CSS, 12-step scales, …) inside the look crate. See `luma_look_core` crate docs and [`docs/look-boundary-inventory.md`](look-boundary-inventory.md).

---

## 6. Playbook: Adding Styles for New Controls

Follow this pipeline when adding a new interactive control or visual sub-component:

1.  **Define Layout Scales in SDK**: Declare your control structural bounds and metrics scales. Avoid raw constants.
2.  **Declare Mapping Configs**: Open `crates/look-shadcn/src/stylesheet/config.rs` and add the matching configuration structs and matcher implementations:
    ```rust
    #[derive(Debug, Deserialize, Clone, Default)]
    pub struct NewControlStylesheet {
        #[serde(default)]
        pub metrics: HashMap<String, NewControlMetricsRule>,
        #[serde(default)]
        pub color_rules: Vec<NewControlColorRule>,
    }
    ```
3.  **Register lookup in `stylesheet/mod.rs`**: Expose helper functions (like `find_new_control_color_rule`).
4.  **Write the Resolver**: Implement the look lookup bridge (e.g., `resolve_new_control_colors`) inside `crates/look-shadcn/src/controls/new_control.rs`.
5.  **Author Mapping Rules**: Open `crates/look-shadcn/assets/style.toml` and write the mapping constraints.
