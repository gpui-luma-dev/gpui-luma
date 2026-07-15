use gpui::{App, Div, Stateful, Window, div, px, prelude::*};

use crate::controls::choice_indicator_layout::{
    ChoiceLayoutPolicy, indicator_oversize_extent, shadow_extent_from_slice, should_paint_shadow,
};
use crate::controls::command::button::{ButtonRenderModel, ButtonTemplate};

use crate::controls::switch::{SwitchTheme, default_switch_theme};
use crate::controls::template::TemplateWithModifiers;
use crate::define_control_template;
use crate::theme::adorner::render_optional_adorner_with_focus_radius;
use crate::theme::snap_to_pixel;

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
        let layout_policy = ChoiceLayoutPolicy::from_render_model(model);
        let scale_factor = window.scale_factor();
        let mut scale = self.theme.scale(model.size, scale_factor);
        scale.track_width = snap_to_pixel(scale.track_width + model.switch_track_width_extra, scale_factor);
        let track_radius = model.radius_override.get().unwrap_or(scale.track_radius);
        let thumb_radius = track_radius.min(scale.thumb_size * 0.5);

        let thumb_left = if model.data {
            scale.track_width - scale.thumb_size - scale.track_padding
        } else {
            scale.track_padding
        };
        let thumb_top = snap_to_pixel(((scale.track_height - scale.thumb_size) * 0.5 - 1.0).max(0.0), scale_factor);
        let mut thumb = div()
            .id(format!("{}-thumb", model.id))
            .absolute()
            .left(px(thumb_left))
            .top(px(thumb_top))
            .size(px(scale.thumb_size))
            .flex()
            .items_center()
            .justify_center()
            .bg(palette.thumb_background)
            .border_1()
            .border_color(palette.thumb_border)
            .rounded(px(thumb_radius));

        if let Some(content) = &model.switch_thumb_content {
            thumb = thumb.child(content(model, cx));
        }

        let adorner = if model.suppress_adorners.get() {
            None
        } else {
            palette.adorner
        };
        let focused_probe_adorner = if model.suppress_adorners.get() {
            None
        } else {
            focused_probe_palette.as_ref().and_then(|probe| probe.adorner)
        };
        let shadow_extent = shadow_extent_from_slice(&palette.thumb_shadow, scale_factor, layout_policy.elevation);
        let oversize_extent = indicator_oversize_extent(
            layout_policy,
            model.state.focused,
            adorner,
            focused_probe_adorner,
            shadow_extent,
        );

        let mut track_visual = div()
            .id(format!("{}-track", model.id))
            .relative()
            .w(px(scale.track_width))
            .h(px(scale.track_height))
            .bg(palette.track_background)
            .border_1()
            .border_color(palette.track_border)
            .rounded(px(track_radius));

        if let Some(content) = &model.switch_track_content {
            track_visual = track_visual.child(
                div()
                    .absolute()
                    .top_0()
                    .right_0()
                    .bottom_0()
                    .left_0()
                    .overflow_hidden()
                    .rounded(px(track_radius))
                    .child(content(model, cx)),
            );
        }

        track_visual = track_visual.child(thumb);

        if should_paint_shadow(layout_policy.elevation, model.state.disabled, !palette.thumb_shadow.is_empty()) {
            track_visual = track_visual.shadow(palette.thumb_shadow.clone());
        }

        let mut track = div().relative().child(track_visual);

        if let Some(adorner) = render_optional_adorner_with_focus_radius(adorner, track_radius) {
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
            .rounded(px(track_radius))
            .child(track)
            .child(label);

        if model.state.disabled {
            root = root.opacity(0.56);
        } else {
            root = root.cursor_pointer();
        }

        self.apply_modifiers(root, model)
    }
}
