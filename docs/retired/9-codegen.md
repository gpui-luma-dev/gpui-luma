# Control API Codegen for Luma Studio

## Purpose

Luma Studio exposes SDK controls as interactive documentation and inspection pages. Those pages need a reliable inventory of the public API for each control:

- look factories from `gpui-luma-look-shadcn`;
- SDK builders and builder methods;
- runtime control methods and getters;
- public types, aliases, and re-exports;
- public event enums and variants.

Rust does not provide the kind of runtime reflection needed to discover this API from a running application. Maintaining the inventory manually is also unreliable: methods can be renamed, signatures can change, events can be added, and documentation can drift without a visible compile error.

This design uses an on-demand workspace code generator to parse Rust source with `syn` and emit a deterministic Rust artifact consumed by Luma Studio.

The generator is an API capture tool. It is not the Luma Studio presentation model and should not contain UI-specific filtering, layout decisions, or curated exposition prose.

## Goals

1. Capture the public Rust API for a selected control accurately and repeatably.
2. Include signatures, types, aliases, trait-provided methods, events, docs, and source provenance.
3. Keep normal application builds free of AST parsing and generator dependencies.
4. Produce checked-in Rust output that Luma Studio can compile and consume.
5. Detect stale generated output in CI or during development.
6. Make missing source documentation visible rather than silently inventing descriptions.
7. Support incremental adoption one control at a time, beginning with Button.

## Non-goals

- Replacing Rust documentation or rustdoc.
- Generating the final Studio presentation model.
- Automatically inventing behavioral claims such as event triggers or disabled-state rules.
- Capturing every private implementation detail as user-facing documentation.
- Replacing tests for interaction behavior.
- Performing runtime reflection inside SDK or application builds.

## Starting conditions

The work may begin in a repository where the old hand-written exposition tables have already been removed or changed. The generator must therefore not depend on existing `PublicInterfaceSpec` arrays, legacy rendering helpers, or a particular exposition UI.

The first output is a raw, typed Rust capture artifact. Luma Studio can later introduce a presentation model after the capture format is proven.

## Proposed workspace layout

```text
gpui-luma/
├── .cargo/
│   └── config.toml                 # cargo xtask alias
├── xtask/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── parser.rs
│       └── codegen.rs
└── apps/luma-studio/
    └── src/studio/controls/
        └── generated/
            └── button_api.rs       # checked-in raw capture
```

The exact generated directory may change as the Studio module structure evolves. The important boundary is that the generated artifact belongs to Luma Studio and is independent of the eventual presentation model.

## Commands

The workspace should expose an `xtask` package through Cargo:

```toml
[alias]
xtask = "run --package xtask --"
```

The required commands are:

```bash
cargo xtask gen-docs
cargo xtask check-docs
```

Recommended Just recipes:

```make
luma-studio-api:
    cargo xtask gen-docs

luma-studio-api-check:
    cargo xtask check-docs
```

`gen-docs` writes deterministic generated Rust. `check-docs` regenerates in memory and compares the result with the checked-in artifact without modifying files.

## Capture model

The generated Rust should contain data records, not UI widgets. The initial schema can be small, but it should preserve enough information to support later presentation decisions.

### Methods

Each captured method should include:

- stable API identity;
- owning group or source surface;
- method name;
- cleanly formatted Rust signature;
- raw rustdoc, if present;
- source file and location;
- optional raw implementation/body tokens for tooling;
- whether it came from an inherent impl, trait declaration, or trait implementation.

The signature should be formatted from the `syn::Signature` structure, not produced by calling `TokenStream::to_string()`. Token-stream output is syntactically valid but exposition-hostile, for example:

```text
fn button (& self , id : impl Into < SharedString >) -> ButtonBuilder < () >
```

The capture should instead preserve a normalized form such as:

```text
fn button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>
```

Formatting is still data normalization; deciding whether a method should appear in a UI section is presentation policy.

### Types

Capture public structs, enums, traits, and type aliases relevant to the control. A type record should preserve:

