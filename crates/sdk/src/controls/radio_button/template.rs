use gpui::{AnyElement, App, Div, Stateful, Window, div, px, prelude::*};

use crate::controls::command::button::{ButtonRenderModel, ButtonTemplate};
use crate::controls::template::TemplateWithModifiers;
use crate::define_control_template;
use crate::theme::adorner::{AdornerSpec, render_adorner};
use crate::controls::radio_button::{RadioButtonTheme, default_radio_button_theme};

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

        let mut indicator = div().relative().child(indicator_visual);

        for spec in &appearance.adorners {
            if let Some(mut adorner) = render_adorner(*spec, indicator_radius) {
                let AdornerSpec::FocusRing(focus_ring) = *spec;
                let focus_ring_radius = match focus_ring.placement {
                    crate::theme::AdornerPlacement::Inset => {
                        (indicator_radius - focus_ring.distance.max(0.0) - focus_ring.width.max(0.0)).max(0.0)
                    }
                    crate::theme::AdornerPlacement::Oversize => indicator_radius + focus_ring.distance.max(0.0),
                };
                adorner = adorner.rounded(px(focus_ring_radius));
                indicator = indicator.child(adorner);
            }
        }

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
