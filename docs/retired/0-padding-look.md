# Issue #0-look: Internal Compact Styling via Look Adjustments

## Description
Instead of introducing new nested parameter structs (`TextFieldTemplateParameters`) or exposing complex sizing overrides directly on the public builder, we can implement compact styling using a simple boolean flag on the model that triggers an internal mutation of the resolved `TextFieldLook`.

This achieves the desired compact styling (e.g. shaving padding and height for sidebar token lists) with zero public API overhead and leverages the existing `look_override` flow.

---

## Rationale
* **API Simplicity:** Downstream users simply call `.compact()` on the builder.
* **No Extra Abstractions:** Avoids creating new parameter structs, mapping templates, or bloating models with multiple public override properties.
* **Compatibility with Custom Overrides:** Applying the compact styling adjustments internally inside `resolved_look` ensures that any user-supplied `.look_override(...)` remains active and is applied on top of the shrunken dimensions.

---

## Proposed Architecture

### 1. Update `TextFieldModel` & `TextFieldBuilder`
Add the `compact` flag to the model and expose the builder method in `crates/sdk/src/controls/textfield/model.rs`:

```rust
pub struct TextFieldModel {
    ...
    pub(crate) compact: bool,
}

impl TextFieldBuilder {
    // In new() initialization:
    // compact: false,
    
    pub fn compact(mut self) -> Self {
        self.model.compact = true;
        self
    }
}
```

### 2. Update `resolved_look` in `TextFieldControl`
Update `resolved_look` in `crates/sdk/src/controls/textfield/control.rs` to apply the compact offsets to the resolved `TextFieldLook` before applying any user-provided `look_override` closures:

```rust
fn resolved_look(&self, window: &Window, cx: &mut Context<Self>) -> TextFieldLook {
    let mut look = self.model.template.resolve_look_with_scale(
        self.model.variant,
        self.state,
        self.model.enabled,
        window.scale_factor(),
        cx,
    );

    // Apply internal compact overrides
    if self.model.compact {
        look.padding_y = 2.0;
        look.min_height = 22.0;
    }

    // Apply custom user overrides on top
    if let Some(override_fn) = &self.model.look_override {
        override_fn(look)
    } else {
        look
    }
}
```

---

## Tasks

- [ ] Add `compact: bool` to `TextFieldModel` (defaulting to `false` in `TextFieldBuilder::new`).
- [ ] Implement the `.compact()` method on `TextFieldBuilder`.
- [ ] Update `TextFieldControl::resolved_look` to modify the resolved `look` with compact bounds (`padding_y = 2.0`, `min_height = 22.0`) if `compact` is active.
