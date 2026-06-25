use std::sync::Arc;

use gpui::{AnyElement, Context, Subscription};
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;

use super::{
    draggable::DraggableDialogDemo, modal::ModalDialogDemo, modeless::ModelessDialogDemo,
    positioning::PositioningDialogDemo,
};

#[derive(Clone, Copy)]
pub(in crate::gallery) enum DialogDemoKind {
    Modal,
    Modeless,
    Positioning,
    Draggable,
}

#[derive(Clone)]
pub(in crate::gallery) struct DialogPane {
    pub(in crate::gallery::panes::dialog) modal: ModalDialogDemo,
    pub(in crate::gallery::panes::dialog) modeless: ModelessDialogDemo,
    pub(in crate::gallery::panes::dialog) positioning: PositioningDialogDemo,
    pub(in crate::gallery::panes::dialog) draggable: DraggableDialogDemo,
}

impl DialogPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        Self {
            modal: ModalDialogDemo::new(cx, look.clone()),
            modeless: ModelessDialogDemo::new(cx, look.clone()),
            positioning: PositioningDialogDemo::new(cx, look.clone()),
            draggable: DraggableDialogDemo::new(cx, look),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        self.modal.subscribe(cx, subscriptions);
        self.modeless.subscribe(cx, subscriptions);
        self.positioning.subscribe(cx, subscriptions);
        self.draggable.subscribe(cx, subscriptions);
    }

    pub(in crate::gallery) fn render(&self, kind: DialogDemoKind, look: &ShadcnLook) -> AnyElement {
        match kind {
            DialogDemoKind::Modal => self.modal.render(look),
            DialogDemoKind::Modeless => self.modeless.render(look),
            DialogDemoKind::Positioning => self.positioning.render(look),
            DialogDemoKind::Draggable => self.draggable.render(look),
        }
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.modal.notify_controls(cx);
        self.modeless.notify_controls(cx);
        self.positioning.notify_controls(cx);
        self.draggable.notify_controls(cx);
    }
}
