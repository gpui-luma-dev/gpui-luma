use std::sync::Arc;

use gpui::{
    AnyElement, Context, Entity, Render, SharedString, Subscription, Window, div, prelude::*, px, transparent_black,
};
use gpui_luma::controls::command::button::{ButtonEvent, ButtonRenderModel};
use gpui_luma::controls::command::icon_button::IconButton;
use gpui_luma::controls::presenter::ControlPresenter;
use gpui_luma::controls::resizable_panels::{
    PanelHideMode, ResizablePanelSpec, ResizablePanels, ResizablePanelsEvent, ResizablePanelsOrientation,
    ResizeHandleSize, ResizeHandleVisibility,
};
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn::prelude::*;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::app::LumaStudioApp;

const DEFAULT_DETAILS_MIN_WIDTH: f32 = 360.0;
const DEFAULT_INSPECTOR_MIN_WIDTH: f32 = 500.0;
const DEFAULT_DETAILS_WEIGHT: f32 = 3.0;
const DEFAULT_INSPECTOR_WEIGHT: f32 = 2.0;
const PANEL_BG_TRANSPARENT: gpui::Hsla = transparent_black();

pub(crate) struct LumaStudioAppHandle {
    pub(crate) entity: Entity<LumaStudioApp>,
}

impl gpui::Global for LumaStudioAppHandle {}

pub struct InspectorSplitShell {
    look: Arc<ShadcnLook>,
    id: SharedString,
    split_panels: Entity<ResizablePanels>,
    inspector_toggle: IconButton,
    _subscriptions: Vec<Subscription>,
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
        let inspector_toggle = look
            .content_only_icon_button(format!("{id}-inspector-toggle"), LucideIcon::InspectionPanel)
            .size(gpui_luma::theme::ControlSize::Sm)
            .spawn(cx);
        let title_color = look.chrome().title_text;
        inspector_toggle.update(cx, |button, cx| {
            button.set_presenter(inspector_toggle_presenter(title_color), cx);
        });

        let inspector_toggle_for_details = inspector_toggle.clone();
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
                    div()
                        .relative()
                        .size_full()
                        .min_h(px(0.0))
                        .min_w(px(0.0))
                        .child(details())
                        .child(
                            div().absolute().top(px(12.0)).right(px(12.0)).child(inspector_toggle_for_details.clone()),
                        )
                        .into_any_element()
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

        let app = cx.global::<LumaStudioAppHandle>().entity.clone();
        let app_for_toggle = app.clone();
        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&inspector_toggle, move |_, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                app_for_toggle.update(cx, |app, cx| {
                    app.set_inspector_open(!app.inspector_open(), cx);
                });
            }
        }));
        let app_for_panels = app.clone();
        subscriptions.push(cx.subscribe(&split_panels, move |_, _, event, cx| {
            if let ResizablePanelsEvent::PanelHiddenChanged { panel_index: 1, hidden } = event {
                app_for_panels.update(cx, |app, cx| {
                    app.set_inspector_open(!hidden, cx);
                });
            }
        }));
        let app_for_observer = app.clone();
        subscriptions.push(cx.observe(&app, move |this, _, cx| {
            let inspector_open = app_for_observer.read(cx).inspector_open();
            this.sync_inspector_panel(inspector_open, cx);
        }));

        Self { look, id, split_panels, inspector_toggle, _subscriptions: subscriptions }
    }

    pub fn request_layout_refresh(&mut self, cx: &mut Context<Self>) {
        self.split_panels.update(cx, |_, cx| cx.notify());
    }

    pub fn set_viewport_size(&mut self, size: gpui::Size<gpui::Pixels>, cx: &mut Context<Self>) {
        self.split_panels.update(cx, |panels, cx| {
            panels.set_frame_size(size.width, size.height, cx);
        });
    }

    fn sync_inspector_panel(&mut self, inspector_open: bool, cx: &mut Context<Self>) {
        let panel_hidden = self.split_panels.read(cx).is_panel_hidden(1);
        if inspector_open == !panel_hidden {
            return;
        }

        self.split_panels.update(cx, |panels, cx| {
            if inspector_open {
                panels.show_panel(1, cx);
            } else {
                panels.hide_panel(1, PanelHideMode::Completely, cx);
            }
        });
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.inspector_toggle.update(cx, |button, cx| {
            button.set_presenter(inspector_toggle_presenter(look.chrome().title_text), cx);
        });
        self.split_panels.update(cx, |panels, cx| {
            panels.set_theme(look.resizable_panels_theme(), cx);
        });
        cx.notify();
    }
}

fn inspector_toggle_presenter(color: gpui::Hsla) -> ControlPresenter<ButtonRenderModel<()>> {
    std::sync::Arc::new(move |_, _| {
        div()
            .child(gpui_luma::controls::icon::lucide_icon(LucideIcon::InspectionPanel, color, 16.0))
            .into_any_element()
    })
}

impl Render for InspectorSplitShell {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let split_panels = self.split_panels.clone();
        let id = self.id.clone();
        with_look(&self.look, || div().id(id).size_full().min_h(px(0.0)).min_w(px(0.0)).child(split_panels))
    }
}
