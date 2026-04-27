use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::{ModButton, ModButtonTemplate, default_mod_button_template};
use crate::controls::button_family::{ButtonInteractionState as ButtonState, ButtonKind, ButtonSize};

#[derive(Clone)]
pub struct ModButtonModel {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) kind: ButtonKind,
    pub(crate) size: ButtonSize,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn ModButtonTemplate>,
}

pub struct ModButtonRenderModel {
    pub id: SharedString,
    pub label: SharedString,
    pub kind: ButtonKind,
    pub size: ButtonSize,
    pub state: ButtonState,
}

pub struct ModButtonBuilder {
    pub(crate) model: ModButtonModel,
}

impl ModButtonBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            model: ModButtonModel {
                label: id.clone(),
                id,
                kind: ButtonKind::Standard,
                size: ButtonSize::Md,
                enabled: true,
                template: default_mod_button_template(),
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

    pub fn template(mut self, template: Arc<dyn ModButtonTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<ModButton> {
        cx.new(|cx| ModButton::from_builder(self, cx))
    }
}
