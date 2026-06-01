# Design Proposal: Unifying Appearance Function Signatures (`AppearanceContext`)

This document describes a code organization refactoring to simplify and unify the verbose parameter signatures used throughout the SDK's Radix/tweakcn theme appearance modules (`crates/sdk/src/theme/radix/*`).

---

## 1. The Problem: Verbose Parameter Signatures

Currently, the style/appearance resolution functions for each control require passing numerous styling catalog references, metric tokens, typographic maps, color modes, and interaction states as individual parameters. 

For example, the **Switch** and **Checkbox** catalog-based appearance resolvers have highly complex, verbose signatures:

```rust
// crates/sdk/src/theme/radix/switch.rs
pub(crate) fn switch_appearance_from_catalog(
    catalog: &CssTokenMap,
    metrics: &crate::theme::MetricTokens,
    typography: &crate::theme::LumaTypography,
    theme_mode: ThemeMode,
    style: RadixButtonStyle,
    on: bool,
    state: InteractionState,
) -> anyhow::Result<SwitchPalette>
```

```rust
// crates/sdk/src/theme/radix/checkbox.rs
pub(crate) fn checkbox_appearance_from_catalog(
    catalog: &CssTokenMap,
    metrics: &crate::theme::MetricTokens,
    typography: &crate::theme::LumaTypography,
    style: RadixButtonStyle,
    checked: bool,
    state: InteractionState,
) -> anyhow::Result<CheckboxPalette>
```

Passing 6 to 7 arguments down to helper functions creates heavy boilerplate, increases cognitive load when writing or modifying components, and makes signature updates highly disruptive if new global styling variables are introduced.

---

## 2. Proposed Solution: `AppearanceContext`

To streamline the interface and organize control themes cleanly, we propose grouping the standard, shared theme dependencies and active state parameters into a single unified context struct: `AppearanceContext`.

### Context Definition

This struct bundles the active mode tokens, the current color mode (light/dark), and the current pointer/focus interaction state. It provides helper methods to access underlying styling tokens directly:

```rust
// crates/sdk/src/theme/radix/context.rs (or theme/radix/mod.rs)

use crate::theme::{LumaTypography, MetricTokens, ThemeMode};
use crate::theme::InteractionState;
use super::catalog::CssTokenMap;
use super::mode::RadixModeTokens;
use super::palette::RadixPalette;

/// Bundles styling dependencies and interactive state for control appearance resolution.
pub struct AppearanceContext<'a> {
    /// Active mode-specific tokens (palette, metrics, typography, catalog)
    pub tokens: &'a RadixModeTokens,
    /// Active color mode (Light/Dark)
    pub theme_mode: ThemeMode,
    /// Active control pointer/focus interaction state
    pub state: InteractionState,
}

impl<'a> AppearanceContext<'a> {
    pub fn new(tokens: &'a RadixModeTokens, theme_mode: ThemeMode, state: InteractionState) -> Self {
        Self { tokens, theme_mode, state }
    }

    /// Access the resolved CSS style variable map.
    pub fn catalog(&self) -> &CssTokenMap {
        &self.tokens.catalog
    }

    /// Access the raw Radix action/surface color palette.
    pub fn palette(&self) -> &RadixPalette {
        &self.tokens.palette
    }

    /// Access the global layout metric sizes.
    pub fn metrics(&self) -> &MetricTokens {
        &self.tokens.metrics
    }

    /// Access typography specs.
    pub fn typography(&self) -> &LumaTypography {
        &self.tokens.typography
    }
}
```

---

## 3. Comparison of Signature Refactoring

By passing a reference to `AppearanceContext`, the signatures of every component's appearance resolvers collapse down to a clean, consistent model:

### Switch Appearance
* **Before (7 parameters):**
  ```rust
  pub(crate) fn switch_appearance_from_catalog(
      catalog: &CssTokenMap,
      metrics: &MetricTokens,
      typography: &LumaTypography,
      theme_mode: ThemeMode,
      style: RadixButtonStyle,
      on: bool,
      state: InteractionState,
  ) -> anyhow::Result<SwitchPalette>
  ```
* **After (3 parameters):**
  ```rust
  pub(crate) fn switch_appearance_from_catalog(
      ctx: &AppearanceContext,
      style: RadixButtonStyle,
      on: bool,
  ) -> anyhow::Result<SwitchPalette>
  ```

### Checkbox Appearance
* **Before (6 parameters):**
  ```rust
  pub(crate) fn checkbox_appearance_from_catalog(
      catalog: &CssTokenMap,
      metrics: &MetricTokens,
      typography: &LumaTypography,
      style: RadixButtonStyle,
      checked: bool,
      state: InteractionState,
  ) -> anyhow::Result<CheckboxPalette>
  ```
* **After (3 parameters):**
  ```rust
  pub(crate) fn checkbox_appearance_from_catalog(
      ctx: &AppearanceContext,
      style: RadixButtonStyle,
      checked: bool,
  ) -> anyhow::Result<CheckboxPalette>
  ```

---

## 4. Simplified Internal Implementation

Using the context inside the resolvers removes parameter repetition and groups operations into a single namespace. 

Here is how the catalog-based switch resolver simplifies:

```rust
// crates/sdk/src/theme/radix/switch.rs

pub(crate) fn switch_appearance_from_catalog(
    ctx: &AppearanceContext,
    style: RadixButtonStyle,
    on: bool,
) -> anyhow::Result<SwitchPalette> {
    let thumb_shadow = {
        let native = LumaTheme::native();
        native.mode(ctx.theme_mode).elevation.thumb.to_box_shadows()
    };

    let track_background = if ctx.state.disabled {
        resolve_color(ctx.catalog(), "muted")?
    } else if on {
        resolve_action_layer(ctx.catalog(), style, InteractionLayer::Default)?
    } else {
        resolve_color(ctx.catalog(), "input")?
    };

    let track_border = if on && !ctx.state.disabled {
        track_background
    } else {
        resolve_color(ctx.catalog(), "border")?
    };

    let (thumb_background, thumb_border) = switch_thumb_colors(ctx.catalog(), style, on, ctx.state.disabled)?;

    Ok(SwitchPalette {
        track_background,
        track_border,
        thumb_background,
        thumb_border,
        thumb_shadow,
        label_color: resolve_label_color(ctx.catalog(), ctx.state.disabled)?,
        adorner: focus_adorner(ctx.catalog(), ctx.metrics(), ctx.state.focused)?,
        label_typography: ctx.typography().text.label,
        label_font_family: ctx.typography().font.sans.family.clone().into(),
    })
}
```

---

## 5. Benefits of this Refactoring

1. **A Single Consistent Contract**: Every module under `crates/sdk/src/theme/radix/*` adopts the exact same structural approach. The function parameters only differ by the component's unique configuration keys (e.g., `checked: bool`, `variant: TextFieldVariant`).
2. **Boilerplate Reduction**: Drastically reduces parameter passing, making signature changes and code reviews much easier to parse.
3. **Decoupled Extensions**: If the GPUI-Luma SDK introduces new global styling configurations (e.g., animation settings, density settings) in the future, we only add them to `AppearanceContext`. Individual resolver signatures remain fully decoupled and untouched.
