use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Render, SharedString, Window, div, prelude::*, px, transparent_black};
use gpui_luma::controls::resizable_panels::{
    ResizablePanelSpec, ResizablePanels, ResizablePanelsOrientation, ResizeHandleSize, ResizeHandleVisibility,
};
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn::prelude::*;

const DEFAULT_DETAILS_MIN_WIDTH: f32 = 360.0;
const DEFAULT_INSPECTOR_MIN_WIDTH: f32 = 500.0;
const DEFAULT_DETAILS_WEIGHT: f32 = 3.0;
const DEFAULT_INSPECTOR_WEIGHT: f32 = 2.0;
const PANEL_BG_TRANSPARENT: gpui::Hsla = transparent_black();

pub struct InspectorSplitShell {
    look: Arc<ShadcnLook>,
    id: SharedString,
    split_panels: Entity<ResizablePanels>,
}

impl InspectorSplitShell {
    pub fn new<L, R>(
        cx: &mut Context<Self>,
        look: Arc<ShadcnLook>,
        id: impl Into<SharedString>,
        details: L,
        inspector: R,
    ) -> Self
    where
        L: Fn() -> AnyElement + 'static,
        R: Fn() -> AnyElement + 'static,
    {
        let id = id.into();
        let split_panels = look
            .resizable_panels(format!("{id}-split"))
            .orientation(ResizablePanelsOrientation::Horizontal)
            .show_border(false)
            .show_handle(true)
            .handle_visibility(ResizeHandleVisibility::Always)
            .resize_handle(ResizeHandleSize::Md)
            .handle_grip(true)
            .panels([
                ResizablePanelSpec::new_render(move || {
                    div().size_full().min_h(px(0.0)).min_w(px(0.0)).child(details()).into_any_element()
                })
                .weight(DEFAULT_DETAILS_WEIGHT)
                .min(px(DEFAULT_DETAILS_MIN_WIDTH))
                .bg(PANEL_BG_TRANSPARENT),
                ResizablePanelSpec::new_render(move || {
                    div().size_full().min_h(px(0.0)).min_w(px(0.0)).child(inspector()).into_any_element()
                })
                .weight(DEFAULT_INSPECTOR_WEIGHT)
                .min(px(DEFAULT_INSPECTOR_MIN_WIDTH))
                .bg(PANEL_BG_TRANSPARENT),
            ])
            .spawn(cx);

        Self { look, id, split_panels }
    }

    pub fn request_layout_refresh(&mut self, cx: &mut Context<Self>) {
        self.split_panels.update(cx, |_, cx| cx.notify());
    }

    pub fn set_viewport_size(&mut self, size: gpui::Size<gpui::Pixels>, cx: &mut Context<Self>) {
        self.split_panels.update(cx, |panels, cx| {
            panels.set_frame_size(size.width, size.height, cx);
        });
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.split_panels.update(cx, |panels, cx| {
            panels.set_theme(look.resizable_panels_theme(), cx);
        });
        cx.notify();
    }
}

impl Render for InspectorSplitShell {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let split_panels = self.split_panels.clone();
        let id = self.id.clone();
        with_look(&self.look, || div().id(id).size_full().min_h(px(0.0)).min_w(px(0.0)).child(split_panels))
    }
}
