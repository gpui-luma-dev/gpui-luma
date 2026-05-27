use gpui::{AnyElement, App, Div, Stateful, Window, div, px, prelude::*};

use crate::controls::command::button::{ButtonRenderModel, ButtonTemplate};
use crate::controls::template::TemplateWithModifiers;
use crate::define_control_template;
use crate::theme::adorner::{adorner_oversize_extent, render_optional_adorner_with_focus_radius};
use crate::controls::radio_button::{RadioButtonTheme, default_radio_button_theme};
use crate::theme::InteractionState;

define_control_template!(
    ThemedRadioButtonTemplate,
    dyn RadioButtonTheme,
    ButtonRenderModel<bool>,
    ButtonTemplate<bool>,
    default_radio_button_theme()
);

impl ButtonTemplate<bool> for ThemedRadioButtonTemplate {
    fn render(&self, model: &ButtonRenderModel<bool>, _window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let appearance = self.theme.resolve(model.data, model.state);

        let focused_probe_appearance = if model.state.disabled {
            None
        } else {
            Some(self.theme.resolve(model.data, InteractionState { focused: true, ..model.state }))
        };

        let indicator_radius = appearance.indicator_size / 2.0;
        let indicator_visual = div()
            .id(format!("{}-indicator", model.id))
            .flex()
            .items_center()
            .justify_center()
            .size(px(appearance.indicator_size))
            .bg(appearance.indicator_background)
            .border_1()
            .border_color(appearance.indicator_border)
            .rounded(px(appearance.indicator_size))
            .child(render_dot(model.data, appearance.dot_size, appearance.dot_color));

        let oversize_extent = adorner_oversize_extent(appearance.adorner)
            .max(focused_probe_appearance.as_ref().map(|probe| adorner_oversize_extent(probe.adorner)).unwrap_or(0.0));
        let mut indicator = div().relative().child(indicator_visual);

        if let Some(adorner) = render_optional_adorner_with_focus_radius(appearance.adorner, indicator_radius) {
            indicator = indicator.child(adorner);
        }

        let indicator = if oversize_extent > 0.0 {
            div()
                .id(format!("{}-indicator-slot", model.id))
                .flex()
                .items_center()
                .justify_center()
                .size(px(appearance.indicator_size + (oversize_extent * 2.0)))
                .child(indicator)
                .into_any_element()
        } else {
            indicator.into_any_element()
        };

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
            .font_family(appearance.label_font_family.clone())
            .font_weight(appearance.label_typography.weight)
            .rounded(px(appearance.control_radius))
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
        } else {
            root = root.cursor_pointer();
        }

        self.apply_modifiers(root, model)
    }
}

fn render_dot(selected: bool, size: f32, color: gpui::Hsla) -> AnyElement {
    if selected {
        div().size(px(size)).bg(color).rounded(px(size)).into_any_element()
    } else {
        div().size(px(size)).into_any_element()
    }
}
