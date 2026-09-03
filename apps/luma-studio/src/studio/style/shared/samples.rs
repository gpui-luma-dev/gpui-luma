use std::sync::Arc;

use gpui::{SharedString, div, prelude::*};
use luma::controls::command::button::ButtonRenderModel;
use luma::theme::InteractionState;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::style::shared::icons::render_lucide_icon;

#[derive(Clone, Copy)]
pub(crate) struct ButtonStateSample {
    pub id: &'static str,
    pub header: &'static str,
    pub state: InteractionState,
}

#[derive(Clone, Copy)]
pub(crate) enum ButtonTemplateVariant {
    TextButton,
    IconButton,
}

pub(crate) fn button_preview_look(
    model: &ButtonRenderModel<()>,
) -> Option<luma::controls::button_family::ButtonFamilyLook> {
    model.look.as_ref().map(|resolve| resolve(model))
}

impl ButtonTemplateVariant {
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::TextButton => "text-button",
            Self::IconButton => "icon-button",
        }
    }

    pub(crate) fn round(self) -> bool {
        matches!(self, Self::IconButton)
    }

    pub(crate) fn content(self) -> luma::controls::command::button::ControlPresenter<ButtonRenderModel<()>> {
        let label = SharedString::from("Button");
        match self {
            Self::TextButton => Arc::new(move |_, _| div().child(label.clone()).into_any_element()),
            Self::IconButton => Arc::new(move |model, _| {
                let icon_size = button_preview_look(model).map(|look| look.icon_size).unwrap_or(16.0);
                render_lucide_icon(
                    LucideIcon::Heart,
                    button_preview_look(model).map(|look| look.foreground).unwrap_or_default(),
                    icon_size,
                )
            }),
        }
    }
}
