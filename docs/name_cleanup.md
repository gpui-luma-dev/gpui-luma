# SDK Controls Naming Cleanup

This project is isolated, so this document defines a **direct cleanup** (no staged migration, no risk plan).

If any rename is undesirable, revert and rerun with adjusted rules.

## Goal

Make `crates/sdk/src/controls` naming consistent and obvious:

1. Public control type is always `Foo`.
2. `FooControl` is internal implementation detail and is **not re-exported** from module `mod.rs`.
3. Constructor style is module-level `new(...)`.
4. Avoid namespaced constructor-only patterns like `Toggle::new(...)`.

## Required naming rules

### Rule A: Public exports

For each control module, public API should expose:

- `Foo` (ergonomic type alias where applicable)
- `FooEvent` (if it emits events)
- `FooBuilder`
- `FooModel`, `FooRenderModel`, template types, and handler aliases as needed

Do **not** publicly re-export `FooControl`.

### Rule B: Internal implementation names

`FooControl` can remain as internal struct names in `control.rs` and internal module wiring.

Examples:

- `ChoiceGroupControl`
- `RadioGroupControl`
- `SliderControl`
- `TextFieldControl`
- `ProgressControl`

These stay internal and are referenced by `pub type Foo = Entity<FooControl>`.

### Rule C: Constructor style

Each control module should provide module-level:

- `pub fn new(...) -> ...Builder`

No module should require `Type::new(...)` as the canonical path.

## Concrete fixes to apply

### 1) Remove `*Control` re-exports from public module surfaces

Update these files:

- `crates/sdk/src/controls/choice_group/mod.rs`
  - change `pub use control::{ChoiceGroupControl, ChoiceGroupEvent};`
  - to `pub use control::ChoiceGroupEvent;`

- `crates/sdk/src/controls/radio_group/mod.rs`
  - change `pub use control::{RadioGroupControl, RadioGroupEvent};`
  - to `pub use control::RadioGroupEvent;`

- `crates/sdk/src/controls/slider/mod.rs`
  - change `pub use control::{SliderControl, SliderDrag, SliderEvent};`
  - to `pub use control::{SliderDrag, SliderEvent};`

- `crates/sdk/src/controls/textfield/mod.rs`
  - change `pub use control::{TextFieldControl, TextFieldEvent};`
  - to `pub use control::TextFieldEvent;`

- `crates/sdk/src/controls/progress/mod.rs`
  - remove `pub use control::ProgressControl;`

Keep the ergonomic aliases in place:

- `pub type ChoiceGroup = Entity<ChoiceGroupControl>;`
- `pub type RadioGroup = Entity<RadioGroupControl>;`
- `pub type Slider = Entity<SliderControl>;`
- `pub type TextField = Entity<TextFieldControl>;`
- `pub type Progress = Entity<ProgressControl>;`

### 2) Normalize `toggle` constructor API

Update `crates/sdk/src/controls/toggle/mod.rs`:

- Add module-level constructor:
  - `pub fn new(id: impl Into<SharedString>) -> ButtonBuilder<bool>`
- Internally this should do what `Toggle::new(...)` currently does.

Then update callsites to use module-level constructor.

Known usage to change:

- `apps/gallery/src/gallery/panes/toggle/pane.rs`
  - replace `Toggle::new(...)` calls with `toggle::new(...)` (or imported `new(...)` from module)

### 3) Keep wrapper controls explicit and consistent

No structural change required for:

- `checkbox`
- `radio_button`
- `switch`

These remain button-backed aliases:

- `type Checkbox = Entity<Button<bool>>`
- `type RadioButton = Entity<Button<bool>>`
- `type Switch = Entity<Button<bool>>`

## Done criteria

Cleanup is complete when:

- `mod.rs` files no longer re-export `*Control` for the target modules.
- `toggle` has module-level `new(...)` and gallery/examples use it.
- Public control usage consistently reads like `controls::<module>::new(...)` + `Foo` aliases.

## Fast verification commands

Run these after renames:

- No public `*Control` re-exports in target modules.
- No remaining `Toggle::new(` usage.
- Workspace builds.

Suggested checks:

- `rg "pub use control::\{[^}]*Control|pub use control::[A-Za-z0-9_]*Control" crates/sdk/src/controls/*/mod.rs`
- `rg "Toggle::new\(" apps crates`
- `cargo check`
