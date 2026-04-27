use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::{ToggleButton, ToggleButtonTemplate, default_toggle_button_template};
use crate::controls::button_family::{
    ButtonInteractionState as ToggleButtonState, ButtonKind as ToggleButtonKind, ButtonSize as ToggleButtonSize,
};

#[derive(Clone)]
pub struct ToggleButtonModel {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) kind: ToggleButtonKind,
    pub(crate) size: ToggleButtonSize,
    pub(crate) enabled: bool,
    pub(crate) selected: bool,
    pub(crate) template: Arc<dyn ToggleButtonTemplate>,
}

pub struct ToggleButtonRenderModel<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub kind: ToggleButtonKind,
    pub size: ToggleButtonSize,
    pub enabled: bool,
    pub selected: bool,
    pub state: ToggleButtonState,
}

pub struct ToggleButtonBuilder {
    pub(crate) model: ToggleButtonModel,
}

impl ToggleButtonBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            model: ToggleButtonModel {
                label: id.clone(),
                id,
                kind: ToggleButtonKind::Standard,
                size: ToggleButtonSize::Md,
                enabled: true,
                selected: false,
                template: default_toggle_button_template(),
            },
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.label = label.into();
        self
    }

    pub fn kind(mut self, kind: ToggleButtonKind) -> Self {
        self.model.kind = kind;
        self
    }

    pub fn size(mut self, size: ToggleButtonSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.model.selected = selected;
        self
    }

    pub fn template(mut self, template: Arc<dyn ToggleButtonTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<ToggleButton> {
        cx.new(|cx| ToggleButton::from_builder(self, cx))
    }
}
