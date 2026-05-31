use gpui::{App, Div, Stateful, Window, div, px, prelude::*};

use crate::controls::command::button::{ButtonRenderModel, ButtonTemplate};

use crate::controls::switch::{SwitchTheme, default_switch_theme};
use crate::controls::template::TemplateWithModifiers;
use crate::define_control_template;
use crate::theme::adorner::{adorner_oversize_extent, render_optional_adorner_with_focus_radius};
use crate::theme::{LayoutCacheKey, LumaLayoutCacheExt, StandardBoxScale, snap_to_pixel};

define_control_template!(
    ThemedSwitchTemplate,
    dyn SwitchTheme,
    ButtonRenderModel<bool>,
    ButtonTemplate<bool>,
    default_switch_theme()
);

impl ButtonTemplate<bool> for ThemedSwitchTemplate {
    fn render(&self, model: &ButtonRenderModel<bool>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let palette = self.theme.resolve(model.data, model.state);
        let focused_probe_palette = if model.state.disabled {
            None
        } else {
            Some(self.theme.resolve(model.data, crate::theme::InteractionState { focused: true, ..model.state }))
        };
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            self.theme.metrics(),
            LayoutCacheKey { size: model.size, scale_factor_bits: scale_factor.to_bits() },
            |metrics| StandardBoxScale::compute(model.size, metrics, scale_factor),
        );

        let thumb_left = if model.data {
            scale.track_width - scale.thumb_size - scale.track_padding
        } else {
            scale.track_padding
        };
        let thumb_top = snap_to_pixel(((scale.track_height - scale.thumb_size) * 0.5 - 1.0).max(0.0), scale_factor);
        let thumb = div()
            .id(format!("{}-thumb", model.id))
            .absolute()
            .left(px(thumb_left))
            .top(px(thumb_top))
            .size(px(scale.thumb_size))
            .bg(palette.thumb_background)
            .border_1()
            .border_color(palette.thumb_border)
            .rounded(px(scale.track_radius))
            .shadow(palette.thumb_shadow.clone());

        let track_visual = div()
            .id(format!("{}-track", model.id))
            .relative()
            .w(px(scale.track_width))
            .h(px(scale.track_height))
            .bg(palette.track_background)
            .border_1()
            .border_color(palette.track_border)
            .rounded(px(scale.track_radius))
            .child(thumb);

        let oversize_extent = adorner_oversize_extent(palette.adorner)
            .max(focused_probe_palette.as_ref().map(|probe| adorner_oversize_extent(probe.adorner)).unwrap_or(0.0));
        let mut track = div().relative().child(track_visual);

        if let Some(adorner) = render_optional_adorner_with_focus_radius(palette.adorner, scale.track_radius) {
            track = track.child(adorner);
        }

        let track = if oversize_extent > 0.0 {
            div()
                .id(format!("{}-track-slot", model.id))
                .flex()
                .items_center()
                .justify_center()
                .w(px(scale.track_width + (oversize_extent * 2.0)))
                .h(px(scale.track_height + (oversize_extent * 2.0)))
                .child(track)
                .into_any_element()
        } else {
            track.into_any_element()
        };

        let label = div().mt(px(scale.label_baseline_shift)).child((model.content)(model, cx));

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .items_center()
            .gap(px(scale.gap))
            .text_color(palette.label_color)
            .text_size(px(palette.label_typography.size))
            .line_height(px(palette.label_typography.line_height))
            .font_family(palette.label_font_family.clone())
            .font_weight(palette.label_typography.weight)
            .rounded(px(scale.track_radius))
            .child(track)
            .child(label);

        if model.state.disabled {
            root = root.opacity(0.56);
        } else {
            root = root.cursor_pointer();
        }

        // Apply modifiers from the pipeline
        self.apply_modifiers(root, model)
    }
}
