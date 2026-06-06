# Design Spec: Dynamic Resolution Mapping & Interaction Fallbacks

This document outlines the architectural limitations of the current interaction color resolution system in `gpui-luma` and details the **prescribed** (recommended) solution, implementation details, and **proscribed** (discouraged) design patterns.

---

## 1. Problem Description

The current styling system resolves interaction colors (hovered, pressed, focused, disabled) during initial catalog parsing and stores them in static structs (`ShadcnPalette`, `ShadcnActionRole`). This approach has three key flaws:

### A. Context-Deaf State Math
In `crates/look-shadcn/src/palette.rs`, interaction states are generated via simple hardcoded multipliers:
```rust
hover_background: darken(background, 0.06),
pressed_background: darken(background, 0.12),
```
* **Contrast Inversion**: While darkening a primary background by `0.06` creates a pleasant hover cue on light backgrounds, doing the same on a dark background (e.g. background of HSL 192, 100%, 10%) reduces contrast, rendering the component nearly invisible or appearing disabled. Dark mode hover states typically require *lightening* the base color.
* **Color Space Distortion**: Standard HSL-based darkening changes perceived saturation unevenly depending on hue. 

### B. Rigid Statically Typed Mappings
Components query style tokens from hardcoded struct fields (e.g., `palette.primary.hover_background`).
* **High Maintenance**: Adding a new interactive state (like `ActiveSelected` or `Dragging`) requires changing the underlying data structures, parsers, and component mappers.
* **CSS Variable Asymmetry**: It forces third-party CSS imports to exactly match the Rust struct's schema, rather than allowing themes to declare only the tokens they care about and let the engine infer the rest.

### C. Fragile CSS Variable Imports
If a theme CSS file (like a customized `retro-arcade.css`) does not define custom properties for hover states, the controls either:
1. Fall back to unstyled/transparent states (visual degradation).
2. Panics or fails to parse if keys are strictly expected.

---

## 2. Prescribed Solution (The Recommended Design)

To achieve theme-resilience and component simplicity, we transition to a **lazy, query-based resolver pipeline** located entirely inside `ShadcnLook`.

### A. The Query Interface
Components do not request state fields. Instead, they make a query passing the base token and the target interaction layer:
```rust
let bg = look.resolve_color_state(ShadcnToken::Primary, InteractionLayer::Hovered);
```

### B. Implementation Blueprint

#### Mapping Tokens to CSS Keys
```rust
impl ShadcnToken {
    pub fn css_name(&self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Muted => "muted",
            Self::Accent => "accent",
            Self::Destructive => "destructive",
            Self::Background => "background",
            Self::Foreground => "foreground",
            Self::Card => "card",
            Self::CardForeground => "card-foreground",
            Self::Popover => "popover",
            Self::PopoverForeground => "popover-foreground",
            Self::PrimaryForeground => "primary-foreground",
            Self::SecondaryForeground => "secondary-foreground",
            Self::MutedForeground => "muted-foreground",
            Self::AccentForeground => "accent-foreground",
            Self::DestructiveForeground => "destructive-foreground",
            Self::Border => "border",
            Self::Input => "input",
            Self::Ring => "ring",
        }
    }
}
```

#### Resolver Method implementation
```rust
use gpui_luma::theme::InteractionLayer;

impl ShadcnLook {
    /// Dynamically resolves a token color for a specific interaction state,
    /// falling back to mathematical color shifts if explicit variables are absent.
    pub fn resolve_color_state(&self, token: ShadcnToken, layer: InteractionLayer) -> Hsla {
        let catalog = &self.mode_tokens().catalog;
        
        // 1. Map the interaction state to a standard CSS variable suffix
        let state_suffix = match layer {
            InteractionLayer::Default => "",
            InteractionLayer::Hovered => "-hover",
            InteractionLayer::Pressed => "-pressed",
            InteractionLayer::Disabled => "-disabled",
        };

        // 2. Look for an explicit override in the catalog (e.g. `--primary-hover`)
        let state_key = format!("{}{}", token.css_name(), state_suffix);
        if let Ok(color) = catalog.color(&state_key) {
            return color;
        }

        // 3. Fallback: Resolve base token color (e.g. `--primary`)
        let base_color = self.color(token);

        // 4. Algorithmic correction based on active Theme Mode (Light / Dark)
        match layer {
            InteractionLayer::Default => base_color,
            InteractionLayer::Disabled => {
                // Fall back to disabled tokens
                self.mode_tokens().palette.disabled_background
            }
            InteractionLayer::Hovered => {
                // Apply lightness shifts in Oklch space depending on Mode
                match self.mode() {
                    ThemeMode::Light => adjust_lightness(base_color, -0.06), // darken light background
                    ThemeMode::Dark => adjust_lightness(base_color, 0.08),   // lighten dark background
                }
            }
            InteractionLayer::Pressed => {
                // Double the shift factor for pressed states
                match self.mode() {
                    ThemeMode::Light => adjust_lightness(base_color, -0.12),
                    ThemeMode::Dark => adjust_lightness(base_color, 0.16),
                }
            }
        }
    }
}
```

#### Color-Space Aware Adjustments (Oklch)
To avoid shifting hues or distorting saturation during state changes, color shifts are calculated using the **Oklch** color space instead of raw HSL:
$$\text{Color}_{\text{hover}} = \text{Oklch}(L \pm \Delta L, C, H)$$

```rust
/// Adjusts the lightness of an HSLA color safely using OKLCH conversion
fn adjust_lightness(color: Hsla, delta: f32) -> Hsla {
    // 1. Convert HSLA to OKLCH coordinates
    let mut oklch = oklch_from_hsla(color);
    
    // 2. Adjust the L (Lightness) channel, keeping it within [0.0, 1.0]
    oklch.l = (oklch.l + delta).clamp(0.0, 1.0);
    
    // 3. Convert back to HSLA
    hsla_from_oklch(oklch)
}
```

---

## 3. Proscribed Patterns (What to Avoid)

To maintain a clean division of labor and high rendering performance, the following anti-patterns are strictly forbidden:

* **[PROSCRIBED] Component-Level State Math**:
  Components (like buttons, checkboxes, inputs) must **never** perform color math (like `.darken()` or `.lighten()`) inside their own `render` methods. This creates inconsistent hover behaviors across different components.
* **[PROSCRIBED] String-Parsing in the Paint Loop**:
  The resolver must **never** perform string matching, regex parsing, or heavy map lookups during layout/paint. All CSS properties must be parsed into high-performance token arrays at load time, allowing resolution queries to run in $O(1)$ time.
* **[PROSCRIBED] Static Monolithic Palettes**:
  Do not create predefined structs with fields for every possible state permutation. This binds the SDK components to a rigid theme layout and breaks compatibility with minimal style templates.
