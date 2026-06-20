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
* **Exceptions for Semantic APIs**: Established semantic builder APIs like `.with_data(...)` (which distinguishes itself from typed variants) are permitted, but should be documented. New APIs or non-standard builder setters must follow the direct field configuration rule or note explicit compatibility impact.
* **`set_` Prefix Prohibition**: Builders must never use the `set_` prefix.

### Rule B: Runtime Control Setter Naming
* **`set_` Prefix Obligation**: Any runtime mutator method on a spawned control entity that updates a model field `x` must be named `set_x(...)` (e.g., `set_enabled(...)`, `set_value(...)`).
* **`with_` Prefix Prohibition**: Spawned control entities must never use the `with_` prefix.

### Rule C: Look & Style Override Naming
* **Post-Resolution Look Overrides**:
  * **Builder**: Standardize on `.look_override(override_fn)` for overriding post-resolution styles.
  * **Control**: Standardize on `.set_look_override(override_fn, cx)` for runtime overrides.
* **Full Look Resolvers/Sources**:
  * For controls that supply a complete look resolver based on full render models (e.g. button-family controls), continue to use `.with_look(...)`.

### Rule D: Value / Data Payload Naming
* **Scalar/Text/Numeric Input Controls**: Standardize on the following for controls whose primary public payload is a scalar/text/numeric value (e.g., `textfield`, `textarea`, `slider`, `scrollbar`, `progress`):
  * **Builder**: `.value(val)`
  * **Control Getter**: `.value()`
  * **Control Setter**: `.set_value(val, cx)`
* **Button & Selection Controls**: Do not force button-family and selection controls (e.g. checkbox, switch, radio button) into `.value(...)`. Retain selection-specific semantics (e.g., `.selected(...)`, `.items(...)`, `.query(...)`) or payload data semantics (`.data(...)` / `.set_data(...)` / `.with_data(...)`).

### Rule E: Enabled / Disabled State Parity
* **Top-Level Controls**: Every interactive control must support an enabled state:
  * Model must hold `enabled: bool`.
  * Builder must expose `.enabled(bool)`.
  * Control must expose `.set_enabled(bool, cx)`.
* **Per-Item Enabled Parity**: For composite/item-based controls (e.g., `AccordionItem`, `ControlGroupItem`, `ListBoxItem`, `NavNode`, `MenuItem`, `SelectionPanelItem`), per-item enabled parity must be audited and supported individually.

---

## 2. Instructions to Do a Comprehensive Examination of the SDK and App Usage

To perform the comprehensive API audit, execute the following steps systematically across the codebase:

### Step 1: Inventory All Controls
* Inventory controls using `crates/sdk/src/controls/mod.rs` to identify all exported public modules and nested control families, rather than just first-level directories.
* Ensure nested families are accounted for, such as:
  * Command family: `command/button`, `command/icon_button`
  * Color family: `color/color_arc`, `color/color_ring`, `color/color_slider`, `color/color_field`
  * File-backed controls directly in `controls/mod.rs`: `label`, `menu_item`, `icon`

### Step 2: Audit Builder Setters
* For each control, open its builder definition (typically in `model.rs` or `mod.rs`).
* Check all builder methods and document any that violate **Rule A** (e.g., methods using `set_` prefixes, or using `with_` for direct field assignments).
* Document the builder methods used to set styling, templates, and value payloads, verifying alignment with **Rule C** and **Rule D**.

### Step 3: Audit Control Entity Mutators & Wrapper Layers
* For each control, open its runtime view definition (typically in `control.rs`).
* **Wrapper Auditing**: Identify controls that act as thin wrappers over shared machinery (e.g., `checkbox`, `switch`, `radio_button`, `toggle` which wrap shared button machinery in `crates/sdk/src/controls/command/button/control.rs`). Ensure the audit inspects both the wrapper API and the shared underlying type runtime mutators.
* Check all public mutators and document any that violate **Rule B** (e.g., methods using `with_` prefixes, or missing `set_` prefixes for state modifications).
* Verify template and styling mutator parity against the builder (e.g., if a builder has `.template(...)`, ensure the control has `set_template(...)`).

### Step 4: Audit Enabled State Parity
* For each control, check top-level enabled state parity as well as per-item enabled parity for composite controls. Document any missing implementations (**Rule E**).

### Step 5: Audit Downstream App Consumption
* Scan the `apps/` directory and `crates/look-shadcn` to locate:
  * Raw builder instantiations (`.new(...)`)
  * Themed control factories/helpers defined in `crates/look-shadcn/src/controls/ext.rs` (such as `look.split_view(...)`, `look.textfield(...)`, `look.primary_checkbox(...)`)
  * App-side `.update(...)` closures that call runtime setters
* Verify that downstream usage compiles and aligns with the builder methods and control setters.
* Identify any "orphaned" controls that are defined in the SDK but never constructed in the applications.

---

## 3. Findings Schema

For each consistency issue identified during the audit, the report must document findings using the following structured fields:

* **Control Name**: The name of the widget/control (e.g., `TextField`, `ListView`, `SelectionPanel`).
* **Layer**: One or more of `builder`, `runtime`, `wrapper`, `look-factory`, `app-usage`.
* **Current API**: The signature or style of the existing API.
* **Expected API / Violated Rule**: The expected API or the specific rule violated (e.g., Rule B, Rule C).
* **File Path**: Absolute or relative path to the implementation file.
* **Downstream Usages**: Description of where and how the API is consumed in apps or looks.
* **Migration Impact**: Anticipated effort or breakages from changing the API.
* **Compatibility Shim Needed?**: `Yes` / `No` (indicating if a deprecated shim is needed for transition).
