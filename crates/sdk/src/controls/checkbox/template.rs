use gpui::{AnyElement, App, Div, FontWeight, Stateful, Window, div, px, prelude::*};
use lucide_icons::Icon as LucideIcon;

use crate::controls::button_family::ButtonFamilyRole;
use crate::controls::choice_indicator_layout::{
    ChoiceLayoutPolicy, indicator_oversize_extent, shadow_extent_from, should_paint_shadow,
};
use crate::controls::command::button::{ButtonRenderModel, ButtonTemplate};
use crate::controls::checkbox::{CheckboxScale, CheckboxTheme, default_checkbox_theme};
use crate::controls::template::TemplateWithModifiers;
use crate::define_control_template;
use crate::theme::adorner::render_optional_adorner_with_focus_radius;
use crate::theme::{InteractionState, LayoutCacheKey, LumaLayoutCacheExt};

define_control_template!(
    ThemedCheckboxTemplate,
    dyn CheckboxTheme,
    ButtonRenderModel<bool>,
    ButtonTemplate<bool>,
    default_checkbox_theme()
);

impl ButtonTemplate<bool> for ThemedCheckboxTemplate {
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
            |metrics| CheckboxScale::compute(model.size, metrics, scale_factor),
        );

        let adorner = if model.suppress_adorners.get() {
            None
        } else {
            palette.adorner
        };
        let focused_probe_adorner = if model.suppress_adorners.get() {
            None
        } else {
            focused_probe_look.as_ref().and_then(|probe| probe.adorner)
        };
        let shadow_extent =
            shadow_extent_from(palette.indicator_shadow.as_ref(), scale_factor, layout_policy.elevation);
        let oversize_extent = indicator_oversize_extent(
            layout_policy,
            model.state.focused,
            adorner,
            focused_probe_adorner,
            shadow_extent,
        );

        let indicator_visual = {
            let mut indicator = div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(scale.indicator_size))
                .bg(palette.indicator_background)
                .border_1()
                .border_color(palette.indicator_border)
                .rounded(px(scale.indicator_radius))
                .child(render_checkmark(model.data, scale.glyph_size, palette.checkmark_color));

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

        let indicator_only = matches!(model.role, ButtonFamilyRole::Icon);
        let mut indicator = div().relative().child(indicator_visual);

        if let Some(adorner) = render_optional_adorner_with_focus_radius(adorner, scale.indicator_radius) {
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

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .items_center()
            .text_color(palette.label_color)
            .text_size(px(palette.label_typography.size))
            .line_height(px(palette.label_typography.line_height))
            .font_family(palette.label_font_family.clone())
            .font_weight(palette.label_typography.weight)
            .rounded(px(scale.control_radius))
            .cursor_pointer();

        if indicator_only {
            root = root.child(indicator);
        } else {
            let label = div().mt(px(scale.label_baseline_shift)).child((model.content)(model, cx));
            root = root
                .gap(px(scale.gap))
                .min_h(px(scale.height))
                .px(px(scale.control_padding_x))
                .py(px(scale.control_padding_y))
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
        }

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
