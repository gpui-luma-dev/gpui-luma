use gpui::{AnyElement, App, Div, Hsla, Stateful, Window, div, hsla, px, prelude::*};

use crate::controls::button_family::ButtonFamilyRole;
use crate::controls::choice_indicator_layout::should_paint_shadow;
use crate::controls::command::button::{ButtonRenderModel, ButtonTemplate};
use crate::controls::radio_button::{
    RadioButtonData, RadioButtonPalette, RadioButtonTheme, RadioScale, default_radio_button_theme,
};
use crate::controls::template::TemplateWithModifiers;
use crate::define_control_template;
use crate::theme::{LayoutCacheKey, LumaLayoutCacheExt};

define_control_template!(
    ThemedRadioButtonTemplate,
    dyn RadioButtonTheme,
    ButtonRenderModel<RadioButtonData>,
    ButtonTemplate<RadioButtonData>,
    default_radio_button_theme()
);

fn lerp_f32(start: f32, end: f32, t: f32) -> f32 {
    start + ((end - start) * t)
}

fn lerp_hsla(start: Hsla, end: Hsla, t: f32) -> Hsla {
    hsla(
        lerp_f32(start.h, end.h, t),
        lerp_f32(start.s, end.s, t),
        lerp_f32(start.l, end.l, t),
        lerp_f32(start.a, end.a, t),
    )
}

fn lerp_optional_hsla(start: Option<Hsla>, end: Option<Hsla>, t: f32) -> Option<Hsla> {
    match (start, end) {
        (Some(a), Some(b)) => Some(lerp_hsla(a, b, t)),
        (Some(a), None) if t < 0.5 => Some(a),
        (None, Some(b)) if t >= 0.5 => Some(b),
        _ => None,
    }
}

fn lerp_radio_palette(off: &RadioButtonPalette, on: &RadioButtonPalette, progress: f32) -> RadioButtonPalette {
    let t = progress.clamp(0.0, 1.0);
    let settled = if t >= 0.5 { on } else { off };
    RadioButtonPalette {
        control_background: lerp_optional_hsla(off.control_background, on.control_background, t),
        control_border: lerp_optional_hsla(off.control_border, on.control_border, t),
        indicator_background: lerp_hsla(off.indicator_background, on.indicator_background, t),
        indicator_border: lerp_hsla(off.indicator_border, on.indicator_border, t),
        dot_color: lerp_hsla(off.dot_color, on.dot_color, t),
        label_color: settled.label_color,
        label_typography: settled.label_typography,
        label_font_family: settled.label_font_family.clone(),
        indicator_shadow: settled.indicator_shadow.clone(),
    }
}

impl ButtonTemplate<RadioButtonData> for ThemedRadioButtonTemplate {
    fn render(&self, model: &ButtonRenderModel<RadioButtonData>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let progress = model.data.progress.clamp(0.0, 1.0);
        let selected = model.data.selected;
        let off_palette = self.theme.resolve(false, model.state, model.size);
        let on_palette = self.theme.resolve(true, model.state, model.size);
        let palette = lerp_radio_palette(&off_palette, &on_palette, progress);
        let settled_palette = if selected { &on_palette } else { &off_palette };

        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            self.theme.metrics(),
            LayoutCacheKey { size: model.size, scale_factor_bits: scale_factor.to_bits() },
            |metrics| RadioScale::compute(model.size, metrics, scale_factor),
        );

        let indicator_only = matches!(model.role, ButtonFamilyRole::Icon);
        let oversize_extent = 0.0;

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
                .child(render_dot(progress, scale.dot_size, palette.dot_color));

            if should_paint_shadow(
                model.elevation,
                model.state.disabled,
                settled_palette.indicator_shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()),
            ) && let Some(shadows) = settled_palette.indicator_shadow.as_ref()
            {
                indicator = indicator.shadow(shadows.clone());
            }

            indicator
        };

        let indicator = div().relative().child(indicator_visual);

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
                .text_color(settled_palette.label_color)
                .text_size(px(settled_palette.label_typography.size))
                .line_height(px(settled_palette.label_typography.line_height))
                .font_family(settled_palette.label_font_family.clone())
                .font_weight(settled_palette.label_typography.weight)
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

fn render_dot(progress: f32, size: f32, color: gpui::Hsla) -> AnyElement {
    let progress = progress.clamp(0.0, 1.0);
    if progress <= f32::EPSILON {
        return div().size(px(size)).into_any_element();
    }

    // Scale the filled dot with progress so selection feels continuous.
    let dot_size = (size * progress).max(1.0);
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .child(div().size(px(dot_size)).bg(color).rounded(px(dot_size)).opacity(progress))
        .into_any_element()
}
