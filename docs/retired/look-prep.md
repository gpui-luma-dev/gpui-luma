# Look Prep Plan: Splitting Radix into a Separate Crate

This document outlines the step-by-step preparation plan to extract the `radix` styling system out of the core SDK (`crates/sdk`) into its own downstream crate (`crates/theme-radix`). 

Executing this plan guarantees that the core SDK remains styling-agnostic, and allows the upcoming `ShadcnLook` implementation to be built as a parallel downstream crate without namespace conflicts.

---

## The Extraction Heuristic

To keep the SDK core decoupled, apply this litmus test to every compile-time fix during the migration:

> **"Would I want a future `gpui-luma-look-skeuo` (skeuomorphic styling) crate to depend on this exact symbol?"**
> * **If Yes**: The code belongs in the core SDK (`gpui-luma`).
> * **If No**: The code belongs in the specific look crate (`gpui-luma-theme-radix`).
> * **If Unclear**: Default to keeping it out of the core SDK.

### What is Acceptable (Fine):
* **Mechanical Namespace Fixes**: Modifying downstream imports to point to the new look crate.
* **Moving Radix-Specific Helpers**: Housing things like `RadixButtonStyle` and theme builders inside the radix crate.
* **Temporary Local Cleanup**: Resolving dependencies (like `LumaTheme::native()`) inside the radix crate code during extraction.
* **Preserving Generic Primitives**: Keeping core structural layout and base styling structures inside the SDK.

### What is Unacceptable (Not Fine):
* **Shared Leakage**: Moving universal default config schemas or helpers into the radix crate if future looks will also need them.
* **Implicit Coupling**: Introducing new core traits or structures that are secretly tailored to radix's design assumptions.
* **Shortcut Re-exports**: Adding re-exports inside the core SDK to make radix "feel" built-in.
* **Opaque Wildcards**: Using massive wildcard overrides or preludes that obscure the boundary between the crates.

---

## Phase 1: Create the new Crate (`gpui-luma-theme-radix`)

1. **Create Crate Directory**:
   * Create the folder `crates/theme-radix` containing a standard Cargo library structure:
     ```text
     crates/theme-radix/
     ├── Cargo.toml
     └── src/
     ```

2. **Configure `crates/theme-radix/Cargo.toml`**:
   * Declare the crate name and set `gpui-luma` (the SDK) as a dependency:
     ```toml
     [package]
     name = "gpui-luma-theme-radix"
     version = "0.1.0"
     edition = "2024"

     [dependencies]
     gpui = { workspace = true }
     gpui-luma = { path = "../sdk" }
     anyhow = { workspace = true }
     serde = { workspace = true }
     toml = { workspace = true }
     lucide-icons = { workspace = true }
     ```

3. **Register in Workspace**:
   * Add the new path to the workspace root `Cargo.toml` under `members`:
     ```toml
     members = [
         "crates/sdk",
         "crates/theme-radix",
         "apps/gallery",
         "apps/theme-studio"
     ]
     ```

---

## Phase 2: Move the Source Files

1. **Move Code Files**:
   * Move the entire contents of `crates/sdk/src/theme/radix/` to `crates/theme-radix/src/`.
   * Rename `crates/theme-radix/src/mod.rs` to `lib.rs` to serve as the new crate entry point.

2. **Clean up SDK Module Declarations**:
   * In [crates/sdk/src/theme/mod.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/theme/mod.rs), remove the `pub mod radix;` declaration and all corresponding radix exports.

---

## Phase 3: Decouple Core SDK References

To make the crate boundary compile, the core SDK must not reference any radix-specific layout rules or builders.

1. **Move Extension Traits**:
   * Ensure extension traits like `RadixThemeControlExt` and builder modifiers (e.g. `RadixButtonStyleExt`) live entirely inside the new `gpui-luma-theme-radix` crate.
2. **Abstract Core Templates**:
   * Controls inside `crates/sdk/src/controls/` should only require generic look configs (like `ButtonFamilyLook` or `SwitchPalette`). They should have no knowledge of `RadixTheme`.

---

## Phase 4: Clean Up Hidden Coupling leaks

With the Radix theme isolated in its own crate, we can safely address the hidden dependencies on `LumaTheme::native()` inside its resolvers:

1. **`floating_menu.rs` (Shadows)**:
   * Replace `LumaTheme::native().mode(...).elevation.menu` with a lookup from the parsed CSS catalog, or a local `BoxShadow` fallback defined inside `theme-radix`.
2. **`slider.rs` & `switch.rs` (Thumb Shadows)**:
   * Replace `LumaTheme::native().mode(...).elevation.thumb` with a local thumb shadow layer configuration or catalog lookup.
3. **`mode.rs` (Fallback Scaffold)**:
   * Instead of importing `LumaTheme::native()` to initialize default metric values when parsing a new CSS catalog, define static fallback metrics inside `theme-radix`.

---

## Phase 5: Update Downstream Applications

Because `apps/gallery` and `apps/theme-studio` reference radix builders, their configurations must be updated to reference both crates.

1. **Cargo Configuration**:
   * Add `gpui-luma-theme-radix` as a dependency in:
     * `apps/gallery/Cargo.toml`
     * `apps/theme-studio/Cargo.toml`
2. **Namespace Updates**:
   * In the application files (e.g. [apps/theme-studio/src/studio/panels/team.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/theme-studio/src/studio/panels/team.rs)), update imports:
     ```rust
     // Before
     use gpui_luma::theme::radix::*;
     
     // After
     use gpui_luma_theme_radix::*;
     ```

---

## Phase 6: Verification

* **Build**: Run `cargo check --workspace` to ensure all crates compile successfully.
* **Test**: Run `cargo test --workspace` to verify existing behavior is preserved.
