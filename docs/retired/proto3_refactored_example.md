# Proto3 Refactored Approach: Typed Metadata for ProtoButton

This document demonstrates the refactored direction described in `docs/proto3.md`, with a clear split between what is already implemented in the prototype and what remains future work. The prototype metadata system now uses typed field identifiers instead of stringly-typed paths.

## The Problem

The original implementation used string paths like:
```rust
param_fields: &["ProtoButtonTemplateParams.variant"],
param_fields: &[
    "ProtoButtonTemplateParams.background.base",
    "ProtoButtonTemplateParams.background.hovered",
    // ... etc
]
```

This approach was prone to drift and typos, making maintenance difficult.

## The Solution

The refactored approach now has two parts:

1. **Implemented now**: typed field identifiers (`ProtoButtonTemplateParamField`) used by metadata.
2. **Planned next**: derive-generated metadata to remove remaining manual registration work.

### 1. New Typed Enum Definition

```rust
pub enum ProtoButtonTemplateParamField {
    Variant,
    Size,
    DisabledOpacity,
    PointerCursorWhenEnabled,
    BackgroundBase,
    BackgroundHovered,
    BackgroundPressed,
    BackgroundFocused,
    BackgroundDisabled,
    ForegroundBase,
    ForegroundHovered,
    ForegroundPressed,
    ForegroundFocused,
    ForegroundDisabled,
    BorderBase,
    BorderHovered,
    BorderPressed,
    BorderFocused,
    BorderDisabled,
    FocusRing,
    Radius,
    PaddingX,
    PaddingY,
    Gap,
    Height,
    TypographySize,
    TypographyLineHeight,
    TypographyWeight,
}
```

### 2. Updated Metadata Structure

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtoButtonTemplateParamUsage {
    pub name: &'static str,
    pub description: &'static str,
    pub states: &'static [&'static str],
    pub param_type: ProtoButtonTemplateParamType,
    pub param_fields: &'static [ProtoButtonTemplateParamField], // Changed from &[&'static str]
    pub default_source: &'static str,
}
```

### 3. Updated Template Usage

```rust
pub const PROTO_BUTTON_TEMPLATE_USAGE: ProtoButtonTemplateUsage = ProtoButtonTemplateUsage {
    component: "ProtoButton Template",
    parameters: &[
        ProtoButtonTemplateParamUsage {
            name: "variant",
            description: "Base variant resolved from theme before applying parameter overrides.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Enum,
            param_fields: &[ProtoButtonTemplateParamField::Variant], // Typed identifier
            default_source: "ButtonVariant::Standard",
        },
        ProtoButtonTemplateParamUsage {
            name: "background",
            description: "Background override with base + per-state values applied after theme resolution.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Color,
            param_fields: &[
                ProtoButtonTemplateParamField::BackgroundBase,
                ProtoButtonTemplateParamField::BackgroundHovered,
                ProtoButtonTemplateParamField::BackgroundPressed,
                ProtoButtonTemplateParamField::BackgroundFocused,
                ProtoButtonTemplateParamField::BackgroundDisabled,
            ], // All typed identifiers
            default_source: "theme",
        },
        // ... other parameters
    ],
};
```

### 4. Benefits of This Approach

1. **Compile-time Safety**: No more stringly-typed field paths that can drift or contain typos
2. **IDE Support**: Better autocomplete and refactoring support
3. **Maintainability**: Typed identifiers eliminate fragile string paths; full automatic propagation is the planned derive-macro step
4. **Type Safety**: The enum provides guaranteed valid field identifiers
5. **Scalability**: Easy to add new parameters without manual metadata management

## The Derive Macro (Future Implementation)

As described in `docs/proto3.md`, the current typed-enum approach is intended to evolve into a derive macro approach:

```rust
#[derive(ProtoComponent)]
#[proto(prefix = "ProtoButton")]
pub struct ProtoButtonTemplateParams {
    /// Base variant resolved from theme before applying parameter overrides.
    #[proto(default = "ButtonVariant::Standard", states = "standard")]
    pub variant: ButtonVariant,

    /// Base size resolved from theme.
    #[proto(default = "ControlSize::Md", states = "standard")]
    pub size: ControlSize,
    
    // ... other fields
}
```

This future derive step would automatically generate:
- The `ProtoButtonTemplateParamField` enum (or equivalent typed metadata output)
- The `PROTO_BUTTON_TEMPLATE_USAGE` constant
- Proper documentation and state mapping
- Compile-time safety against field name drift
```
