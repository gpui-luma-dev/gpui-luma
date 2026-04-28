

use gpui::{AnyElement, App, Div, FontWeight, Hsla, Stateful, Window, div, hsla, px, prelude::*};
use lucide_icons::Icon as LucideIcon;

use crate::controls::prototypes::mod_button::{ButtonRenderModel, ButtonTemplate};
use crate::theme::{CheckboxTheme, default_checkbox_theme};

use crate::controls::template::TemplateWithModifiers;
use crate::define_control_template;

define_control_template!(
    ThemedCheckboxTemplate,
    dyn CheckboxTheme,
    ButtonRenderModel<bool>,
    ButtonTemplate<bool>,
    default_checkbox_theme()
);

impl ButtonTemplate<bool> for ThemedCheckboxTemplate {
    fn render(&self, model: &ButtonRenderModel<bool>, _window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let appearance = self.theme.resolve(model.data, model.state);
        let indicator = div()
            .flex()
            .items_center()
            .justify_center()
            .size(px(appearance.indicator_size))
            .bg(appearance.indicator_background)
            .border_1()
            .border_color(appearance.indicator_border)
            .rounded(px(appearance.indicator_radius))
            .child(render_checkmark(model.data, appearance.checkmark_size, appearance.checkmark_color));
        let indicator = render_checkbox_focus_ring(indicator, appearance.focus_ring, appearance.indicator_radius);

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
            .child((model.content)(model, cx));

        if let Some(background) = appearance.control_background {
            root = root.bg(background);
        }

        if let Some(border) = appearance.control_border {
            root = root.border_1().border_color(border);
        }

        if model.state.disabled {
            root = root.opacity(0.56);
        }

        // Apply modifiers from the pipeline
        self.apply_modifiers(root, model)
    }
}

fn render_checkbox_focus_ring(indicator: Div, focus_ring: Option<Hsla>, radius: f32) -> Div {
    let ring_gap = 1.0;
    let ring_width = 1.0;
    let ring_color = focus_ring.unwrap_or_else(|| hsla(0.0, 0.0, 0.0, 0.0));

    div()
        .flex()
        .items_center()
        .justify_center()
        .p(px(ring_gap))
        .border_1()
        .border_color(ring_color)
        .rounded(px(radius + ring_gap + ring_width))
        .child(indicator)
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
