# Theme & Controls Ergonomics Work Plan

This document outlines the roadmap for standardizing the GPUI-Luma Radix theme system, removing legacy style patterns, and introducing declarative control builder extensions.

## 1. Objectives

- **Code-First Ergonomics**: Simplify widget declaration. Replace verbose `.with_appearance` and `.template` closures with clean, chainable builder helpers (e.g., `theme.button("id").primary()`).
- **Remove Legacy Patterns**: Deprecate/remove `ButtonKind` and `ButtonVariant` mapping. Use direct style variant mapping (`RadixButtonStyle`). (**COMPLETED**)
- **Theme Code Consolidation**: Merge tiny and single-line files in `crates/sdk/src/theme/radix` into logical modules to reduce directory clutter. (**COMPLETED**)

---

## 2. Proposed Roadmap

### Phase 0: Pre-Task (Consolidation & Directory Flattening) - **COMPLETED**
Flatten the `properties/` nested directory directly into `radix/`.
- Removes all 20+ single-line files.
- Eliminates the extra nesting directory.
- Brings all controls in line with the structure already used by `button.rs`.
- Requires very minor changes to imports (simply moving the files and updating `mod.rs`).

### Phase 1: Style Variant Standardization
Align all core controls to direct `RadixButtonStyle` options.
- **Task 1.1**: Audit and standardize button style variant mappings. Ensure `RadixButtonStyle` (Primary, Secondary, Outline, Ghost) is the primary driver of appearance.
- **Task 1.2**: Audit and align checkbox, switch, and radio button styles to reference the standardized Radix styles directly.
- **Task 1.3**: Deprecate legacy structures like `ButtonKind` and `ButtonVariant` which add an unnecessary layer of indirection. (**COMPLETED**)

### Phase 2: Control Builder Extensions & Theme Factory - **COMPLETED**
Implement two complementary helper APIs to instantiate and style controls:

#### Part A: Theme Factory (`RadixThemeControlExt`) - **COMPLETED**
Implement an extension trait on `Arc<RadixTheme>` allowing controls to be constructed directly from the theme with templates pre-bound.
- **Task 2.1**: Implement `theme.button("id")`, `theme.checkbox("id")`, `theme.switch("id")`, and `theme.textfield("id")` factory methods. (**COMPLETED**)
- **Task 2.2**: Implement style-specific factories where style is part of the name (e.g., `theme.primary_button("id")`, `theme.secondary_button("id")`). (**COMPLETED**)
- **Task 2.3**: Add factories for remaining controls including `radio_group`, `listbox`, `context_menu`, etc. (**COMPLETED**)

#### Part B: Builder Extensions (`RadixButtonStyleExt`, etc.) - **COMPLETED**
Implement extension traits on the control builders to easily apply styles explicitly.
- **Task 2.4**: Style chainers on `ButtonBuilder<()>` and `TextFieldBuilder` (e.g., `.primary(&theme)`). (**COMPLETED**)
- **Task 2.5**: `CheckboxBuilder` / `SwitchBuilder` newtypes with `RadixCheckboxStyleExt` and `RadixSwitchStyleExt` so `.primary(&theme)` applies the correct template without `ButtonBuilder<bool>` ambiguity. (**COMPLETED**)

---

## 3. Migration Plan (e.g., `payment_panel.rs`) - **COMPLETED**

Gallery control construction migrated to theme factories. Template showcase panes (`choice_controls_template`, `selector_controls_template`) and custom template demos still bind templates directly where factories do not apply or where local theme overrides are intentional (e.g. tabs local-theme example, context menu radial/default gallery templates, radio group indented/card layouts). Floating menu is a render primitive — use `radix_theme.floating_menu_theme()` rather than a spawned control factory.

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

### After Option A (Theme Factory):
```rust
use gpui_luma::theme::radix::prelude::*;

let submit_button = radix_theme.primary_button("intro-submit")
    .label("Submit")
    .spawn(cx);

let cancel_button = radix_theme.secondary_button("intro-cancel")
    .label("Cancel")
    .spawn(cx);
```

### After Option B (Builder Extensions):
```rust
use gpui_luma::theme::radix::prelude::*;

let submit_button = Button::new("intro-submit")
    .label("Submit")
    .primary(&radix_theme)
    .spawn(cx);

let cancel_button = Button::new("intro-cancel")
    .label("Cancel")
    .secondary(&radix_theme)
    .spawn(cx);

let terms = checkbox::new("terms")
    .label("Accept terms")
    .primary(&radix_theme)
    .spawn(cx);
```
