use std::sync::{Arc, OnceLock};

use gpui::{App, Div, Stateful, Window, canvas, div, px, prelude::*};

use super::{
    DIAL_SIZE, Slider2Template, Slider2TemplateHandlers, TRACK_RADIUS, attach_radial_interaction, paint_radial_annulus,
    paint_radial_fill_track, render_slider2_thumb_at, slider2_thumb_size, track_bounds_canvas, track_muted_background,
    track_surface_background, uses_static_track_surface,
};
use crate::controls::slider::{SliderTheme, default_slider_theme};

use super::super::input::{Slider2InputStrategy, angle_for_percentage};
use super::super::layout::display_position;
use super::super::model::{Slider2RenderModel, ThumbId, TrackPresentation};

pub struct ThemedCircularRingTemplate {
    theme: Arc<dyn SliderTheme>,
}

impl ThemedCircularRingTemplate {
    pub fn new(theme: Arc<dyn SliderTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_circular_ring_template() -> Arc<dyn Slider2Template> {
    static TEMPLATE: OnceLock<Arc<dyn Slider2Template>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedCircularRingTemplate::new(default_slider_theme()))).clone()
}

impl Slider2Template for ThemedCircularRingTemplate {
    fn render(
        &self,
        model: &Slider2RenderModel<'_>,
        handlers: Slider2TemplateHandlers,
        primary_thumb_id: ThumbId,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let look = self.theme.resolve(model.size, model.thumb_size.map(slider2_thumb_size), model.state);
        let Slider2InputStrategy::Angular { min_angle, max_angle } = model.strategy else {
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
        let track_background = track_muted_background(&look);
        let fill_background = look.fill_background;
        let surface_background = track_surface_background(model, &look);
        let static_surface = uses_static_track_surface(model);

        let (track_bounds, interaction) = handlers.into();

        let root = div()
            .id(model.id.clone())
            .relative()
            .size(px(DIAL_SIZE))
            .child(
                canvas(
                    move |_, _, _| (),
                    move |bounds, _, window, _| {
                        if static_surface {
                            paint_radial_annulus(
                                bounds,
                                min_angle,
                                max_angle,
                                surface_background,
                                arc_thickness,
                                window,
                            );
                            return;
                        }

                        match presentation {
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
                                paint_radial_annulus(
                                    bounds,
                                    min_angle,
                                    max_angle,
                                    surface_background,
                                    arc_thickness,
                                    window,
                                );
                            }
                        }
                    },
                )
                .absolute()
                .size_full(),
            )
            .child(render_slider2_thumb_at(&look, format!("{}-thumb", model.id), thumb_x, thumb_y, primary_thumb))
            .child(track_bounds_canvas(track_bounds));

        attach_radial_interaction(root, model, interaction, primary_thumb_id)
    }
}
