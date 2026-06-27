use gpui::{AnyElement, App, Div, Stateful, Window, div, px, prelude::*};

use crate::controls::button_family::ButtonFamilyRole;
use crate::controls::choice_indicator_layout::{
    ChoiceLayoutPolicy, indicator_oversize_extent, shadow_extent_from, should_paint_shadow,
};
use crate::controls::command::button::{ButtonRenderModel, ButtonTemplate};
use crate::controls::template::TemplateWithModifiers;
use crate::define_control_template;
use crate::theme::adorner::render_optional_adorner_with_focus_radius;
use crate::controls::radio_button::{RadioButtonTheme, RadioScale, default_radio_button_theme};
use crate::theme::{InteractionState, LayoutCacheKey, LumaLayoutCacheExt};

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

        let focused_probe_look = if model.state.disabled {
            None
        } else {
            Some(self.theme.resolve(model.data, InteractionState { focused: true, ..model.state }))
        };
        let layout_policy = ChoiceLayoutPolicy::from_render_model(model);
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            self.theme.metrics(),
            LayoutCacheKey { size: model.size, scale_factor_bits: scale_factor.to_bits() },
            |metrics| RadioScale::compute(model.size, metrics, scale_factor),
        );

        let indicator_radius = scale.indicator_size / 2.0;
        let indicator_only = matches!(model.role, ButtonFamilyRole::Icon);
        let shadow_extent =
            shadow_extent_from(palette.indicator_shadow.as_ref(), scale_factor, layout_policy.elevation);
        let oversize_extent = if indicator_only {
            0.0
        } else {
            indicator_oversize_extent(
                layout_policy,
                model.state.focused,
                palette.adorner,
                focused_probe_look.as_ref().and_then(|probe| probe.adorner),
                shadow_extent,
            )
        };

        let indicator_visual = {
            let mut indicator = div()
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

            if should_paint_shadow(
                layout_policy.elevation,
                model.state.disabled,
                palette.indicator_shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()),
            ) && let Some(shadows) = palette.indicator_shadow.as_ref()
            {
                indicator = indicator.shadow(shadows.clone());
            }

            indicator
        };

        let mut indicator = div().relative().child(indicator_visual);

        if !indicator_only && let Some(adorner) = render_optional_adorner_with_focus_radius(palette.adorner, indicator_radius) {
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
            .rounded(px(scale.control_radius));

        if indicator_only {
            root = root.child(indicator);
        } else {
            root = root
                .text_color(palette.label_color)
                .text_size(px(palette.label_typography.size))
                .line_height(px(palette.label_typography.line_height))
                .font_family(palette.label_font_family.clone())
                .font_weight(palette.label_typography.weight)
                .child(indicator)
                .child(label);
        }

        if let Some(background) = palette.control_background {
            root = root.bg(background);
        }

        if let Some(border) = palette.control_border {
            root = root.border_1().border_color(border);
        }

        if model.state.disabled {
            root = root.opacity(0.56);
        } else if !indicator_only {
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
