# GPUI-Luma Templates v2: Parameterization & Introspection

This document outlines the architecture for "cleanly parameterizing" control templates in the GPUI-Luma SDK, allowing for structural customization without rewriting complex template implementations.

## The Problem

Currently, many default control templates (e.g., `NavigationSidebar`) contain dozens of hardcoded `const` values for structural spacing, sizes, and layout choices. While colors and primitive metrics belong to the `Theme`, these layout-specific constants are structural configuration for the *Template* itself.

Users who want to tweak these layout constraints (like indent multipliers or container gaps) are currently forced to copy-paste the entire trait implementation.

## The Solution: Template Parameter Structs

To solve this without breaking the existing `Arc<dyn Template>` dynamic dispatch architecture, we will extract hardcoded constants into strongly-typed parameter structs owned by the default templates, and expose introspection metadata for dynamic tooling.

### 1. Template Parameter Structs

Every complex template will expose a `Params` struct containing its structural properties:

```rust
#[derive(Clone, Debug)]
pub struct NavigationSidebarTemplateParams {
    pub container_gap: f32,
    pub child_depth_indent_multiplier: f32,
    // ...
}

impl Default for NavigationSidebarTemplateParams {
    // ...
}
```

### 2. Builder Injection

The default template (e.g., `ThemedNavigationSidebarTemplate`) will own these parameters alongside the theme, allowing users to override the defaults cleanly during construction:

```rust
let mut params = NavigationSidebarTemplateParams::default();
params.child_depth_indent_multiplier = 1.5;

let custom_template = ThemedNavigationSidebarTemplate::new(theme)
    .with_params(params);

NavigationSidebar::new("sidebar")
    .template(Arc::new(custom_template))
```

### 3. Introspection Metadata (Reflection)

To support advanced tooling like in-app "Property Editors", templates will expose reflection metadata mirroring the `ThemeUsage` pattern.

We will introduce a `TemplateUsage` registry:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplatePropertyType {
    Pixels,
    Multiplier,
    FontSize,
    FontWeight,
    Icon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemplatePropertyMetadata {
    pub name: &'static str,
    pub description: &'static str,
    pub property_type: TemplatePropertyType,
    pub field: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemplateUsage {
    pub component: &'static str,
    pub properties: &'static [TemplatePropertyMetadata],
}
```

This metadata allows applications (like the SDK Gallery) to iterate over `all_template_usages()` and dynamically generate inputs (sliders for `Pixels`, dropdowns for `FontWeight`) for every tunable constant in the SDK.

## Scope

This refactoring applies primarily to complex controls (`NavigationSidebar`, `NavView`, `PopupMenu`, `ContextMenu`) where layout relies on many structural constants, but can also be utilized for simpler controls (`Button`, `Checkbox`) to parameterize structural affordances like focus ring gaps.
