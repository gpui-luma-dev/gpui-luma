use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, Div, FontWeight, Stateful, Window, div, px, svg, prelude::*};

use super::{IconButtonIcon, IconButtonRenderModel};
use crate::controls::button_family::ButtonKind;
use crate::theme::{ButtonFamilyRole, ButtonFamilyTheme, ButtonVariant, default_button_family_theme};

pub trait IconButtonTemplate: Send + Sync {
    fn render(
        &self,
        model: &IconButtonRenderModel<'_>,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedIconButtonTemplate {
    theme: Arc<dyn ButtonFamilyTheme>,
}

impl ThemedIconButtonTemplate {
    pub fn new(theme: Arc<dyn ButtonFamilyTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_icon_button_template() -> Arc<dyn IconButtonTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn IconButtonTemplate>> = OnceLock::new();

    TEMPLATE
        .get_or_init(|| Arc::new(ThemedIconButtonTemplate::new(default_button_family_theme())))
        .clone()
}

impl IconButtonTemplate for ThemedIconButtonTemplate {
    fn render(
        &self,
        model: &IconButtonRenderModel<'_>,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let appearance = self.theme.resolve(
            button_variant(model.kind),
            ButtonFamilyRole::Icon,
            model.size,
            model.state,
        );
        let mut root = div()
            .id(model.id.clone())
            .flex()
            .items_center()
            .justify_center()
            .size(px(appearance.height))
            .bg(appearance.background)
            .text_color(appearance.foreground)
            .border_1()
            .border_color(appearance.border)
            .rounded(px(appearance.radius))
            .font_weight(FontWeight::MEDIUM)
            .cursor_pointer()
            .child(render_icon(model.icon, appearance.foreground));

        if model.state.disabled {
            root = root.opacity(0.56);
        }

        if let Some(focus_ring) = appearance.focus_ring {
            root = root.focus_visible(move |style| style.border_color(focus_ring));
        }

        root
    }
}

fn render_icon(icon: &IconButtonIcon, color: gpui::Hsla) -> AnyElement {
    if let Some(icon) = icon.lucide() {
        div()
            .size(px(16.0))
            .flex()
            .items_center()
            .justify_center()
            .font_family("lucide")
            .font_weight(FontWeight::NORMAL)
            .text_size(px(16.0))
            .line_height(px(16.0))
            .text_color(color)
            .child(char::from(icon).to_string())
            .into_any_element()
    } else if let Some(path) = icon.svg_path() {
        svg()
            .external_path(path.clone())
            .size(px(16.0))
            .text_color(color)
            .into_any_element()
    } else {
        div().size(px(16.0)).into_any_element()
    }
}

fn button_variant(kind: ButtonKind) -> ButtonVariant {
    match kind {
        ButtonKind::Default => ButtonVariant::Default,
        ButtonKind::Primary => ButtonVariant::Primary,
        ButtonKind::Destructive => ButtonVariant::Destructive,
    }
}
