use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, Div, FontWeight, Stateful, Window, div, px, svg, prelude::*};

use super::{IconButtonIcon, IconButtonRenderModel};
use crate::controls::button_family::ButtonKind;
use crate::controls::button_family_template::render_button_family_focus_ring;
use crate::theme::{ButtonFamilyRole, ButtonFamilyTheme, ButtonVariant, default_button_family_theme};

const DISABLED_OPACITY: f32 = 0.56;
const ICON_SIZE: f32 = 16.0;

pub trait IconButtonTemplate: Send + Sync {
    fn render(&self, model: &IconButtonRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<Div>;
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
    fn render(&self, model: &IconButtonRenderModel<'_>, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let appearance =
            self.theme.resolve(button_variant(model.kind), ButtonFamilyRole::Icon, model.size, model.state);
        let control = div()
            .flex()
            .items_center()
            .justify_center()
            .size(px(appearance.height))
            .bg(appearance.background)
            .text_color(appearance.foreground)
            .border_1()
            .border_color(appearance.border)
            .rounded(px(appearance.radius))
            .text_size(px(appearance.typography.size))
            .line_height(px(appearance.typography.line_height))
            .font_weight(appearance.typography.weight)
            .child(render_icon(model.icon, appearance.foreground));

        let mut root =
            render_button_family_focus_ring(model.id.clone(), control, appearance.focus_ring, appearance.radius);

        if model.state.disabled {
            root = root.opacity(DISABLED_OPACITY);
        } else {
            root = root.cursor_pointer();
        }

        root
    }
}

fn render_icon(icon: &IconButtonIcon, color: gpui::Hsla) -> AnyElement {
    if let Some(icon) = icon.lucide() {
        div()
            .size(px(ICON_SIZE))
            .flex()
            .items_center()
            .justify_center()
            .font_family("lucide")
            .font_weight(FontWeight::NORMAL)
            .text_size(px(ICON_SIZE))
            .line_height(px(ICON_SIZE))
            .text_color(color)
            .child(char::from(icon).to_string())
            .into_any_element()
    } else if let Some(path) = icon.svg_path() {
        svg().external_path(path.clone()).size(px(ICON_SIZE)).text_color(color).into_any_element()
    } else {
        div().size(px(ICON_SIZE)).into_any_element()
    }
}

fn button_variant(kind: ButtonKind) -> ButtonVariant {
    match kind {
        ButtonKind::Default => ButtonVariant::Default,
        ButtonKind::Primary => ButtonVariant::Primary,
        ButtonKind::Destructive => ButtonVariant::Destructive,
    }
}
