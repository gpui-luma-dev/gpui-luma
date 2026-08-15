use gpui::{AnyElement, App, Div, FontWeight, Hsla, Stateful, Window, div, hsla, px, prelude::*};
use lucide_icons::Icon as LucideIcon;

use crate::controls::button_family::ButtonFamilyRole;
use crate::controls::choice_indicator_layout::{reserve_shadow_extent, should_paint_shadow};
use crate::controls::command::button::{ButtonRenderModel, ButtonTemplate};
use crate::controls::checkbox::{CheckboxData, CheckboxPalette, CheckboxScale, CheckboxTheme, default_checkbox_theme};
use crate::controls::template::TemplateWithModifiers;
use crate::define_control_template;
use crate::theme::{InteractionState, LayoutCacheKey, LumaLayoutCacheExt};

define_control_template!(
    ThemedCheckboxTemplate,
    dyn CheckboxTheme,
    ButtonRenderModel<CheckboxData>,
    ButtonTemplate<CheckboxData>,
    default_checkbox_theme()
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

fn lerp_checkbox_palette(off: &CheckboxPalette, on: &CheckboxPalette, progress: f32) -> CheckboxPalette {
    let t = progress.clamp(0.0, 1.0);
    let settled = if t >= 0.5 { on } else { off };
    CheckboxPalette {
        control_background: lerp_optional_hsla(off.control_background, on.control_background, t),
        control_border: lerp_optional_hsla(off.control_border, on.control_border, t),
        indicator_background: lerp_hsla(off.indicator_background, on.indicator_background, t),
        indicator_border: lerp_hsla(off.indicator_border, on.indicator_border, t),
        checkmark_color: lerp_hsla(off.checkmark_color, on.checkmark_color, t),
        label_color: settled.label_color,
        label_typography: settled.label_typography,
        label_font_family: settled.label_font_family.clone(),
        indicator_shadow: settled.indicator_shadow.clone(),
    }
}

impl ButtonTemplate<CheckboxData> for ThemedCheckboxTemplate {
    fn render(&self, model: &ButtonRenderModel<CheckboxData>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let progress = model.data.progress.clamp(0.0, 1.0);
        let checked = model.data.checked;
        let off_palette = self.theme.resolve(false, model.state, model.size);
        let on_palette = self.theme.resolve(true, model.state, model.size);
        let palette = lerp_checkbox_palette(&off_palette, &on_palette, progress);
        let settled_palette = if checked { &on_palette } else { &off_palette };
        let elevation_probe_look = if model.elevation && model.state.disabled {
            Some(self.theme.resolve(checked, InteractionState { disabled: false, ..model.state }, model.size))
        } else {
            None
        };
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            self.theme.metrics(),
            LayoutCacheKey { size: model.size, scale_factor_bits: scale_factor.to_bits() },
            |metrics| CheckboxScale::compute(model.size, metrics, scale_factor),
        );

        let shadow_extent = reserve_shadow_extent(
            settled_palette.indicator_shadow.as_ref(),
            elevation_probe_look.as_ref().and_then(|probe| probe.indicator_shadow.as_ref()),
            scale_factor,
            model.elevation,
        );
        let oversize_extent = shadow_extent;

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
                .child(render_checkmark(progress, scale.glyph_size, palette.checkmark_color));

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

        let indicator_only = matches!(model.role, ButtonFamilyRole::Icon);
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

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .items_center()
            .text_color(settled_palette.label_color)
            .text_size(px(settled_palette.label_typography.size))
            .line_height(px(settled_palette.label_typography.line_height))
            .font_family(settled_palette.label_font_family.clone())
            .font_weight(settled_palette.label_typography.weight)
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

fn render_checkmark(progress: f32, size: f32, color: gpui::Hsla) -> AnyElement {
    let progress = progress.clamp(0.0, 1.0);
    if progress <= f32::EPSILON {
        return div().size(px(size)).into_any_element();
    }

    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .font_weight(FontWeight::NORMAL)
        .text_size(px(size))
        .line_height(px(size))
        .text_color(color)
        .opacity(progress)
        .child(crate::controls::icon::lucide_glyph(LucideIcon::Check))
        .into_any_element()
}
