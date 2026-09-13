//! Look-owned text-field builder. Spawn synthesizes the SDK [`luma::controls::textfield::TextField`].

use std::sync::Arc;

use gpui::{App, Context, Div, SharedString, Stateful};
use luma::controls::button::ControlIcon;
use luma::controls::textfield::{TextFieldBuilder, TextFieldLook, TextFieldLookOverride, TextFieldRenderModel, Validator};

use crate::look::{Look, resolve_look};
use crate::textfield::{TextFieldSize, TextFieldVariant, textfield_template};

type TextFieldModifier = Box<dyn Fn(Stateful<Div>, &TextFieldRenderModel<'_>) -> Stateful<Div> + Send + Sync>;

/// Builder in the guise of a text field: Radix axes plus SDK options, until `.spawn(cx)`.
pub struct TextField {
    id: SharedString,
    look: Option<Look>,
    variant: TextFieldVariant,
    size: TextFieldSize,
    placeholder: SharedString,
    value: SharedString,
    prefix_icon: Option<ControlIcon>,
    enabled: bool,
    tab_stop: bool,
    compact: bool,
    full_width: bool,
    clean_on_escape: bool,
    select_all_on_tab_focus: bool,
    propagate_home_end_to_parent: bool,
    validator: Option<Validator>,
    look_override: Option<TextFieldLookOverride>,
    modifiers: Vec<TextFieldModifier>,
}

impl TextField {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            look: None,
            variant: TextFieldVariant::default(),
            size: TextFieldSize::default(),
            placeholder: SharedString::default(),
            value: SharedString::default(),
            prefix_icon: None,
            enabled: true,
            tab_stop: true,
            compact: false,
            full_width: false,
            clean_on_escape: false,
            select_all_on_tab_focus: false,
            propagate_home_end_to_parent: false,
            validator: None,
            look_override: None,
            modifiers: Vec::new(),
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn classic(self) -> Self {
        self.variant(TextFieldVariant::Classic)
    }

    pub fn surface(self) -> Self {
        self.variant(TextFieldVariant::Surface)
    }

    pub fn soft(self) -> Self {
        self.variant(TextFieldVariant::Soft)
    }

    pub fn size(mut self, size: TextFieldSize) -> Self {
        self.size = size;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into();
        self
    }

    pub fn prefix_icon(mut self, icon: impl Into<ControlIcon>) -> Self {
        self.prefix_icon = Some(icon.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.tab_stop = tab_stop;
        self
    }

    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    pub fn clean_on_escape(mut self, clean: bool) -> Self {
        self.clean_on_escape = clean;
        self
    }

    pub fn select_all_on_tab_focus(mut self, select_all: bool) -> Self {
        self.select_all_on_tab_focus = select_all;
        self
    }

    pub fn propagate_home_end_to_parent(mut self, propagate: bool) -> Self {
        self.propagate_home_end_to_parent = propagate;
        self
    }

    pub fn validator(mut self, validator: Validator) -> Self {
        self.validator = Some(validator);
        self
    }

    pub fn look_override<F>(mut self, look_override: F) -> Self
    where
        F: Fn(TextFieldLook) -> TextFieldLook + Send + Sync + 'static,
    {
        self.look_override = Some(Arc::new(look_override));
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &TextFieldRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> luma::controls::textfield::TextField {
        let look = self.resolve_look(cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn resolve_look(&self, cx: &App) -> Look {
        resolve_look(self.look.as_ref(), cx.try_global::<Look>())
    }

    fn into_sdk_builder(self, look: Look) -> TextFieldBuilder {
        let template = textfield_template(&look, self.variant);
        let mut builder = luma::controls::textfield::new(self.id)
            .template(template)
            .placeholder(self.placeholder)
            .value(self.value)
            .enabled(self.enabled)
            .tab_stop(self.tab_stop)
            .size(self.size.control_size())
            .full_width(self.full_width)
            .clean_on_escape(self.clean_on_escape)
            .select_all_on_tab_focus(self.select_all_on_tab_focus)
            .propagate_home_end_to_parent(self.propagate_home_end_to_parent);
        if let Some(icon) = self.prefix_icon {
            builder = builder.prefix_icon(icon);
        }
        if self.compact {
            builder = builder.compact();
        }
        if let Some(validator) = self.validator {
            builder = builder.validator(validator);
        }
        if let Some(look_override) = self.look_override {
            builder = builder.look_override(move |look| (look_override)(look));
        }
        for modifier in self.modifiers {
            builder = builder.with_template_modifier(move |root, model| (modifier)(root, model));
        }
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luma::theme::ControlSize;

    #[test]
    fn size_maps_to_sdk_control_size() {
        assert_eq!(TextField::new("s").size(TextFieldSize::One).size.control_size(), ControlSize::Sm);
        assert_eq!(TextField::new("s").size(TextFieldSize::Two).size.control_size(), ControlSize::Md);
        assert_eq!(TextField::new("s").size(TextFieldSize::Three).size.control_size(), ControlSize::Lg);
    }

    #[test]
    fn value_keeps_radix_axes() {
        let field = TextField::new("name").soft().size(TextFieldSize::Three).placeholder("Name").value("Ada");
        assert!(matches!(field.variant, TextFieldVariant::Soft));
        assert_eq!(field.size, TextFieldSize::Three);
        assert_eq!(field.placeholder.as_ref(), "Name");
        assert_eq!(field.value.as_ref(), "Ada");
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = Look::built_in();
        let _builder = TextField::new("ok")
            .look(&look)
            .classic()
            .placeholder("Search…")
            .with_template_modifier(|root, _| root)
            .into_sdk_builder(look);
    }
}
