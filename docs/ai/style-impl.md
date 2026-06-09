# Implementation Plan: Stylesheet Decoupling (`style.toml`)

This document outlines the step-by-step implementation plan for transitioning `look-shadcn` controls from hardcoded procedural macro lookup tables to a dynamic, unified stylesheet (`style.toml`) mapping system.

---

## Current Status & Verification
* **Docs & Configuration**: `style.md`, `style_concerns.md`, and a complete `style.toml` mapping 18 controls are in place. No further doc updates are required.
* **Compiler Sanity**: Current code compiles and all existing tests pass under the macro lookup system.
* **Goal**: Reorganize the code so that styling is fully thinned out of Rust controls, hot-reloadable at runtime, and easily inspectable without macro code-gen.

---

## Phase 1: Core Parsing Engine & Traits
Implement the parsing layer and token evaluation traits in `crates/look-shadcn`.

### 1. Deserializable Config Models
* **File**: `crates/look-shadcn/src/stylesheet/config.rs` (new)
* Define Serde structures (`StylesheetConfig`, `ButtonStylesheet`, `CheckboxStylesheet`, etc.) mapping directly to the TOML sections.

### 2. Standard Input Translation (`AsSelectorState`)
* **File**: `crates/look-shadcn/src/stylesheet/selector.rs` (new)
* Implement `SelectorMap` and the `AsSelectorState` trait to standardize input translation:
  ```rust
  pub trait AsSelectorState {
      fn to_selector_map(&self) -> SelectorMap;
  }
  ```
* Implement the matching engine that evaluates sequential rules using the mapped selectors.

### 3. Value Resolution (`ResolveValue<T>`)
* **File**: `crates/look-shadcn/src/stylesheet/resolve.rs` (new)
* Define the `ResolveValue<T>` trait.
* Implement:
  * `ResolveValue<f32>`: Handles dimensions, padding, radii, and parses simple `calc()` arithmetic.
  * `ResolveValue<Vec<BoxShadow>>`: Parses shadow parameters (blur, spread, color, offsets) into GPUI shadows.
  * `ResolveValue<SharedString>`: Resolves font family tokens.
  * **Variable Interpolation**: Evaluates `@field` references (e.g. `border = "@background"`) within resolved rules.

---

## Phase 2: Bootstrap Integration (Button Proof of Concept)
Integrate the engine into the main lifecycle and migrate the first control.

### 1. Load the Stylesheet
* **File**: `crates/look-shadcn/src/lib.rs` (or `ShadcnLook`)
* Parse `style.toml` during theme startup.
* Store the `StylesheetConfig` on `ShadcnLook` or `AppearanceContext`.

### 2. Migrate Button Control
* **File**: `crates/look-shadcn/src/controls/button.rs`
* Remove `declare_look_table!` macro call.
* Rewrite `button_palette` to query the stylesheet, apply derived border fallbacks, construct the focus ring spec, and assemble the palette.
* **Verify**: Run `cargo test` to ensure all button visual tests and hover states compile and pass.

---

## Phase 3: Roll Out to Remaining Controls
Iteratively migrate the remaining 17 controls. We group them by complexity to minimize regression risk.

### Group A: Simplest Controls (Color-only, few states)
* **Controls**: `split_view`, `progress`, `control_group`.
* **Changes**: Remove macro tables, query flat rules, map directly to output structs.

### Group B: Standard Controls (Derived fields, hover/pressed/disabled)
* **Controls**: `checkbox`, `radio`, `switch`, `slider`, `scrollbar`, `accordion`, `resizable_panels`.
* **Changes**: Move padding/sizing constants into the metrics tables; resolve fields dynamically; implement derived logic (like checkbox indicators) in the resolver.

### Group C: Composite & Menu Controls (Multi-part schemas)
* **Controls**: `listbox`, `floating_menu`, `popup_menu`, `list_view`, `tree_view`, `navigation_sidebar`, `tabs_navigation`.
* **Changes**: Integrate sub-template queries (e.g. container vs row list view); replace hardcoded offset/padding calculations.

### Group D: Input Controls (Complex state axes)
* **Controls**: `textfield`, `textarea`.
* **Changes**: Remap caret, selection backgrounds, invalid states, and input focus visibility.

---

## Phase 4: Clean Up & Inspection Pivot
Retire legacy systems and pivot the Inspector.

### 1. Retain Look Inspector via TOML
* **File**: `crates/look-shadcn-inspect`
* Pivot the Look Probe UI to iterate over the loaded `StylesheetConfig` rather than calling generated metadata functions.
* Cleanly map selected rules to the Inspector list view.

### 2. Macro Retirement
* Remove the AST metadata generator inside `crates/look-shadcn-macros/src/lib.rs`.
* Retire `declare_look_table!` once all controls are migrated.

---

## Verification Plan

### Automated Tests
* `cargo test -p gpui-luma-look-shadcn` after migrating each control group.
* Ensure visual assertions (like hover contrast assertions in `button.rs` and `switch.rs`) remain identical.

### Manual Verification
* Run the Gallery Look Probe application.
* Verify that changing theme presets updates colors and metrics correctly.
* Inspect the rule tables live inside the Look Probe UI.
