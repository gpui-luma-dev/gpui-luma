use std::sync::Arc;

use gpui::{AnyElement, Context, Subscription};
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;

use super::modal::ModalDialogDemo;

#[derive(Clone, Copy)]
pub(in crate::gallery) enum DialogDemoKind {
    Modal,
}

#[derive(Clone)]
pub(in crate::gallery) struct DialogPane {
    pub(in crate::gallery::panes::dialog) modal: ModalDialogDemo,
}

impl DialogPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        Self { modal: ModalDialogDemo::new(cx, look) }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        self.modal.subscribe(cx, subscriptions);
    }

    pub(in crate::gallery) fn render(&self, kind: DialogDemoKind, look: &ShadcnLook) -> AnyElement {
        match kind {
            DialogDemoKind::Modal => self.modal.render(look),
        }
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.modal.notify_controls(cx);
    }
}
