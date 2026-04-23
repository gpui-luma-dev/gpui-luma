use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::{TextArea, TextAreaState, TextAreaTemplate, default_textarea_template};

pub type Validator = Arc<dyn Fn(&str) -> bool + Send + Sync>;

#[derive(Clone)]
pub struct TextAreaModel {
    pub(crate) id: SharedString,
    pub(crate) placeholder: SharedString,
    pub(crate) value: SharedString,
    pub(crate) enabled: bool,
    pub(crate) clean_on_escape: bool,
    pub(crate) rows: usize,
    pub(crate) validator: Option<Validator>,
    pub(crate) template: Arc<dyn TextAreaTemplate>,
}

pub struct TextAreaRenderModel<'a> {
    pub id: &'a SharedString,
    pub placeholder: &'a SharedString,
    pub value: &'a SharedString,
    pub enabled: bool,
    pub rows: usize,
    pub state: TextAreaState,
}

pub struct TextAreaBuilder {
    pub(crate) model: TextAreaModel,
}

impl TextAreaBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: TextAreaModel {
                id: id.into(),
                placeholder: SharedString::default(),
                value: SharedString::default(),
                enabled: true,
                clean_on_escape: false,
                rows: 4,
                validator: None,
                template: default_textarea_template(),
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

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn clean_on_escape(mut self, clean: bool) -> Self {
        self.model.clean_on_escape = clean;
        self
    }

    pub fn rows(mut self, rows: usize) -> Self {
        self.model.rows = rows.max(2);
        self
    }

    pub fn validator(mut self, validator: Validator) -> Self {
        self.model.validator = Some(validator);
        self
    }

    pub fn template(mut self, template: Arc<dyn TextAreaTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<TextArea> {
        cx.new(|cx| TextArea::from_builder(self, cx))
    }
}
