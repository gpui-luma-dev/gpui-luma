use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::{IconButton, IconButtonIcon, IconButtonTemplate, default_icon_button_template};
use crate::controls::button_family::{
    ButtonInteractionState as IconButtonState, ButtonKind as IconButtonKind, ButtonSize as IconButtonSize,
};

#[derive(Clone)]
pub struct IconButtonModel {
    pub(crate) id: SharedString,
    pub(crate) icon: IconButtonIcon,
    pub(crate) kind: IconButtonKind,
    pub(crate) size: IconButtonSize,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn IconButtonTemplate>,
}

pub struct IconButtonRenderModel<'a> {
    pub id: &'a SharedString,
    pub icon: &'a IconButtonIcon,
    pub kind: IconButtonKind,
    pub size: IconButtonSize,
    pub state: IconButtonState,
}

pub struct IconButtonBuilder {
    pub(crate) model: IconButtonModel,
}

impl IconButtonBuilder {
    pub fn new(id: impl Into<SharedString>, icon: impl Into<IconButtonIcon>) -> Self {
        let id = id.into();

        Self {
            model: IconButtonModel {
                icon: icon.into(),
                id,
                kind: IconButtonKind::Default,
                size: IconButtonSize::Md,
                enabled: true,
                template: default_icon_button_template(),
            },
        }
    }

    pub fn icon(mut self, icon: impl Into<IconButtonIcon>) -> Self {
        self.model.icon = icon.into();
        self
    }

    pub fn kind(mut self, kind: IconButtonKind) -> Self {
        self.model.kind = kind;
        self
    }

    pub fn size(mut self, size: IconButtonSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn IconButtonTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<IconButton> {
        cx.new(|cx| IconButton::from_builder(self, cx))
    }
}
