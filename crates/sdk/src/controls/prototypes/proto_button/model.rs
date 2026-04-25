use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::{ProtoButton, ProtoButtonTemplate, default_proto_button_template};
use crate::controls::button_family::ButtonInteractionState as ProtoButtonState;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProtoButtonSize {
    Sm,
    #[default]
    Md,
    Lg,
}

#[derive(Clone)]
pub struct ProtoButtonModel {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) size: ProtoButtonSize,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn ProtoButtonTemplate>,
}

pub struct ProtoButtonRenderModel<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub size: ProtoButtonSize,
    pub state: ProtoButtonState,
}

pub struct ProtoButtonBuilder {
    pub(crate) model: ProtoButtonModel,
}

impl ProtoButtonBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            model: ProtoButtonModel {
                label: id.clone(),
                id,
                size: ProtoButtonSize::Md,
                enabled: true,
                template: default_proto_button_template(),
            },
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.label = label.into();
        self
    }

    pub fn size(mut self, size: ProtoButtonSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn ProtoButtonTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<ProtoButton> {
        cx.new(|cx| ProtoButton::from_builder(self, cx))
    }
}
