# GPUI-Luma Control Design Guidelines

This document details the architectural patterns, API conventions, and styling pipelines for constructing and extending SDK controls in `gpui-luma`. It provides the "how, what, and why" for maintaining consistency and separation of concerns across the codebase.

---

## 1. Architectural Foundation: The LMTP Split

Every interactive control in the SDK is split into four distinct boundaries (Model, Control, Template, Theme) to keep the core layout logic decoupled from product-specific visual design languages:

```text
  1. Model (model.rs)      <-- Configuration payload (Cheap builder output)
           │
           ▼
  2. Control (control.rs)  <-- Stateful GPUI Entity (Coordinates input/focus state)
           │
           ▼
  3. Theme (theme.rs)      <-- Resolves styling (Colors, cached layout scales)
           │
           ▼
  4. Template (template.rs)<-- Presentation engine (Stateless element painting)
```

1. **Model (`model.rs`)**:
   * *What it does:* Holds configuration properties and templates. 
   * *Why:* Cheap and side-effect-free to clone and construct. Used as the blueprint before spawning.
2. **Control (`control.rs`)**:
   * *What it does:* Represents the active GPUI Entity (`cx.new(...)`). Manages runtime states (focus, hover, caret blink, text selection), performs validation, and triggers visual updates (`cx.notify()`).
   * *Why:* Keeps interactive logic separate from raw rendering mechanics.
3. **Theme (`theme.rs` or Look Crate)**:
   * *What it does:* Maps the active mode (light/dark), layout scale variables, and state parameters (hovered, pressed, focused, disabled) to paint values (colors, borders, padding).
   * *Why:* Allows the design details to live in look crates (e.g. `look-shadcn`) while keeping the core SDK lookless.
4. **Template (`template.rs`)**:
   * *What it does:* Receives a readonly render model and handlers, and returns concrete GPUI elements (`Stateful<Div>`).
   * *Why:* Stateless rendering. Templates do not alter control state directly; they map user gestures to control handlers.

---

## 2. API Consistency Conventions (Rules A–E)

To ensure uniform API styles across all widgets, all core and custom controls must strictly align with the following conventions:

### Rule A: Builder Setter Naming
* **Direct Configuration:** Setters mapping directly to a field `x` on the model must be named exactly `.x(value)` (e.g., `.placeholder(text)`, `.enabled(bool)`).
* **Builder Prefix Prohibition:** Builders must **never** use the `set_` prefix.
* **`with_` Prefix Reservation:** The `with_` prefix is reserved strictly for builder methods that accept raw arguments (like closures) and wrap/transform them internally (e.g., `.with_row_template(...)`).

### Rule B: Runtime Control Setter Naming
* **`set_` Prefix Obligation:** Any runtime mutator method on a spawned control entity that updates a model field `x` must be named `set_x(...)` (e.g., `set_enabled(...)`, `set_value(...)`).
* **`with_` Prefix Prohibition:** Spawned control entities must **never** use the `with_` prefix.

### Rule C: Appearance & Style Override Naming
* **Builder-side Override:** Standardize on `.appearance_override(override_fn)` for overriding resolved styles:
  ```rust
  pub fn appearance_override<F>(mut self, appearance_override: F) -> Self
  where
      F: Fn(Appearance) -> Appearance + Send + Sync + 'static
  ```
* **Runtime Control Override:** Standardize on `.set_appearance_override(override_fn, cx)` for runtime changes.
* **Full Appearance Resolvers:** For controls that supply a complete appearance resolver based on full render models (e.g., button-family controls), continue to use `.with_appearance(...)`.

### Rule D: Value / Data Payload Naming
* **Input Controls:** Standardize on `.value(val)` (builder), `.value()` (getter), and `.set_value(val, cx)` (runtime setter) for scalar/text/numeric inputs (e.g. text fields, text areas, sliders, scrollbars, progress bars).
* **Selection Controls:** Retain selection-specific semantics (e.g. `.selected(bool)`, `.items(vec)`, `.query(text)`) instead of forcing them to use generic value naming.

### Rule E: Enabled / Disabled State Parity
* **Top-Level Controls:** Every interactive control must support an enabled state:
  * Model holds `enabled: bool`.
  * Builder exposes `.enabled(bool)`.
  * Control exposes `.set_enabled(bool, cx)`.
* **Per-Item Enabled Parity:** Composite controls (e.g., accordions, control groups, list boxes, navigation items) must support individual per-item enabled state parameters.

---

## 3. The "Why" Behind Appearance Overrides

During early design phases, Luma prototyped a state-aware template parameterization system (e.g., `ProtoButtonTemplateParams`). This was designed to allow explicit state overrides for colors, margins, and sizes at compile-time.

### Why Parameterization was Retired
* **Boilerplate & Compile-Time Weight:** It required massive macro support (`#[derive(ProtoComponent)]` via `darling`) to map metadata strings safely, which slowed down build cycles.
* **UX Fallback Confusion:** Centralizing state-fallback resolution (base override vs. hover override vs. theme fallback) was difficult to explain and debug during customization.

### The Role of `appearance_override`
To avoid over-engineering, Luma consolidated around the closure-based `appearance_override` model. It serves two distinct purposes:

1. **The Escape Hatch:**
   It provides a simple, zero-boilerplate escape hatch for downstream apps to modify resolved layouts after the theme resolver finishes.
2. **Design Discovery:**
   Using `appearance_override` in production app code signals a gap where the theme system wasn't fully configured to support a specific variation. 
   * **Prototyping:** Hack layout adjustments inline using the override to maintain speed.
   * **Promotion:** Once an override pattern is repeated across different files or apps (e.g. compact, monospace inputs for tokens), refactor it into standard `ControlSize` presets or `TextFieldVariant` options mapped directly through the look crate's stylesheet config (`style.toml`).
