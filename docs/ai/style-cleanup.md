# Stylesheet Decoupling: Legacy Palette Code Cleanup Plan

Now that the dynamic stylesheet configuration (`style.toml`) is fully implemented and integrated across all 18 controls, the dual-path resolution scaffolding (separating legacy `Palette` fallbacks from stylesheet `Catalog` lookups) is redundant.

This document outlines the scope, before-and-after structures, and overall code-reduction impact of cleaning up this legacy code.

---

## 1. Context: Why the Dual Path Existed
During the migration phase, controls maintained a branch to guard against empty catalogs:
```rust
pub fn control_appearance(...) -> ControlPalette {
    if mode.catalog.tokens.is_empty() {
        control_appearance_from_palette(...) // Legacy Hardcoded Path
    } else {
        control_appearance_from_catalog(...) // Dynamic TOML/CSS Path
    }
}
```
Because the stylesheet (`style.toml`) is now fully embedded, parsed, and validated at startup for all styles and modes, `mode.catalog` is the guaranteed source of truth. The fallback code pathways can be retired completely.

---

## 2. Scope of Cleanup

The cleanup target involves:
1. **Removing Legacy Helper Functions**: Delete all private and public `*_from_palette` helpers.
2. **Collapsing Entry Points**: Merge `*_from_catalog` logic directly into the main `*_appearance` (or `*_palette`) entry points.
3. **Visibility Restructure**: Restrict internal matching and resolver functions (like `resolve_checkbox_colors`) to `pub(crate)` or private helper functions.

---

## 3. Example Comparison (Checkbox)

### Before (161 lines of code)
Double parsing pathways, redundant structures, and duplicate metric builders:
```rust
// crates/look-shadcn/src/controls/checkbox.rs (excerpt)

pub fn checkbox_appearance(...) -> CheckboxPalette {
    if mode.catalog.tokens.is_empty() {
        checkbox_appearance_from_palette(...)
    } else {
        checkbox_appearance_from_catalog(...)
    }
}

pub fn checkbox_appearance_from_palette(...) -> CheckboxPalette {
    // 40 lines of match state tables ...
}

pub fn checkbox_appearance_from_catalog(...) -> CheckboxPalette {
    // 30 lines of resolver checks ...
}
```

### After (48 lines of code)
A single, clean, linear mapping function:
```rust
// crates/look-shadcn/src/controls/checkbox.rs (consolidated)

pub fn checkbox_appearance(
    mode: &ShadcnModeTokens,
    style: ShadcnButtonStyle,
    checked: bool,
    state: InteractionState,
) -> CheckboxPalette {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, state);
    let catalog = ctx.catalog();
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "checkbox");
    
    let colors = resolve_checkbox_colors(&resolver, style, checked, state.layer())
        .unwrap_or_else(|_| CheckboxColorTable::fallback());

    let indicator_border = if checked && !state.disabled {
        colors.indicator_background.hsla()
    } else {
        resolver.resolve_decl("border").map(|c| c.hsla()).unwrap_or(colors.indicator_background.hsla())
    };

    CheckboxPalette {
        control_background: None,
        control_border: None,
        indicator_background: colors.indicator_background.hsla(),
        indicator_border,
        checkmark_color: colors.checkmark_color.hsla(),
        label_color: colors.label_color.hsla(),
        adorner: focus_adorner(catalog, ctx.metrics(), state.focused).unwrap_or(None),
        label_typography: ctx.typography().text.label,
        label_font_family: ctx.typography().font.sans.family.clone().into(),
    }
}
```

---

## 4. Impact Summary

* **Code Size**: Deletes `~1,000+ lines` of duplicate code across the 18 controls in `crates/look-shadcn/src/controls/`.
* **Symmetry**: Standardizes all controls to have exactly one entrance function and one stylesheet resolve method.
* **Inspectability**: Unifies the inspector look probe UI around a single theme loading context.