- kind;
- declared name and canonical path;
- declaration or structured fields/variants;
- rustdoc;
- source provenance.

Type collection should eventually be reachability-based. Starting from the control, builder, factory, and event roots, follow public parameter, return, alias, and trait references. This prevents unrelated presenter infrastructure or shared implementation types from overwhelming a Button page.

### Events

Capture public event enums and variants as factual API data:

- canonical enum identity;
- exported aliases;
- variant name;
- named or unnamed fields;
- `#[non_exhaustive]` status;
- rustdoc and source provenance.

The generator should not synthesize rows such as “disabled buttons emit no event.” That is behavioral documentation and belongs either in source rustdoc, tests, or a separately evidenced behavior-analysis layer.

### Aliases and re-exports

Rust APIs often expose one type under another public name. Button currently illustrates this pattern when a command event is re-exported as a button event.

Capture aliases explicitly:

```text
exported: ButtonEvent
target: crate::controls::command::CommandEvent
```

Downstream consumers should be able to resolve a canonical identity while retaining alias provenance. A Studio page should not show duplicate event tables merely because the same enum is reachable through two names.

### Rustdoc

Capture source rustdoc as-is, with a presence/status field such as:

- documented in source;
- missing;
- inherited or resolved from a trait declaration.

Do not fill missing docs with generated prose in the capture layer. A fallback such as “Factory method for ButtonBuilder” is presentation text and is too weak to serve as an API contract.

## Source discovery

The initial Button capture should explicitly identify the source surfaces:

```text
Look factories:
  crates/look-shadcn/src/controls/ext.rs

Builder/model APIs:
  crates/sdk/src/controls/command/button/model.rs

Runtime control APIs:
  crates/sdk/src/controls/command/button/control.rs

Button event definition:
  crates/sdk/src/controls/command/core.rs

Trait-provided builder APIs:
  crates/sdk/src/controls/presenter.rs
```

The control naming convention can later drive source discovery, but a small explicit control manifest is preferable to relying solely on filename guesses. It makes the intended capture roots reviewable and handles controls whose APIs are spread across modules.

## Look factory capture

Look factories are more than signatures. Their implementation establishes the relationship between a factory and the SDK builder:

```rust
fn primary_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
    Button::new(id).template(self.button_template(ShadcnButtonStyle::Primary))
}
```

The capture should preserve the implementation body or a structured construction trace for tooling. It should not convert that body into final UI prose.

For Button, the raw capture can reveal:

- which `ShadcnButtonStyle` is selected;
- whether the factory starts from a text or icon builder;
- whether content-only factories disable elevation or adorners.

Those facts can later support a Studio presentation, but they remain source-derived data until then.

## Documentation backfill

Automated capture exposes missing documentation but cannot reliably author the intended contract by itself. The recommended workflow is:

1. Run the generator for the target control.
2. Inspect generated methods, types, and events with empty rustdoc.
3. Read the SDK/look implementation and existing tests.
4. Add concise rustdoc to the public SDK/look API.
5. Regenerate the artifact.
6. Review the resulting capture and run validation.

The backfill should be modest for an open-source component SDK. Document intent and non-obvious behavior, not every obvious assignment.

High-value documentation includes:

- what the control represents;
- what a builder method changes;
- what a runtime setter changes and whether it notifies;
- event meaning and emission conditions;
- look factory differences;
- constraints, clamping, defaults, or no-op behavior.

For Button, this includes the builder/control API, Shadcn style factory differences, `HasPresenter` methods, and command event variants. Shared infrastructure should be documented separately when it becomes a first-class Studio surface.

## Presentation boundary

The generated capture should not decide:

- which rows are displayed;
- how methods are grouped visually;
- whether construction is expanded or collapsed;
- which types are considered “high value” for a page;
- how missing docs are worded;
- whether an alias is displayed or merged;
- what event trigger labels should say.

Those decisions belong to a later Luma Studio presentation model. A possible future pipeline is:

