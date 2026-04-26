use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::{default_toggle_template, Toggle, ToggleTemplate};
use crate::controls::button_family::{
    ButtonInteractionState as ToggleState, ButtonKind as ToggleKind, ButtonSize as ToggleSize,
};

#[derive(Clone)]
pub struct ToggleModel {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) kind: ToggleKind,
    pub(crate) size: ToggleSize,
    pub(crate) enabled: bool,
    pub(crate) selected: bool,
    pub(crate) template: Arc<dyn ToggleTemplate>,
}

pub struct ToggleRenderModel<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub kind: ToggleKind,
    pub size: ToggleSize,
    pub enabled: bool,
    pub selected: bool,
    pub state: ToggleState,
}

pub struct ToggleBuilder {
    pub(crate) model: ToggleModel,
}

impl ToggleBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            model: ToggleModel {
                label: id.clone(),
                id,
                kind: ToggleKind::Default,
                size: ToggleSize::Md,
                enabled: true,
                selected: false,
                template: default_toggle_template(),
            },
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.label = label.into();
        self
    }

    pub fn kind(mut self, kind: ToggleKind) -> Self {
        self.model.kind = kind;
        self
    }

    pub fn size(mut self, size: ToggleSize) -> Self {
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

    pub fn template(mut self, template: Arc<dyn ToggleTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<Toggle> {
        cx.new(|cx| Toggle::from_builder(self, cx))
    }
}

// Backward-compatible aliases while callers migrate from ToggleButton naming.
pub type ToggleButtonModel = ToggleModel;
pub type ToggleButtonRenderModel<'a> = ToggleRenderModel<'a>;
pub type ToggleButtonBuilder = ToggleBuilder;
