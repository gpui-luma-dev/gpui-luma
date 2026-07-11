use std::sync::Arc;

use gpui::{AnyElement, AppContext, Context, Entity, Subscription, IntoElement};
use gpui_luma_look_shadcn::ShadcnLook;

use super::control::RadioControlGroupDemo;
use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::gallery_pane_with_description;

const DESCRIPTION: &str = "Prototype-only radio control group built on an app-local headless group, with the group root as the single tab stop and arrow-key selection inside the group.";

#[derive(Clone)]
pub(in crate::gallery) struct RadioControlGroupPane {
    demo: Entity<RadioControlGroupDemo>,
}

impl RadioControlGroupPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let demo = cx.new(|cx| RadioControlGroupDemo::new(cx, look));
        Self { demo }
    }

    pub(in crate::gallery) fn subscribe(&self, _cx: &mut Context<GalleryApp>, _subscriptions: &mut Vec<Subscription>) {}

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        gallery_pane_with_description(
            "Radio Control Group",
            Some(DESCRIPTION),
            self.demo.clone().into_any_element(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.demo.update(cx, |demo, cx| {
            demo.notify_controls(cx);
            cx.notify();
        });
    }
}
