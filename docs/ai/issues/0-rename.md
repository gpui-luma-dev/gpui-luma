# Issue #0: Rename Control Appearance to Look

## Description
To align the SDK controls with the Luma workspace's theme-branding terminology (e.g. `look-shadcn`), all constructs, structs, methods, and closures containing `Appearance` should be renamed to `Look`.

## Rationale
* **Conceptual Consistency:** The SDK acts as a lookless core, and styling engines (e.g. [ShadcnLook](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/look.rs)) resolve visual designs. Resolving a control's visual style yields the "Look" of that control.
* **Ergonomics:** "Look" is a shorter, punchier, and more intuitive term than "Appearance".

---

## Proposed Refactoring Tasks

### 1. Rename Visual State Structs
Rename all control-specific `Appearance` structs to `Look`.
* `TextFieldAppearance` $\rightarrow$ `TextFieldLook`
* `SliderAppearance` $\rightarrow$ `SliderLook`
* `ProgressAppearance` $\rightarrow$ `ProgressLook`
* `PagerAppearance` $\rightarrow$ `PagerLook`
* `ListBoxListAppearance` $\rightarrow$ `ListBoxListLook`
* `ListBoxRowAppearance` $\rightarrow$ `ListBoxRowLook`
* *(Apply this renaming consistently to all 20+ control appearance types)*

### 2. Rename Override Setters and Closures
Rename post-resolution escape hatch methods and fields from `appearance_override` to `look_override`.
* **Builders:** `.appearance_override(override_fn)` $\rightarrow$ `.look_override(override_fn)`
* **Controls:** `.set_appearance_override(override_fn, cx)` $\rightarrow$ `.set_look_override(override_fn, cx)`
* **Model Fields:** `appearance_override: Option<...>` $\rightarrow$ `look_override: Option<...>`

### 3. Update Look/Theme Resolvers
Rename helper traits and resolution functions.
* `TextFieldTheme::resolve_appearance` $\rightarrow$ `TextFieldTheme::resolve_look`
* `ThemedTextFieldTemplate::resolve_appearance_with_scale` $\rightarrow$ `ThemedTextFieldTemplate::resolve_look_with_scale`

---

## Post-Completion Verification & Documentation

> [!IMPORTANT]
> Immediately after completing the code refactoring, **you must update the core guidelines in [docs/ai/control-design.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/ai/control-design.md)** to reflect the new `Look` naming conventions and replace all references to `Appearance` with `Look`.
