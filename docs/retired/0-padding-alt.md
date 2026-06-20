# Issue #0-alt: Flat Builder Configuration for Text Field Padding/Sizing

## Description
The proposed design in `0-padding.md` introduces a nested `TextFieldTemplateParameters` struct to wrap sizing and visual offsets. While clean, this requires downstream callers to instantiate and chain a separate struct, then pass it back to the text field:

```rust
// Original proposal (nested):
let compact_params = TextFieldTemplateParameters::default()
    .size(ControlSize::Sm)
    .padding_y_override(2.0);

let field = look.textfield("field")
    .template_parameters(compact_params)
    .spawn(cx);
```

As an alternative, we can flatten this configuration directly onto the `TextFieldModel` and `TextFieldBuilder`. This provides the exact same cache-aware look resolution without the extra abstraction layer of a secondary parameter struct.

```rust
// Alternative proposal (flat builder):
let field = look.textfield("field")
    .size(ControlSize::Sm)
    .padding_y_override(2.0)
    .min_height_override(22.0)
    .spawn(cx);
```

---

## Proposed Architecture

### 1. Update `TextFieldModel` & `TextFieldRenderModel`
Add the configuration and override fields directly to the models in `crates/sdk/src/controls/textfield/model.rs`:

```rust
pub struct TextFieldModel {
    ...
    pub(crate) size: ControlSize,
    pub(crate) padding_y_override: Option<f32>,
    pub(crate) min_height_override: Option<f32>,
}

pub struct TextFieldRenderModel<'a> {
    ...
    pub size: ControlSize,
    pub padding_y_override: Option<f32>,
    pub min_height_override: Option<f32>,
}
```

### 2. Update `TextFieldBuilder`
Expose the configuration methods directly on the builder:

```rust
impl TextFieldBuilder {
    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn compact(self) -> Self {
        self.size(ControlSize::Sm)
    }

    pub fn padding_y_override(mut self, padding_y: f32) -> Self {
        self.model.padding_y_override = Some(padding_y);
        self
    }

    pub fn min_height_override(mut self, min_height: f32) -> Self {
        self.model.min_height_override = Some(min_height);
        self
    }
}
```

### 3. Integrate with `TextFieldTemplate` Resolution
Update `TextFieldTemplate::resolve_look_with_scale` in `crates/sdk/src/controls/textfield/template.rs` to consume these fields:

```rust
fn resolve_look_with_scale(
    &self,
    variant: TextFieldVariant,
    state: TextFieldState,
    enabled: bool,
    size: ControlSize,
    padding_y_override: Option<f32>,
    min_height_override: Option<f32>,
    scale_factor: f32,
    cx: &mut App,
) -> TextFieldLook {
    let scale = cx.use_cached_layout(
        self.theme.metrics(),
        LayoutCacheKey { size, scale_factor_bits: scale_factor.to_bits() },
        |metrics| StandardBoxScale::compute(size, metrics, scale_factor),
    );
    
    let mut look = self.theme.resolve_look(variant, state, enabled, &scale);
    
    if let Some(py) = padding_y_override {
        look.padding_y = py;
    }
    if let Some(h) = min_height_override {
        look.min_height = h;
    }
    
    look
}
```

---

## Tasks

- [ ] Add `size`, `padding_y_override`, and `min_height_override` fields directly to `TextFieldModel`.
- [ ] Implement direct builder methods on `TextFieldBuilder` (`.size()`, `.compact()`, `.padding_y_override()`, `.min_height_override()`).
- [ ] Expose these fields in `TextFieldRenderModel`.
- [ ] Refactor `TextFieldTemplate` and `ThemedTextFieldTemplate` to apply the overrides directly during `resolve_look_with_scale`.
