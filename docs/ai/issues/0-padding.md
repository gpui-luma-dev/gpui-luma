# Issue #0: Compact Text Field Sizing via Template Parameters

## Description
Instead of using custom closures/hacks via `appearance_override` which bypass the model-driven design of the SDK controls, we should implement a structured **`TextFieldTemplateParameters`** configuration pattern. This is consistent with other SDK controls (such as `PagerTemplateParameters`) and preserves theme-aware scale caching.

---

## Proposed Architecture

### 1. Define `TextFieldTemplateParameters`
Add the parameters struct in `crates/sdk/src/controls/textfield/model.rs`:

```rust
#[derive(Clone, Debug)]
pub struct TextFieldTemplateParameters {
    pub size: ControlSize,
    pub padding_y_override: Option<f32>,
    pub min_height_override: Option<f32>,
}

impl Default for TextFieldTemplateParameters {
    fn default() -> Self {
        Self {
            size: ControlSize::Md,
            padding_y_override: None,
            min_height_override: None,
        }
    }
}

impl TextFieldTemplateParameters {
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn padding_y_override(mut self, padding_y: f32) -> Self {
        self.padding_y_override = Some(padding_y);
        self
    }

    pub fn min_height_override(mut self, min_height: f32) -> Self {
        self.min_height_override = Some(min_height);
        self
    }
}
```

---

### 2. Update `TextFieldModel` & `TextFieldRenderModel`
Add the parameters field to the model structs in `crates/sdk/src/controls/textfield/model.rs`:

```rust
pub struct TextFieldModel {
    ...
    pub(crate) template_parameters: TextFieldTemplateParameters,
}

pub struct TextFieldRenderModel<'a> {
    ...
    pub template_parameters: &'a TextFieldTemplateParameters,
}
```

---

### 3. Add Builder and Control Methods
Add chainable builder methods to `TextFieldBuilder` in `crates/sdk/src/controls/textfield/model.rs`:

```rust
impl TextFieldBuilder {
    pub fn template_parameters(mut self, params: TextFieldTemplateParameters) -> Self {
        self.model.template_parameters = params;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.template_parameters.size = size;
        self
    }
}
```

And a setter on `TextFieldControl` in `crates/sdk/src/controls/textfield/control.rs`:

```rust
impl TextFieldControl {
    pub fn set_template_parameters(&mut self, params: TextFieldTemplateParameters, cx: &mut Context<Self>) {
        self.model.template_parameters = params;
        self.layout_cache = None;
        cx.notify();
    }
}
```

---

### 4. Integrate with `TextFieldTemplate` Resolution
Update `TextFieldTemplate::resolve_appearance_with_scale` in `crates/sdk/src/controls/textfield/template.rs` to consume these parameters:

```rust
fn resolve_appearance_with_scale(
    &self,
    variant: TextFieldVariant,
    state: TextFieldState,
    enabled: bool,
    params: &TextFieldTemplateParameters,
    scale_factor: f32,
    cx: &mut App,
) -> TextFieldAppearance {
    // Resolve the Cached standard box scale using params.size instead of hardcoded ControlSize::Md
    let scale = cx.use_cached_layout(
        self.theme.metrics(),
        LayoutCacheKey { size: params.size, scale_factor_bits: scale_factor.to_bits() },
        |metrics| StandardBoxScale::compute(params.size, metrics, scale_factor),
    );
    
    let mut appearance = self.theme.resolve_appearance(variant, state, enabled, &scale);
    
    // Apply optional visual overrides
    if let Some(py) = params.padding_y_override {
        appearance.padding_y = py;
    }
    if let Some(h) = params.min_height_override {
        appearance.min_height = h;
    }
    
    appearance
}
```

---

## Sidebar Usage Example

Once the template parameter system is in place, you can configure compact styles cleanly without closures or traits:

```rust
// Create template parameters for a compact token input
let compact_params = TextFieldTemplateParameters::default()
    .size(ControlSize::Sm)
    .padding_y_override(2.0)
    .min_height_override(22.0);

// Builder chain
let field = look
    .textfield("compact-field")
    .template_parameters(compact_params)
    .spawn(cx);
```
