use std::sync::Arc;

use gpui::{App, Div, FontWeight, Stateful, Window, div, px, prelude::*};

use super::ButtonRenderModel;
use crate::theme::button::ButtonTheme;

pub trait ButtonTemplate: Send + Sync {
    fn render(
        &self,
        model: &ButtonRenderModel<'_>,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedButtonTemplate {
    theme: Arc<dyn ButtonTheme>,
}

impl ThemedButtonTemplate {
    pub fn new(theme: Arc<dyn ButtonTheme>) -> Self {
        Self { theme }
    }
}

impl ButtonTemplate for ThemedButtonTemplate {
    fn render(
        &self,
        model: &ButtonRenderModel<'_>,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let appearance = self.theme.resolve(model.kind, model.size, model.state);
        let mut root = div()
            .id(model.id.clone())
            .flex()
            .items_center()
            .justify_center()
            .gap(px(appearance.gap))
            .px(px(appearance.padding_x))
            .py(px(appearance.padding_y))
            .bg(appearance.background)
            .text_color(appearance.foreground)
            .border_1()
            .border_color(appearance.border)
            .rounded(px(appearance.radius))
            .font_weight(FontWeight::MEDIUM)
            .cursor_pointer()
            .child(model.label.clone());

        if model.state.disabled {
            root = root.opacity(0.56);
        }

        if let Some(focus_ring) = appearance.focus_ring {
            root = root.focus_visible(move |style| style.border_color(focus_ring));
        }

        root
    }
}
