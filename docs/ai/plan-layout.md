# Layout Refactoring Plan: Gallery App Integration

This document serves as the implementation plan and LLM prompt to integrate the modern declarative layout macros into the GPUI Luma gallery introduction pane.

---

## LLM Task Prompt

```text
You are an expert Rust software engineer working on GPUI (a GPU-accelerated immediate-mode UI framework). Your task is to implement the modern layout macros specified in `docs/ai/future-layout-2.md` locally in the gallery app, and use them to refactor the render functions in the gallery's introduction pane.

### CRITICAL RULES:
1. Do NOT modify any business logic, state handling, event emissions, handlers, or existing controls.
2. Only refactor `impl Render` and visual layout hierarchy code to use the new declarative layout macros.
3. Ensure no visual or layout spacing regressions occur (preserve gaps, padding, alignments).
4. Run `cargo fmt`, `cargo clippy`, and tests to verify correctness after refactoring.

---

### STEP 1: Implement Local Layout Macros in the Gallery
Create a new file `apps/gallery/src/gallery/panes/introduction/layouts.rs` and copy the macro definitions (`vstack!`, `hstack!`, and `flow!`) directly from `docs/ai/future-layout-2.md`. 

Register this file in `apps/gallery/src/gallery/panes/introduction/mod.rs` as:
```rust
#[macro_use]
mod layouts;
```

---

### STEP 2: Refactor Gallery Introduction Panels
Refactor the `impl Render` blocks in the following files to use the local layout macros instead of raw nested child chaining:

1. `apps/gallery/src/gallery/panes/introduction/payment_panel.rs`
   - Replace the card's nested children chain with a single, flat `vstack!`.
   - Replace the inline button row container with `hstack!`.

2. `apps/gallery/src/gallery/panes/introduction/system_panel.rs`
   - Replace the card's nested children chain with `vstack!`.
   - Flatten vertical label-control stacks (e.g. Slider/Progress label + slider element) into `vstack!`.

3. `apps/gallery/src/gallery/panes/introduction/workspace_panel.rs`
   - Replace the main layout tree with `vstack!`.
   - Replace label-control/menu sections with `vstack!` / `hstack!`.

4. `apps/gallery/src/gallery/panes/introduction/pane.rs`
   - Simplify the main view composition into nested `vstack!` and `hstack!` compositions.
   - Refactor `render_panel_row` to use `flow!` for wrapping the cards.

---

### STEP 3: Verify the Changes
1. Run `cargo check` to ensure macros compile successfully in all scopes.
2. Run `cargo fmt` to keep the layout structures clean.
3. Run `cargo clippy` and existing tests to ensure no type errors or logic breaks.
```
