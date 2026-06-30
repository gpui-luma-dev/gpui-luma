use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Subscription, prelude::*};
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::notify_entity;

#[derive(Clone)]
pub(in crate::gallery) struct SvTrianglePane {
    state_sm: Entity<super::compositions::HueRingSvTriangleState>,
    state_md: Entity<super::compositions::HueRingSvTriangleState>,
    state_lg: Entity<super::compositions::HueRingSvTriangleState>,
}

impl SvTrianglePane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let state_sm = cx.new(|cx| {
            super::compositions::HueRingSvTriangleState::with_size(
                look.clone(),
                super::compositions::CompositionSize::Sm,
                cx,
            )
        });
        let state_md = cx.new(|cx| {
            super::compositions::HueRingSvTriangleState::with_size(
                look.clone(),
                super::compositions::CompositionSize::Md,
                cx,
            )
        });
        let state_lg = cx.new(|cx| {
            super::compositions::HueRingSvTriangleState::with_size(
                look.clone(),
                super::compositions::CompositionSize::Lg,
                cx,
            )
        });
        Self { state_sm, state_md, state_lg }
    }

    pub(in crate::gallery) fn subscribe(&self, _cx: &mut Context<GalleryApp>, _subscriptions: &mut Vec<Subscription>) {}

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        super::render_sized_composition_page(
            "SV Triangle",
            "Hue ring with an embedded Photoshop-style saturation/value triangle.",
            self.state_sm.clone(),
            self.state_md.clone(),
            self.state_lg.clone(),
            420.0,
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.state_sm.update(cx, |state, cx| state.notify_controls(cx));
        self.state_md.update(cx, |state, cx| state.notify_controls(cx));
        self.state_lg.update(cx, |state, cx| state.notify_controls(cx));
        notify_entity(&self.state_sm, cx);
        notify_entity(&self.state_md, cx);
        notify_entity(&self.state_lg, cx);
    }
}