```text
Rust source
    ↓
syn capture
    ↓
normalized generated Rust data
    ↓
canonical identity / alias resolution
    ↓
control-specific presentation selection
    ↓
Luma Studio rendering
```

The source capture should remain useful even if Studio changes its visual model.

## Validation

The initial proof of concept should validate all of the following:

1. Generated output parses as valid Rust.
2. Generated output compiles when included by Luma Studio.
3. Re-running generation produces byte-for-byte stable output.
4. `check-docs` detects a stale artifact.
5. Public builder/control methods are captured from all relevant impl blocks.
6. Trait-provided methods such as `HasPresenter::content` and `HasPresenter::label` are visible.
7. Public types and aliases are captured with source provenance.
8. `ButtonEvent` and its canonical command-event source are not presented as duplicate identities.
9. Missing rustdoc is visible in the raw capture.

Recommended parser fixtures should cover:

- generic methods and where clauses;
- associated functions;
- trait default methods;
- public `use` aliases and grouped re-exports;
- named and unnamed enum fields;
- non-exhaustive enums;
- multiline signatures and docs.

## CI and developer workflow

When public SDK/look APIs change:

```bash
just luma-studio-api
cargo fmt --all
just luma-studio-api-check
```

The generated artifact should be reviewed together with the source API change. CI should run `check-docs` and fail if generated output is stale.

The generator itself should not run as part of ordinary `cargo build` or `cargo run` for Luma Studio. The checked-in artifact is compiled normally; `syn` and codegen dependencies are used only by the explicit xtask.

## Known issues and future work

### Missing documentation

The generator can report missing docs but cannot determine the intended contract from syntax alone. The durable solution is concise rustdoc in the SDK/look source, backfilled with source and test review.

### Type noise

Capturing all public types from every involved file produces unrelated presenter and shared-command types. Move from per-file public-item collection to a reachable-type graph rooted at the selected control API.

### Duplicate identities

Aliases and re-exports can make one type appear under several names. Add canonical path resolution before presentation consumption.

### Raw construction output

Implementation bodies are valuable for analysis but too noisy for normal UI. Retain them in the raw artifact, then expose structured construction facts or a collapsed debug view later.

### Shared generated schema

Defining `ApiMethod`, `ApiType`, `ApiEvent`, and `ApiAlias` in every generated control file does not scale. Hoist these records into a shared generated support module and keep per-control files limited to data constants.

### Behavioral documentation

Event trigger and disabled-state explanations require semantic understanding. Prefer source rustdoc and tests first; only add automated behavior analysis if it can provide evidence and provenance rather than guesses.

## Recommended rollout

### Phase 1: Button capture

- Add the xtask package and Cargo alias.
- Capture Button look factories, builder/control methods, events, traits, types, and aliases.
- Emit dependency-free Rust.
- Add `gen-docs` and `check-docs` commands.

### Phase 2: Documentation backfill

- Add concise rustdoc to the Button SDK and Shadcn look APIs.
- Regenerate and verify that the expected public surfaces are documented.

### Phase 3: Studio read-only inspection

- Consume the raw artifact without requiring the final presentation model.
- Display signatures, source names, and available rustdoc.
- Keep construction and raw declarations hidden or explicitly marked as debug data.

### Phase 4: Shared schema and identity resolution

- Hoist generated record types.
- Add stable IDs, source spans, canonical paths, alias resolution, and reachable type capture.

### Phase 5: Additional controls

- Expand the source manifest and capture one control at a time.
- Add parser fixtures for each new structural pattern.
- Add control-specific presentation only after the raw data is trusted.

## Decision summary

- Use an on-demand `xtask`, not `build.rs` or runtime reflection.
- Emit checked-in Rust for Luma Studio.
- Treat source rustdoc as the API documentation source of truth.
- Backfill missing docs in the SDK/look rather than maintaining long-term sidecar prose.
- Keep raw capture complete and presentation-agnostic.
- Resolve aliases and type reachability before scaling to many controls.
- Preserve raw implementation data for tooling, but do not make it the default UI representation.
