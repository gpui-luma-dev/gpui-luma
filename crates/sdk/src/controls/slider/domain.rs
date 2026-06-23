use std::sync::Arc;

use gpui::{Bounds, Hsla, Image, Pixels, Window, canvas, prelude::*};

use super::model::{SliderOrientation, SliderRenderModel};

pub trait DomainTrackRenderer: Send + Sync {
    fn paint(&self, bounds: Bounds<Pixels>, orientation: SliderOrientation, reversed: bool, window: &mut Window);

    fn get_color_at_position(&self, position: f32) -> Option<Hsla> {
        let _ = position;
        None
    }

    /// When set, templates may render this bitmap instead of invoking [`Self::paint`].
    fn raster_image(
        &self,
        _bounds: Bounds<Pixels>,
        _orientation: SliderOrientation,
        _reversed: bool,
    ) -> Option<Arc<Image>> {
        None
    }
}

pub(crate) fn render_domain_track_layer(
    model: &SliderRenderModel<'_>,
    track_radius: Pixels,
) -> Option<impl IntoElement> {
    let renderer = model.domain_track.as_ref()?;

    let orientation = model.orientation;
    let reversed = model.reversed;
    let renderer = Arc::clone(renderer);

    Some(
        canvas(
            move |_, _, _| (),
            move |bounds, _, window, _| {
                renderer.paint(bounds, orientation, reversed, window);
            },
        )
        .absolute()
        .inset_0()
        .rounded(track_radius)
        .overflow_hidden(),
    )
}
