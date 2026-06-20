use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::{TextFieldLook, TextFieldState, TextFieldTemplate, TextFieldVariant, default_textfield_template};
use super::control::TextFieldControl;
use crate::controls::command::button::ControlIcon;

pub type Validator = Arc<dyn Fn(&str) -> bool + Send + Sync>;
pub type TextFieldLookOverride = Arc<dyn Fn(TextFieldLook) -> TextFieldLook + Send + Sync + 'static>;

#[derive(Clone)]
pub struct TextFieldModel {
    pub(crate) id: SharedString,
    pub(crate) placeholder: SharedString,
    pub(crate) value: SharedString,
    pub(crate) prefix_icon: Option<ControlIcon>,
    pub(crate) variant: TextFieldVariant,
    pub(crate) enabled: bool,
    pub(crate) full_width: bool,
    pub(crate) clean_on_escape: bool,
    pub(crate) select_all_on_tab_focus: bool,
    pub(crate) propagate_home_end_to_parent: bool,
    pub(crate) validator: Option<Validator>,
    pub(crate) look_override: Option<TextFieldLookOverride>,
    pub(crate) template: Arc<dyn TextFieldTemplate>,
}

pub struct TextFieldRenderModel<'a> {
    pub id: &'a SharedString,
    pub placeholder: &'a SharedString,
    pub value: &'a SharedString,
    pub prefix_icon: Option<&'a ControlIcon>,
    pub variant: TextFieldVariant,
    pub enabled: bool,
    pub full_width: bool,
    pub state: TextFieldState,
    pub caret_visible: bool,
    pub horizontal_scroll: f32,
    pub character_offsets: Vec<f32>,
    pub look: TextFieldLook,
}

pub struct TextFieldBuilder {
    pub(crate) model: TextFieldModel,
}

impl TextFieldBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: TextFieldModel {
                id: id.into(),
                placeholder: SharedString::default(),
                value: SharedString::default(),
                prefix_icon: None,
                variant: TextFieldVariant::Standard,
                enabled: true,
                full_width: false,
                clean_on_escape: false,
                select_all_on_tab_focus: false,
                propagate_home_end_to_parent: false,
                validator: None,
                look_override: None,
                template: default_textfield_template(),
            },
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.model.placeholder = placeholder.into();
        self
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.model.value = value.into();
        self
    }

    pub fn prefix_icon(mut self, icon: impl Into<ControlIcon>) -> Self {
        self.model.prefix_icon = Some(icon.into());
        self
    }

    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
        self.model.variant = variant;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.model.full_width = full_width;
        self
    }

    pub fn clean_on_escape(mut self, clean: bool) -> Self {
        self.model.clean_on_escape = clean;
        self
    }

    pub fn select_all_on_tab_focus(mut self, select_all: bool) -> Self {
        self.model.select_all_on_tab_focus = select_all;
        self
    }

    pub fn propagate_home_end_to_parent(mut self, propagate: bool) -> Self {
        self.model.propagate_home_end_to_parent = propagate;
        self
    }

    pub fn validator(mut self, validator: Validator) -> Self {
        self.model.validator = Some(validator);
        self
    }

    pub fn look_override<F>(mut self, look_override: F) -> Self
    where
        F: Fn(TextFieldLook) -> TextFieldLook + Send + Sync + 'static,
    {
        self.model.look_override = Some(Arc::new(look_override));
        self
    }

    pub fn template(mut self, template: Arc<dyn TextFieldTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<TextFieldControl> {
        cx.new(|cx| TextFieldControl::from_builder(self, cx))
    }
}
