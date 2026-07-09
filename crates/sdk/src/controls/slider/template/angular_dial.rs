use std::sync::{Arc, OnceLock};

use gpui::{App, Div, Stateful, Window, canvas, div, px, prelude::*};

use super::{
    DIAL_SIZE, SliderTemplate, SliderTemplateHandlers, TRACK_RADIUS, attach_radial_interaction, effective_thumb_radius,
    paint_radial_annulus, paint_radial_fill_track, render_slider_thumb_at, track_bounds_canvas,
};
use crate::controls::slider::{SliderTheme, default_slider_theme};

use super::super::input::{SliderInputStrategy, angle_for_percentage};
use super::super::layout::display_position;
use super::super::model::{SliderRenderModel, ThumbId, TrackPresentation};

pub struct ThemedAngularDialTemplate {
    theme: Arc<dyn SliderTheme>,
}

impl ThemedAngularDialTemplate {
    pub fn new(theme: Arc<dyn SliderTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_angular_dial_template() -> Arc<dyn SliderTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn SliderTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedAngularDialTemplate::new(default_slider_theme()))).clone()
}

impl SliderTemplate for ThemedAngularDialTemplate {
    fn render(
        &self,
        model: &SliderRenderModel<'_>,
        handlers: SliderTemplateHandlers,
        primary_thumb_id: ThumbId,
        window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let look = self.theme.resolve(model.size, model.thumb_size, model.state);
        let SliderInputStrategy::Angular { min_angle, max_angle } = model.strategy else {
            return div().id(model.id.clone());
        };

        let primary_thumb =
            model.thumbs.iter().find(|thumb| thumb.id == primary_thumb_id).or_else(|| model.thumbs.first());
        let thumb_position = primary_thumb.map(|thumb| thumb.position).unwrap_or(0.0);
        let display_percentage = display_position(thumb_position, model.reversed);
        let angle = angle_for_percentage(min_angle, max_angle, display_percentage);
        let center = DIAL_SIZE * 0.5;
        let thumb_x = center + TRACK_RADIUS * angle.cos();
        let thumb_y = center + TRACK_RADIUS * angle.sin();
        let arc_thickness = look.track_height;
        let presentation = model.presentation;
        let track_background = look.track_background;
        let fill_background = look.fill_background;
        let thumb_radius = effective_thumb_radius(model, &look, window.rem_size());

        let (track_bounds, interaction) = handlers.into();

        let root = div()
            .id(model.id.clone())
            .relative()
            .size(px(DIAL_SIZE))
            .child(
                canvas(
                    move |_, _, _| (),
                    move |bounds, _, window, _| match presentation {
                        TrackPresentation::Fill => paint_radial_fill_track(
                            bounds,
                            min_angle,
                            max_angle,
                            display_percentage,
                            track_background,
                            fill_background,
                            arc_thickness,
                            window,
                        ),
                        TrackPresentation::Domain => {
                            paint_radial_annulus(bounds, min_angle, max_angle, track_background, arc_thickness, window);
                        }
                    },
                )
                .absolute()
                .size_full(),
            )
            .child(render_slider_thumb_at(
                &look,
                format!("{}-thumb", model.id),
                thumb_x,
                thumb_y,
                thumb_radius,
                primary_thumb,
            ))
            .child(track_bounds_canvas(track_bounds));

        attach_radial_interaction(root, model, interaction, primary_thumb_id)
    }
}
