use std::sync::Arc;
use std::sync::OnceLock;

use gpui::{App, Div, FontWeight, Stateful, Window, div, px, prelude::*};

use super::ButtonRenderModel;
use crate::controls::button_family::ButtonKind;
use crate::controls::state::focus_debug_border;
use crate::theme::{ButtonFamilyRole, ButtonFamilyTheme, ButtonVariant, default_button_family_theme};

pub trait ButtonTemplate: Send + Sync {
    fn render(&self, model: &ButtonRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<Div>;
}

pub struct ThemedButtonTemplate {
    theme: Arc<dyn ButtonFamilyTheme>,
}

impl ThemedButtonTemplate {
    pub fn new(theme: Arc<dyn ButtonFamilyTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_button_template() -> Arc<dyn ButtonTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ButtonTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedButtonTemplate::new(default_button_family_theme()))).clone()
}

impl ButtonTemplate for ThemedButtonTemplate {
    fn render(&self, model: &ButtonRenderModel<'_>, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let appearance =
            self.theme.resolve(button_variant(model.kind), ButtonFamilyRole::Text, model.size, model.state);
        let mut root = div()
            .id(model.id.clone())
            .flex()
            .items_center()
            .justify_center()
            .gap(px(appearance.gap))
            .px(px(appearance.padding_x))
            .py(px(appearance.padding_y))
            .h(px(appearance.height))
            .bg(appearance.background)
            .text_color(appearance.foreground)
            .border_1()
            .border_color(appearance.border)
            .rounded(px(appearance.radius))
            .font_weight(FontWeight::MEDIUM)
            .child(model.label.clone());

        if model.state.disabled {
            root = root.opacity(0.56);
        } else {
            root = root.cursor_pointer();
        }

        if model.state.focused {
            root = root.border_1().border_color(appearance.focus_ring.unwrap_or_else(focus_debug_border));
        }

        root
    }
}

fn button_variant(kind: ButtonKind) -> ButtonVariant {
    match kind {
        ButtonKind::Default => ButtonVariant::Default,
        ButtonKind::Primary => ButtonVariant::Primary,
        ButtonKind::Destructive => ButtonVariant::Destructive,
    }
}
