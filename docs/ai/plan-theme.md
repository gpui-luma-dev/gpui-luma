# Theme & Controls Ergonomics Work Plan

This document outlines the roadmap for standardizing the GPUI-Luma Radix theme system, removing legacy style patterns, and introducing declarative control builder extensions.

## 1. Objectives

- **Code-First Ergonomics**: Simplify widget declaration. Replace verbose `.with_appearance` and `.template` closures with clean, chainable builder helpers (e.g., `.primary()`, `.secondary()`).
- **Remove Legacy Patterns**: Deprecate/remove `ButtonKind` and `ButtonVariant` mapping. Use direct style variant mapping (`RadixButtonStyle`). (COMPLETED)
- **Theme Code Consolidation**: Merge tiny and single-line files in `crates/sdk/src/theme/radix` into logical modules to reduce directory clutter.

---

## 2. Proposed Roadmap

### Phase 1: Style Variant Standardization
Align all core controls to direct `RadixButtonStyle` options.
- **Task 1.1**: Audit and standardize button style variant mappings. Ensure `RadixButtonStyle` (Primary, Secondary, Outline, Ghost) is the primary driver of appearance.
- **Task 1.2**: Audit and align checkbox, switch, and radio button styles to reference the standardized Radix styles directly.
- **Task 1.3**: Deprecate legacy structures like `ButtonKind` and `ButtonVariant` which add an unnecessary layer of indirection.

### Phase 2: Control Builder Extension Traits
Implement builder extension traits that pull from `active_radix_theme()` under the hood, removing theme parameters and closures from view panels.
- **Task 2.1**: Implement `RadixButtonExt` on `ButtonBuilder` containing:
  - `.primary()`
  - `.secondary()`
  - `.outline()`
  - `.ghost()`
- **Task 2.2**: Implement `RadixCheckboxExt` on `CheckboxBuilder`:
  - `.primary()`
  - `.secondary()`
- **Task 2.3**: Implement `RadixSwitchExt` on `SwitchBuilder`:
  - `.primary()`
  - `.secondary()`
- **Task 2.4**: Expose these traits in a new `gpui_luma::theme::radix::prelude` module for easy import.

### Phase 3: Theme Module Consolidation
Review and consolidate small, single-item modules under `crates/sdk/src/theme/radix`.
- **Task 3.1**: Consolidate `active.rs`, `usage.rs`, and minor template bindings into `mod.rs` or structured core modules to reduce directory clutter.

---

## 3. Migration Plan (e.g., `payment_panel.rs`)

Once the new API is ready, migrate target panels like `payment_panel.rs`:

### Before (Verbose/Imperative):
```rust
let submit_button = Button::new("intro-submit")
    .label("Submit")
    .with_appearance({
        let theme = radix_theme.clone();
        move |model| theme.primary_button(model.role, model.size, model.state)
    })
    .spawn(cx);

let cancel_button = Button::new("intro-cancel")
    .label("Cancel")
    .with_appearance({
        let theme = radix_theme.clone();
        move |model| theme.secondary_button(model.role, model.size, model.state)
    })
    .spawn(cx);
```

### After (Declarative):
```rust
use gpui_luma::theme::radix::prelude::*;

let submit_button = Button::new("intro-submit")
    .label("Submit")
    .primary()
    .spawn(cx);

let cancel_button = Button::new("intro-cancel")
    .label("Cancel")
    .secondary()
    .spawn(cx);
```
