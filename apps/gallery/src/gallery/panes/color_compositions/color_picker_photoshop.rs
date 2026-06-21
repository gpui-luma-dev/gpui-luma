use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Subscription, prelude::*};
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::notify_entity;

#[derive(Clone)]
pub(in crate::gallery) struct ColorPickerPane {
    state: Entity<super::compositions::ColorPlaneControlsState>,
}

impl ColorPickerPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let state = cx.new(|cx| super::compositions::ColorPlaneControlsState::new(look, cx));
        Self { state }
    }

    pub(in crate::gallery) fn subscribe(&self, _cx: &mut Context<GalleryApp>, _subscriptions: &mut Vec<Subscription>) {}

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        super::render_single_composition_page(
            "Color Picker Photoshop",
            "A composed picker built from the migrated field and slider primitives.",
            self.state.clone(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.state.update(cx, |state, cx| state.notify_controls(cx));
        notify_entity(&self.state, cx);
    }
}
