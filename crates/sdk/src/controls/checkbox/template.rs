use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, Div, FontWeight, Stateful, Window, div, px, prelude::*};
use lucide_icons::Icon as LucideIcon;

use super::CheckboxRenderModel;
use crate::controls::state::focus_debug_border;
use crate::theme::{CheckboxTheme, default_checkbox_theme};

pub trait CheckboxTemplate: Send + Sync {
    fn render(&self, model: &CheckboxRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<Div>;
}

pub struct ThemedCheckboxTemplate {
    theme: Arc<dyn CheckboxTheme>,
}

impl ThemedCheckboxTemplate {
    pub fn new(theme: Arc<dyn CheckboxTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_checkbox_template() -> Arc<dyn CheckboxTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn CheckboxTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedCheckboxTemplate::new(default_checkbox_theme()))).clone()
}

impl CheckboxTemplate for ThemedCheckboxTemplate {
    fn render(&self, model: &CheckboxRenderModel<'_>, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let appearance = self.theme.resolve(model.checked, model.state);
        let indicator = div()
            .flex()
            .items_center()
            .justify_center()
            .size(px(appearance.indicator_size))
            .bg(appearance.indicator_background)
            .border_1()
            .border_color(appearance.indicator_border)
            .rounded(px(appearance.indicator_radius))
            .child(render_checkmark(model.checked, appearance.checkmark_size, appearance.checkmark_color));

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .items_center()
            .gap(px(appearance.gap))
            .min_h(px(appearance.height))
            .px(px(appearance.control_padding_x))
            .py(px(appearance.control_padding_y))
            .text_color(appearance.label_color)
            .text_size(px(appearance.label_typography.size))
            .line_height(px(appearance.label_typography.line_height))
            .font_weight(appearance.label_typography.weight)
            .rounded(px(appearance.control_radius))
            .cursor_pointer()
            .child(indicator)
            .child(model.label.clone());

        if let Some(background) = appearance.control_background {
            root = root.bg(background);
        }

        if let Some(border) = appearance.control_border {
            root = root.border_1().border_color(border);
        }

        if !model.enabled {
            root = root.opacity(0.56);
        }

        if model.state.focused {
            root = root.child(render_focus_ring(
                appearance.focus_ring.unwrap_or_else(focus_debug_border),
                appearance.control_radius,
            ));
        }

        root
    }
}

fn render_focus_ring(color: gpui::Hsla, radius: f32) -> Div {
    div().absolute().size_full().border_1().border_color(color).rounded(px(radius))
}

fn render_checkmark(checked: bool, size: f32, color: gpui::Hsla) -> AnyElement {
    if checked {
        div()
            .size(px(size))
            .flex()
            .items_center()
            .justify_center()
            .font_family("lucide")
            .font_weight(FontWeight::NORMAL)
            .text_size(px(size))
            .line_height(px(size))
            .text_color(color)
            .child(char::from(LucideIcon::Check).to_string())
            .into_any_element()
    } else {
        div().size(px(size)).into_any_element()
    }
}
