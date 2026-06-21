use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Subscription, prelude::*};
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::notify_entity;

#[derive(Clone)]
pub(in crate::gallery) struct MultiMixerPane {
    state: Entity<super::compositions::MultiMixerState>,
}

impl MultiMixerPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let state = cx.new(|cx| super::compositions::MultiMixerState::new(look, cx));
        Self { state }
    }

    pub(in crate::gallery) fn subscribe(&self, _cx: &mut Context<GalleryApp>, _subscriptions: &mut Vec<Subscription>) {}

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        super::render_single_composition_page(
            "Multi Mixer",
            "The original mixer page explored multiple color spaces. This port keeps each space as its own interactive card so you can compare the slider primitives directly.",
            self.state.clone(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.state.update(cx, |state, cx| state.notify_controls(cx));
        notify_entity(&self.state, cx);
    }
}
