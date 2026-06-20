# Issue #0: Rename Control Look to Look

## Description
To align the SDK controls with the Luma workspace's theme-branding terminology (e.g. `look-shadcn`), all constructs, structs, methods, and closures containing `Look` should be renamed to `Look`.

## Rationale
* **Conceptual Consistency:** The SDK acts as a lookless core, and styling engines (e.g. [ShadcnLook](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/look.rs)) resolve visual designs. Resolving a control's visual style yields the "Look" of that control.
* **Ergonomics:** "Look" is a shorter, punchier, and more intuitive term than "Look".

---

## Proposed Refactoring Tasks

### 1. Rename Visual State Structs
Rename all control-specific `Look` structs to `Look`.
* `TextFieldLook` $\rightarrow$ `TextFieldLook`
* `SliderLook` $\rightarrow$ `SliderLook`
* `ProgressLook` $\rightarrow$ `ProgressLook`
* `PagerLook` $\rightarrow$ `PagerLook`
* `ListBoxListLook` $\rightarrow$ `ListBoxListLook`
* `ListBoxRowLook` $\rightarrow$ `ListBoxRowLook`
* *(Apply this renaming consistently to all 20+ control look types)*

### 2. Rename Override Setters and Closures
Rename post-resolution escape hatch methods and fields from `look_override` to `look_override`.
* **Builders:** `.look_override(override_fn)` $\rightarrow$ `.look_override(override_fn)`
* **Controls:** `.set_look_override(override_fn, cx)` $\rightarrow$ `.set_look_override(override_fn, cx)`
* **Model Fields:** `look_override: Option<...>` $\rightarrow$ `look_override: Option<...>`

### 3. Update Look/Theme Resolvers
Rename helper traits and resolution functions.
* `TextFieldTheme::resolve_look` $\rightarrow$ `TextFieldTheme::resolve_look`
* `ThemedTextFieldTemplate::resolve_look_with_scale` $\rightarrow$ `ThemedTextFieldTemplate::resolve_look_with_scale`

---

## Post-Completion Verification & Documentation

> [!IMPORTANT]
> Immediately after completing the code refactoring, **you must update the core guidelines in [docs/ai/control-design.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/ai/control-design.md)** to reflect the new `Look` naming conventions and replace all references to `Look` with `Look`.
