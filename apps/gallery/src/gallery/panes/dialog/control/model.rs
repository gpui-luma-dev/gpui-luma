use std::sync::Arc;

use gpui::{AnyElement, App, SharedString, Window};
use gpui_luma::controls::overlay_window::OverlayWindowRenderModel;

pub(in crate::gallery) type DialogRenderer =
    Arc<dyn Fn(&OverlayWindowRenderModel<'_>, &mut Window, &mut App) -> AnyElement + Send + Sync + 'static>;

#[derive(Default)]
pub(in crate::gallery) struct DialogModel {
    pub title: Option<SharedString>,
    pub header_end: Option<DialogRenderer>,
    pub body: Option<DialogRenderer>,
    pub footer: Option<DialogRenderer>,
}

pub(in crate::gallery) struct DialogRenderModel {
    pub title: Option<SharedString>,
    pub header_end: Option<DialogRenderer>,
    pub body: Option<DialogRenderer>,
    pub footer: Option<DialogRenderer>,
}

impl DialogModel {
    pub fn render_model(&self) -> DialogRenderModel {
        DialogRenderModel {
            title: self.title.clone(),
            header_end: self.header_end.clone(),
            body: self.body.clone(),
            footer: self.footer.clone(),
        }
    }
}
