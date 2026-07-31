# SDK Control Contract Review

## Refactor compound selector controls

### Problem

`Autocomplete`, `ComboBox`, and `SearchSelector` resolve fallback looks inside `Render::render` instead of receiving all visual inputs through their models.

Current `Autocomplete` code:

```rust
impl Render for AutocompleteTextBoxControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = crate::theme::ThemeTokens::default();
        let autocomplete_look = DefaultAutocompleteTextBoxTheme::new(tokens.clone()).resolve(self.model.size);
        let look = (self.model.popup_look_provider)(self.model.size);

        // Build popup and render model using those values...
    }
}
```

Current `ComboBox` code:

```rust
let tokens = crate::theme::ThemeTokens::default();
let autocomplete_look = DefaultAutocompleteTextBoxTheme::new(tokens.clone()).resolve(self.model.size);
let look = (self.model.popup_look_provider)(self.model.size);

let textfield_theme = crate::controls::textfield::default_textfield_theme();
let textfield_look = textfield_theme.resolve_look(
    TextFieldVariant::Standard,
    TextFieldState::default(),
    true,
    self.model.size,
    &StandardBoxScale::compute(self.model.size, &textfield_theme.metrics(), 1.0),
);
```

`SearchSelector` has the same pattern:

```rust
let tokens = crate::theme::ThemeTokens::default();
let autocomplete_look = DefaultAutocompleteTextBoxTheme::new(tokens.clone()).resolve(self.model.size);
let look = (self.model.popup_look_provider)(self.model.size);

let textfield_theme = crate::controls::textfield::default_textfield_theme();
let textfield_look = textfield_theme.resolve_look(/* ... */);
```

Affected files:

- `crates/sdk/src/controls/autocomplete/control.rs`
- `crates/sdk/src/controls/combobox/control.rs`
- `crates/sdk/src/controls/search_selector/control.rs`

### Refactor

Add the missing look/theme inputs to each model. The existing popup provider is already in the right location; add equivalent providers for the control chrome and any text-field metrics needed for sizing.

Illustrative model shape:

```rust
pub struct ComboBoxModel {
    // Existing behavior/configuration...
    pub(crate) size: ControlSize,

    pub(crate) autocomplete_theme: Arc<dyn AutocompleteTextBoxTheme>,
    pub(crate) textfield_theme: Arc<dyn TextFieldTheme>,
    pub(crate) popup_look_provider: ComboBoxPopupLookProvider,

    pub(crate) template: Arc<dyn ComboBoxTemplate>,
    pub(crate) items_template: Arc<dyn ComboBoxItemsTemplate>,
    pub(crate) panel_template: Arc<dyn ComboBoxPanelTemplate>,
}
```

Initialize those fields in the builder and expose builder methods consistent with the existing `popup_look_provider(...)` method:

```rust
impl ComboBoxBuilder {
    pub fn autocomplete_theme(mut self, theme: Arc<dyn AutocompleteTextBoxTheme>) -> Self {
        self.model.autocomplete_theme = theme;
        self
    }

    pub fn textfield_theme(mut self, theme: Arc<dyn TextFieldTheme>) -> Self {
        self.model.textfield_theme = theme;
        self
    }

    pub fn popup_look_provider(mut self, provider: ComboBoxPopupLookProvider) -> Self {
        self.model.popup_look_provider = provider;
        self
    }
}
```

The exact API names may differ, but the important change is that a Shadcn look or another look can provide these values when spawning the control.

### Target render flow

Resolve from model-owned providers rather than constructing defaults in `control.rs`:

```rust
impl Render for ComboBoxControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let autocomplete_look = self.model.autocomplete_theme.resolve(self.model.size);
        let textfield_look = self.model.textfield_theme.resolve_look(
            TextFieldVariant::Standard,
            TextFieldState::default(),
            self.model.enabled,
            self.model.size,
            &StandardBoxScale::compute(
                self.model.size,
                &self.model.textfield_theme.metrics(),
                1.0,
            ),
        );
        let popup_look = (self.model.popup_look_provider)(self.model.size);

        let render_model = ComboBoxRenderModel {
            // Existing behavior-derived fields...
            status_color: autocomplete_look.status_color,
            muted_text_color: autocomplete_look.muted_text_color,
            popup_look,
            // Existing fields...
        };

        self.model.template.render(render_model, handlers, window, cx)
    }
}
```

If the text-field template already owns the authoritative text-field theme, expose a metrics/look query through that template instead of adding a second independent `TextFieldTheme`. Do not resolve a separate default text-field theme solely for sizing.

### Acceptance criteria

- `control.rs` for all three controls no longer calls `ThemeTokens::default()` to construct control looks.
- `control.rs` for `ComboBox` and `SearchSelector` no longer calls `default_textfield_theme()` solely for sizing.
- Shadcn look extensions can provide the control, text-field, and popup look inputs through the builder/model.
- Templates continue to receive resolved visual data through their render models.
- Existing default behavior remains available through default providers in the builders.

