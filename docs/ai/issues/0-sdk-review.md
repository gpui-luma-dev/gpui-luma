# Do a Comprehensive Examination of the SDK and App Usage

This plan establishes API design rules and outlines the steps to perform a comprehensive examination of the builders, models, and controls across the GPUI-Luma SDK to establish incremental consistency.

## Goal & Product Deliverable
The direct product/output of this plan is a new markdown document listing all of the API consistency issues identified across all of the controls in the SDK.

---

## 1. SDK-Wide API Conventions & Rules

To ensure uniform API style, all components must align with the following rules:

### Rule A: Builder Setter Naming
* **Direct Field Configuration**: Any setter mapping directly to a field `x` on the model must be named exactly `.x(value)` (e.g., `.placeholder(text)`, `.enabled(bool)`).
* **`with_` Prefix Reservation**: The `with_` prefix is reserved strictly for builder methods that accept raw arguments (like closures) and wrap/transform them internally (e.g., `.with_row_template(...)`).
* **`set_` Prefix Prohibition**: Builders must never use the `set_` prefix.

### Rule B: Runtime Control Setter Naming
* **`set_` Prefix Obligation**: Any runtime mutator method on a spawned control entity that updates a model field `x` must be named `set_x(...)` (e.g., `set_enabled(...)`, `set_value(...)`).
* **`with_` Prefix Prohibition**: Spawned control entities must never use the `with_` prefix.

### Rule C: Appearance & Style Override Naming
* **Builder**: Standardize on `.appearance_override(override_fn)` for look/style overrides.
* **Control**: Standardize on `.set_appearance_override(override_fn, cx)` for runtime overrides.

### Rule D: Value / Data Payload Naming
* Interactive input controls must standardized on:
  * Builder: `.value(val)`
  * Control Getter: `.value()`
  * Control Setter: `.set_value(val, cx)`

### Rule E: Enabled / Disabled State Parity
* Every interactive control must support an enabled state:
  * Model must hold `enabled: bool`.
  * Builder must expose `.enabled(bool)`.
  * Control must expose `.set_enabled(bool, cx)`.

---

## 2. Instructions to Do a Comprehensive Examination of the SDK and App Usage

To perform the comprehensive API audit, execute the following steps systematically across the codebase:

### Step 1: Inventory All Controls
* List all subdirectories under `crates/sdk/src/controls/` to identify the full suite of widgets.

### Step 2: Audit Builder Setters
* For each control, open its builder definition (typically in `model.rs` or `mod.rs`).
* Check all builder methods and document any that violate **Rule A** (e.g., methods using `set_` prefixes, or using `with_` for direct field assignments).
* Document the builder methods used to set styling, templates, and value payloads, verifying alignment with **Rule C** and **Rule D**.

### Step 3: Audit Control Entity Mutators
* For each control, open its runtime view definition (typically in `control.rs`).
* Check all public mutators and document any that violate **Rule B** (e.g., methods using `with_` prefixes, or missing `set_` prefixes for state modifications).
* Verify template and styling mutator parity against the builder (e.g., if a builder has `.template(...)`, ensure the control has `set_template(...)`).

### Step 4: Audit Enabled State Parity
* For each control, check if `enabled` is defined in its model, builder, control entity, and render model. Document any missing implementations (**Rule E**).

### Step 5: Audit Downstream App Consumption
* Scan the `apps/` directory and `crates/look-shadcn` to locate all builder instantiations (`.new(...)`) and entity updates (`.update(...)`).
* Verify that downstream usage compiles and aligns with the builder methods and control setters.
* Identify any "orphaned" controls that are defined in the SDK but never constructed in the applications.
