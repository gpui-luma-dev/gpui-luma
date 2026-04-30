# SDK Naming Conventions

This document defines the naming conventions for `crates/sdk/src/controls`.

## Goals

- Keep the public control API predictable.
- Keep implementation details out of public module surfaces.
- Keep constructor usage consistent across controls.

## Core conventions

### 1) Public control type naming

Use `Foo` as the public control type name.

Examples:

- `ChoiceGroup`
- `Slider`
- `TextField`
- `Progress`

When the implementation is a dedicated control struct, expose `Foo` as the primary type alias:

- `pub type Foo = Entity<FooControl>;`

### 2) Internal implementation naming

Use `FooControl` for implementation structs in `control.rs`.

Examples:

- `ChoiceGroupControl`
- `SliderControl`
- `TextFieldControl`
- `ProgressControl`

`FooControl` is an implementation detail and should **not** be publicly re-exported from module `mod.rs`.

### 3) Constructor naming

Each control module should provide a module-level constructor:

- `pub fn new(...) -> ...Builder`

Canonical usage style:

- `controls::<module>::new(...)`

Avoid making `Type::new(...)` the only constructor style.

### 4) Event, builder, model, and template names

Use `Foo`-prefixed names for related API types:

- `FooEvent`
- `FooBuilder`
- `FooModel`
- `FooRenderModel`
- `FooTemplate`
- `FooTemplateHandlers` (when needed)

### 5) Wrapper control modules

For thin wrappers around shared button primitives (e.g. checkbox-like controls), keep names explicit and ergonomic.

Examples:

- `Checkbox`
- `RadioButton`
- `Switch`

These may be aliases over shared button entities when appropriate.

## Public module surface rules (`mod.rs`)

For each control module, prefer this public shape:

- export `Foo` (ergonomic type alias, where applicable)
- export `FooEvent` (if any)
- export `FooBuilder`, model/render model types, template types, and handler aliases
- do **not** re-export `FooControl`

## Current normalized patterns in this repository

- `choice_group`, `slider`, `textfield`, `progress` do not publicly re-export `*Control`.
- `toggle` supports module-level `new(...)` for consistent constructor usage.

## Quick checks

- Ensure no public `*Control` re-exports:
  - `rg "pub use control::\{[^}]*Control|pub use control::[A-Za-z0-9_]*Control" crates/sdk/src/controls/*/mod.rs`
- Ensure no stale namespaced toggle constructor usage:
  - `rg "Toggle::new\(" apps crates`
