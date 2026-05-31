use gpui::{AnyElement, App, Div, Stateful, Window, div, px, prelude::*};

use crate::controls::command::button::{ButtonRenderModel, ButtonTemplate};
use crate::controls::template::TemplateWithModifiers;
use crate::define_control_template;
use crate::theme::adorner::{adorner_oversize_extent, render_optional_adorner_with_focus_radius};
use crate::controls::radio_button::{RadioButtonTheme, default_radio_button_theme};
use crate::theme::{GlyphIndicatorScale, InteractionState, LayoutCacheKey, LumaLayoutCacheExt};

define_control_template!(
    ThemedRadioButtonTemplate,
    dyn RadioButtonTheme,
    ButtonRenderModel<bool>,
    ButtonTemplate<bool>,
    default_radio_button_theme()
);

impl ButtonTemplate<bool> for ThemedRadioButtonTemplate {
    fn render(&self, model: &ButtonRenderModel<bool>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let palette = self.theme.resolve(model.data, model.state);

        let focused_probe_appearance = if model.state.disabled {
            None
        } else {
            Some(self.theme.resolve(model.data, InteractionState { focused: true, ..model.state }))
        };
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            self.theme.metrics(),
            LayoutCacheKey { size: model.size, scale_factor_bits: scale_factor.to_bits() },
            |metrics| GlyphIndicatorScale::compute(model.size, metrics, scale_factor),
        );

        let indicator_radius = scale.indicator_size / 2.0;
        let indicator_visual = div()
            .id(format!("{}-indicator", model.id))
            .flex()
            .items_center()
            .justify_center()
            .size(px(scale.indicator_size))
            .bg(palette.indicator_background)
            .border_1()
            .border_color(palette.indicator_border)
            .rounded(px(scale.indicator_size))
            .child(render_dot(model.data, scale.dot_size, palette.dot_color));

        let oversize_extent = adorner_oversize_extent(palette.adorner)
            .max(focused_probe_appearance.as_ref().map(|probe| adorner_oversize_extent(probe.adorner)).unwrap_or(0.0));
        let mut indicator = div().relative().child(indicator_visual);

        if let Some(adorner) = render_optional_adorner_with_focus_radius(palette.adorner, indicator_radius) {
            indicator = indicator.child(adorner);
        }

        let indicator = if oversize_extent > 0.0 {
            div()
                .id(format!("{}-indicator-slot", model.id))
                .flex()
                .items_center()
                .justify_center()
                .size(px(scale.indicator_size + (oversize_extent * 2.0)))
                .child(indicator)
                .into_any_element()
        } else {
            indicator.into_any_element()
        };

        let label = div().mt(px(scale.label_baseline_shift)).child((model.content)(model, cx));

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .items_center()
            .gap(px(scale.gap))
            .min_h(px(scale.height))
            .px(px(scale.control_padding_x))
            .py(px(scale.control_padding_y))
            .text_color(palette.label_color)
            .text_size(px(palette.label_typography.size))
            .line_height(px(palette.label_typography.line_height))
            .font_family(palette.label_font_family.clone())
            .font_weight(palette.label_typography.weight)
            .rounded(px(scale.control_radius))
            .child(indicator)
            .child(label);

        if let Some(background) = palette.control_background {
            root = root.bg(background);
        }

        if let Some(border) = palette.control_border {
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
