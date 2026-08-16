use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::template::template_with_modifier;
use super::{TextArea, TextAreaLook, TextAreaState, TextAreaTemplate, default_textarea_template};
use crate::controls::textarea::{TextAreaTheme, default_textarea_theme};
use crate::controls::icon::IconSource;
use crate::theme::ControlSize;

pub type Validator = Arc<dyn Fn(&str) -> bool + Send + Sync>;
pub type TextAreaLookOverride = Arc<dyn Fn(TextAreaLook) -> TextAreaLook + Send + Sync + 'static>;

#[derive(Clone)]
pub struct TextAreaModel {
    pub(crate) id: SharedString,
    pub(crate) placeholder: SharedString,
    pub(crate) value: SharedString,
    pub(crate) enabled: bool,
    pub(crate) full_width: bool,
    pub(crate) size: ControlSize,
    pub(crate) rows: usize,
    pub(crate) clean_on_escape: bool,
    pub(crate) select_all_on_tab_focus: bool,
    pub(crate) resize_handle_icon: IconSource,
    pub(crate) validator: Option<Validator>,
    pub(crate) look_override: Option<TextAreaLookOverride>,
    pub(crate) template: Arc<dyn TextAreaTemplate>,
    pub(crate) theme: Arc<dyn TextAreaTheme>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextAreaLineMetric {
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub y: f32,
    pub height: f32,
    pub character_offsets: Vec<f32>,
}

pub struct TextAreaRenderModel<'a> {
    pub id: &'a SharedString,
    pub placeholder: &'a SharedString,
    pub value: &'a SharedString,
    pub enabled: bool,
    pub full_width: bool,
    pub size: ControlSize,
    pub rows: usize,
    pub state: TextAreaState,
    pub caret_visible: bool,
    pub vertical_scroll: f32,
    pub line_metrics: Vec<TextAreaLineMetric>,
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
                full_width: false,
                size: ControlSize::Md,
                rows: 4,
                clean_on_escape: false,
                select_all_on_tab_focus: false,
                resize_handle_icon: lucide_icons::Icon::Scaling.into(),
                validator: None,
                look_override: None,
                template: default_textarea_template(),
                theme: default_textarea_theme(),
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

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.model.full_width = full_width;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn rows(mut self, rows: usize) -> Self {
        self.model.rows = rows.max(1);
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

    /// Set the icon displayed by the draggable resize handle.
    pub fn resize_handle_icon(mut self, icon: impl Into<IconSource>) -> Self {
        self.model.resize_handle_icon = icon.into();
        self
    }

    pub fn validator(mut self, validator: Validator) -> Self {
        self.model.validator = Some(validator);
        self
    }

    pub fn look_override<F>(mut self, look_override: F) -> Self
    where
        F: Fn(TextAreaLook) -> TextAreaLook + Send + Sync + 'static,
    {
        self.model.look_override = Some(Arc::new(look_override));
        self
    }

    pub fn template(mut self, template: Arc<dyn TextAreaTemplate>) -> Self {
        if let Some(theme) = template.theme() {
            self.model.theme = theme;
        }
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &TextAreaRenderModel<'_>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = template_with_modifier(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn theme(mut self, theme: Arc<dyn TextAreaTheme>) -> Self {
        self.model.theme = theme;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<TextArea> {
        cx.new(|cx| TextArea::from_builder(self, cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_textarea_template();
        let builder = TextAreaBuilder::new("textarea-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }
}
