use std::sync::Arc;

use gpui::{App, Context, EventEmitter, IntoElement, Render, SharedString, Window};
use gpui_luma::controls::overlay_window::{
    OverlayWindow, OverlayWindowEvent, OverlayWindowMode, OverlayWindowPosition, OverlayWindowRenderModel,
};
use gpui_luma_look_shadcn::ShadcnLook;

use super::model::DialogModel;
use super::template::DialogTemplate;

pub(in crate::gallery) struct Dialog {
    overlay: OverlayWindow,
    model: DialogModel,
    template: Arc<dyn DialogTemplate>,
}

impl EventEmitter<OverlayWindowEvent> for Dialog {}

impl Dialog {
    pub(in crate::gallery) fn spawn(
        look: Arc<ShadcnLook>,
        id: impl Into<SharedString>,
        template: Arc<dyn DialogTemplate>,
        cx: &mut Context<Self>,
    ) -> Self {
        let dialog = cx.entity().clone();
        let overlay = look
            .overlay_window(id)
            .mode(OverlayWindowMode::Modal)
            .position(OverlayWindowPosition::Center)
            .content(move |overlay_model, window, app| {
                let (render_model, template) = {
                    let dialog = dialog.read(app);
                    (dialog.model.render_model(), Arc::clone(&dialog.template))
                };
                template.render(&render_model, overlay_model, window, app)
            })
            .spawn(cx);

        cx.subscribe(&overlay, |_, _, event: &OverlayWindowEvent, cx| {
            cx.emit(event.clone());
        })
        .detach();

        Self { overlay, model: DialogModel::default(), template }
    }

    pub(in crate::gallery) fn set_title(&mut self, title: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.model.title = Some(title.into());
        self.notify_overlay(cx);
    }

    pub(in crate::gallery) fn set_body_render<F>(&mut self, body: F, cx: &mut Context<Self>)
    where
        F: Fn(&OverlayWindowRenderModel<'_>, &mut Window, &mut App) -> gpui::AnyElement + Send + Sync + 'static,
    {
        self.model.body = Some(Arc::new(body));
        self.notify_overlay(cx);
    }

    pub(in crate::gallery) fn set_footer_render<F>(&mut self, footer: F, cx: &mut Context<Self>)
    where
        F: Fn(&OverlayWindowRenderModel<'_>, &mut Window, &mut App) -> gpui::AnyElement + Send + Sync + 'static,
    {
        self.model.footer = Some(Arc::new(footer));
        self.notify_overlay(cx);
    }

    pub(in crate::gallery) fn open(&mut self, opener: Option<gpui::FocusHandle>, cx: &mut Context<Self>) {
        self.overlay.update(cx, |overlay, cx| overlay.open_from(opener, cx));
    }

    pub(in crate::gallery) fn dismiss(&mut self, cx: &mut Context<Self>) {
        self.overlay.update(cx, |overlay, cx| overlay.dismiss(cx));
    }

    fn notify_overlay(&mut self, cx: &mut Context<Self>) {
        cx.notify();
        self.overlay.update(cx, |_, cx| cx.notify());
    }
}

impl Render for Dialog {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.overlay.clone()
    }
}
