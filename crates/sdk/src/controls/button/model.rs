use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::{Button, ButtonTemplate, default_button_template};
use crate::controls::button_family::{ButtonInteractionState as ButtonState, ButtonKind, ButtonSize};

#[derive(Clone)]
pub struct ButtonModel {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) kind: ButtonKind,
    pub(crate) size: ButtonSize,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn ButtonTemplate>,
}

pub struct ButtonRenderModel<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub kind: ButtonKind,
    pub size: ButtonSize,
    pub state: ButtonState,
}

pub struct ButtonBuilder {
    pub(crate) model: ButtonModel,
}

impl ButtonBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            model: ButtonModel {
                label: id.clone(),
                id,
                kind: ButtonKind::Default,
                size: ButtonSize::Md,
                enabled: true,
                template: default_button_template(),
            },
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.label = label.into();
        self
    }

    pub fn kind(mut self, kind: ButtonKind) -> Self {
        self.model.kind = kind;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn ButtonTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<Button> {
        cx.new(|cx| Button::from_builder(self, cx))
    }
}
