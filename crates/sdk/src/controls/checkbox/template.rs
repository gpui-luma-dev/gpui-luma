use gpui::{AnyElement, App, Div, FontWeight, Stateful, Window, div, px, prelude::*};
use lucide_icons::Icon as LucideIcon;

use crate::controls::command::button::{ButtonRenderModel, ButtonTemplate};
use crate::theme::adorner::{AdornerSpec, max_oversize_extent, render_adorner};
use crate::controls::checkbox::{CheckboxTheme, default_checkbox_theme};
use crate::theme::InteractionState;

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
        let focused_probe_appearance = if model.state.disabled {
            None
        } else {
            Some(self.theme.resolve(model.data, InteractionState { focused: true, ..model.state }))
        };

        let indicator_visual = div()
            .flex()
            .items_center()
            .justify_center()
            .size(px(appearance.indicator_size))
            .bg(appearance.indicator_background)
            .border_1()
            .border_color(appearance.indicator_border)
            .rounded(px(appearance.indicator_radius))
            .child(render_checkmark(model.data, appearance.checkmark_size, appearance.checkmark_color));

        let oversize_extent = max_oversize_extent(&appearance.adorners)
            .max(focused_probe_appearance.as_ref().map(|probe| max_oversize_extent(&probe.adorners)).unwrap_or(0.0));
        let mut indicator = div().relative().child(indicator_visual);

        for spec in &appearance.adorners {
            if let Some(mut adorner) = render_adorner(*spec, appearance.indicator_radius) {
                let AdornerSpec::FocusRing(focus_ring) = *spec;
                let focus_ring_radius = match focus_ring.placement {
                    crate::theme::AdornerPlacement::Inset => {
                        (appearance.indicator_radius - focus_ring.distance.max(0.0) - focus_ring.width.max(0.0))
                            .max(0.0)
                    }
                    crate::theme::AdornerPlacement::Oversize => {
                        appearance.indicator_radius + focus_ring.distance.max(0.0)
                    }
                };
                adorner = adorner.rounded(px(focus_ring_radius));
                indicator = indicator.child(adorner);
            }
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
