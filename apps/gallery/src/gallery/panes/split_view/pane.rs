#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use gpui::{
    AnyElement, Context, Entity, FontWeight, IntoElement, ParentElement, SharedString, Subscription, div, prelude::*,
    px, rgb,
};
use gpui_luma::controls::command::button::{Button, ButtonEvent, HasPresenter};
use gpui_luma::controls::command::icon_button::IconButton;
use gpui_luma::controls::split_view::{SplitView, SplitViewEvent, SplitViewSeparatorVisibility};
use gpui_luma::controls::switch::Switch;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_split_view_inspect_tree;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_with_inspector_description, notify_entity, InspectorToggleRegistry};
use super::shared::{
    DEMO_HEIGHT, DEMO_WIDTH, ICON_RAIL_COLLAPSED_WIDTH, apply_separator_visibility_toggle, content_pane_with_panel,
    demo_frame, inset_content_pane, mockup_shell, nav_pane_mock, separator_switch, separator_visibility_label,
    shell_content_pane,
};

#[derive(Clone, Copy)]
pub(in crate::gallery) enum SplitViewDemoKind {
    Unified,
    Inset,
    IconRail,
    Detached,
}

#[derive(Clone)]
struct DemoState {
    split_view: Entity<SplitView>,
    separator_switch: Switch,
    sidebar_width: f32,
    separator_visibility: SplitViewSeparatorVisibility,
}

#[derive(Clone)]
struct IconRailDemo {
    split_view: Entity<SplitView>,
    separator_switch: Switch,
    toggle_button: Entity<Button<()>>,
    sidebar_width: f32,
    effective_width: f32,
    collapsed: bool,
    separator_visibility: SplitViewSeparatorVisibility,
    last_event: SharedString,
}

#[derive(Clone)]
struct DetachedDemo {
    split_view: Entity<SplitView>,
    separator_switch: Switch,
    toggle_button: IconButton,
    sidebar_width: f32,
    separator_visibility: SplitViewSeparatorVisibility,
}

#[derive(Clone)]
pub(in crate::gallery) struct SplitViewPane {
    unified: DemoState,
    inset: DemoState,
    icon_rail: IconRailDemo,
    detached: DetachedDemo,
    inspector: Entity<ColorInspectorShell>,
}

impl SplitViewPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let unified = Self::spawn_demo(
            look.clone(),
            "split-view-unified",
            px(300.0),
            px(240.0),
            px(0.0),
            SplitViewSeparatorVisibility::Hover,
            nav_pane_mock,
            shell_content_pane,
            cx,
        );

        let inset = Self::spawn_demo(
            look.clone(),
            "split-view-inset",
            px(400.0),
            px(320.0),
            px(0.0),
            SplitViewSeparatorVisibility::Hover,
            nav_pane_mock,
            inset_content_pane,
            cx,
        );

        let icon_rail_split = look
            .split_view("split-view-icon-rail")
            .sidebar_width(px(560.0))
            .sidebar_min_width(px(440.0))
            .sidebar_max_width(px(760.0))
            .sidebar_collapsed_width(px(ICON_RAIL_COLLAPSED_WIDTH))
            .separator_visibility(SplitViewSeparatorVisibility::Hover)
            .sidebar(icon_rail_nav_pane)
            .content(shell_content_pane)
            .spawn(cx);
        let icon_rail_visibility = icon_rail_split.read(cx).separator_visibility();
        let icon_rail_sidebar_width = icon_rail_split.read(cx).sidebar_width().as_f32();
        let icon_rail_effective_width = icon_rail_split.read(cx).effective_sidebar_width().as_f32();
        let icon_rail_collapsed = icon_rail_split.read(cx).collapsed();
        let icon_rail = IconRailDemo {
            split_view: icon_rail_split,
            separator_switch: separator_switch(
                &look,
                "split-view-icon-rail-separator-visibility",
                icon_rail_visibility,
                cx,
            ),
            toggle_button: look.secondary_button("split-view-icon-rail-toggle").label("Toggle Collapse").spawn(cx),
            sidebar_width: icon_rail_sidebar_width,
            effective_width: icon_rail_effective_width,
            collapsed: icon_rail_collapsed,
            separator_visibility: icon_rail_visibility,
            last_event: "None".into(),
        };

        let detached_toggle = look.ghost_icon_button("split-view-detached-toggle", LucideIcon::Menu).spawn(cx);
        let detached_toggle_for_content = detached_toggle.clone();
        let detached_split = look
            .split_view("split-view-detached")
            .sidebar_width(px(560.0))
            .sidebar_min_width(px(440.0))
            .sidebar_max_width(px(840.0))
            .separator_visibility(SplitViewSeparatorVisibility::Hover)
            .sidebar(detached_nav_pane)
            .content(move || detached_content_pane(detached_toggle_for_content.clone()))
            .spawn(cx);
        let detached_visibility = detached_split.read(cx).separator_visibility();
        let detached_sidebar_width = detached_split.read(cx).sidebar_width().as_f32();
        let detached = DetachedDemo {
            split_view: detached_split,
            separator_switch: separator_switch(
                &look,
                "split-view-detached-separator-visibility",
                detached_visibility,
                cx,
            ),
            toggle_button: detached_toggle,
            sidebar_width: detached_sidebar_width,
            separator_visibility: detached_visibility,
        };

        let tree =
            spawn_color_inspector_tree("split-view-inspector-tree", look.clone(), build_split_view_inspect_tree, cx);
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "split-view-inspector",
                "split-view-inspector-split",
                "split-view-inspector-detail",
                build_split_view_inspect_tree,
                cx,
            )
        });

        Self { unified, inset, icon_rail, detached, inspector }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        Self::subscribe_demo(&self.unified, cx, subscriptions, |app, event, cx| {
            app.panes.split_view.unified.apply_separator_toggle(event, cx);
        });
        Self::subscribe_demo(&self.inset, cx, subscriptions, |app, event, cx| {
            app.panes.split_view.inset.apply_separator_toggle(event, cx);
        });
        subscriptions.push(cx.subscribe(&self.icon_rail.separator_switch, |app, _, event: &ButtonEvent, cx| {
            app.panes.split_view.icon_rail.apply_separator_toggle(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.detached.separator_switch, |app, _, event: &ButtonEvent, cx| {
            app.panes.split_view.detached.apply_separator_toggle(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.unified.split_view, |app, _, event: &SplitViewEvent, cx| {
            app.panes.split_view.unified.apply_split_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.inset.split_view, |app, _, event: &SplitViewEvent, cx| {
            app.panes.split_view.inset.apply_split_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.icon_rail.toggle_button, |app, _, _: &ButtonEvent, cx| {
            app.panes.split_view.icon_rail.split_view.update(cx, |split_view, cx| {
                split_view.toggle_collapsed(cx);
            });
        }));
        subscriptions.push(cx.subscribe(&self.icon_rail.split_view, |app, _, event: &SplitViewEvent, cx| {
            app.panes.split_view.icon_rail.apply_split_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.detached.toggle_button, |app, _, _: &ButtonEvent, cx| {
            app.panes.split_view.detached.split_view.update(cx, |split_view, cx| {
                split_view.toggle_collapsed(cx);
            });
        }));
        subscriptions.push(cx.subscribe(&self.detached.split_view, |app, _, event: &SplitViewEvent, cx| {
            app.panes.split_view.detached.apply_split_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(
        &self,
        kind: SplitViewDemoKind,
        look: &ShadcnLook,
        toggles: &InspectorToggleRegistry,
    ) -> AnyElement {
        match kind {
            SplitViewDemoKind::Unified => self.render_unified(look, toggles),
            SplitViewDemoKind::Inset => self.render_inset(look, toggles),
            SplitViewDemoKind::IconRail => self.render_icon_rail(look, toggles),
            SplitViewDemoKind::Detached => self.render_detached(look, toggles),
        }
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.unified.split_view, cx);
        notify_entity(&self.unified.separator_switch, cx);
        notify_entity(&self.inset.split_view, cx);
        notify_entity(&self.inset.separator_switch, cx);
        notify_entity(&self.icon_rail.split_view, cx);
        notify_entity(&self.icon_rail.separator_switch, cx);
        notify_entity(&self.icon_rail.toggle_button, cx);
        notify_entity(&self.detached.split_view, cx);
        notify_entity(&self.detached.separator_switch, cx);
        notify_entity(&self.detached.toggle_button, cx);
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }

    fn spawn_demo(
        look: Arc<ShadcnLook>,
        id: impl Into<SharedString>,
        sidebar_width: gpui::Pixels,
        sidebar_min_width: gpui::Pixels,
        sidebar_collapsed_width: gpui::Pixels,
        separator_visibility: SplitViewSeparatorVisibility,
        sidebar: impl Fn() -> AnyElement + 'static,
        content: impl Fn() -> AnyElement + 'static,
        cx: &mut Context<GalleryApp>,
    ) -> DemoState {
        let id = id.into();
        let switch_id = format!("{id}-separator-visibility");
        let split_view = look
            .split_view(id.clone())
            .sidebar_width(sidebar_width)
            .sidebar_min_width(sidebar_min_width)
            .sidebar_collapsed_width(sidebar_collapsed_width)
            .separator_visibility(separator_visibility)
            .sidebar(sidebar)
            .content(content)
            .spawn(cx);
        let read_visibility = split_view.read(cx).separator_visibility();
        let read_sidebar_width = split_view.read(cx).sidebar_width().as_f32();
        DemoState {
            split_view,
            separator_switch: separator_switch(&look, &switch_id, read_visibility, cx),
            sidebar_width: read_sidebar_width,
            separator_visibility: read_visibility,
        }
    }

    fn subscribe_demo(
        demo: &DemoState,
        cx: &mut Context<GalleryApp>,
        subscriptions: &mut Vec<Subscription>,
        handler: fn(&mut GalleryApp, &ButtonEvent, &mut Context<GalleryApp>),
    ) {
        let switch = demo.separator_switch.clone();
        subscriptions.push(cx.subscribe(&switch, move |app, _, event: &ButtonEvent, cx| {
            handler(app, event, cx);
        }));
    }

    fn render_unified(&self, look: &ShadcnLook, toggles: &InspectorToggleRegistry) -> AnyElement {
        let canvas_width = px(DEMO_WIDTH);
        let canvas_height = px(DEMO_HEIGHT);

        gallery_pane_with_inspector_description(
            "split-view-unified",
            "Split View: Unified",
            Some("Baseline unified split view with fixed nav pane width in a shared frame."),
            self.demo_body(
                look,
                demo_frame(
                    "Unified",
                    mockup_shell(
                        sized_split_view(self.unified.split_view.clone(), canvas_width, canvas_height),
                        canvas_width,
                        canvas_height,
                    ),
                ),
                self.unified.separator_switch.clone(),
                telemetry_rows(
                    look,
                    "unified",
                    self.unified.separator_visibility,
                    self.unified.sidebar_width,
                    None,
                    None,
                    None,
                ),
            ),
            self.inspector.clone(),
            toggles,
            look,
        )
    }

    fn render_inset(&self, look: &ShadcnLook, toggles: &InspectorToggleRegistry) -> AnyElement {
        let shell_pad = px(10.0);
        let inset_pad = px(10.0);
        let shell_width = px(DEMO_WIDTH);
        let split_height = px(DEMO_HEIGHT);
        let sample_height = split_height + inset_pad * 2.0;
        let shell_height = sample_height + shell_pad * 2.0;
        let sample_width = (shell_width - shell_pad * 2.0).max(px(1.0));
        let split_width = (sample_width - inset_pad * 2.0).max(px(1.0));

        gallery_pane_with_inspector_description(
            "split-view-inset",
            "Split View: Layered Inset",
            Some("Layered inset split view with nav and content panes in one frame."),
            self.demo_body(
                look,
                demo_frame(
                    "Layered Inset",
                    mockup_shell(
                        div().size_full().p(shell_pad).child(
                            div().size_full().rounded(px(16.0)).bg(rgb(0x242835)).p(inset_pad).child(sized_split_view(
                                self.inset.split_view.clone(),
                                split_width,
                                split_height,
                            )),
                        ),
                        shell_width,
                        shell_height,
                    ),
                ),
                self.inset.separator_switch.clone(),
                telemetry_rows(
                    look,
                    "layered_inset",
                    self.inset.separator_visibility,
                    self.inset.sidebar_width,
                    None,
                    None,
                    None,
                ),
            ),
            self.inspector.clone(),
            toggles,
            look,
        )
    }

    fn render_icon_rail(&self, look: &ShadcnLook, toggles: &InspectorToggleRegistry) -> AnyElement {
        let canvas_width = px(DEMO_WIDTH);
        let canvas_height = px(DEMO_HEIGHT);

        gallery_pane_with_inspector_description(
            "split-view-icon-rail",
            "Split View: Icon Rail",
            Some("Split view with collapse-to-icon-rail behavior."),
            self.demo_body(
                look,
                demo_frame(
                    "Icon Rail",
                    mockup_shell(
                        sized_split_view(self.icon_rail.split_view.clone(), canvas_width, canvas_height),
                        canvas_width,
                        canvas_height,
                    ),
                ),
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .child(self.icon_rail.toggle_button.clone())
                    .child(self.icon_rail.separator_switch.clone())
                    .into_any_element(),
                telemetry_rows(
                    look,
                    "icon_rail",
                    self.icon_rail.separator_visibility,
                    self.icon_rail.sidebar_width,
                    Some(self.icon_rail.collapsed),
                    Some(self.icon_rail.effective_width),
                    Some(self.icon_rail.last_event.clone()),
                ),
            ),
            self.inspector.clone(),
            toggles,
            look,
        )
    }

    fn render_detached(&self, look: &ShadcnLook, toggles: &InspectorToggleRegistry) -> AnyElement {
        let canvas_width = px(DEMO_WIDTH);
        let canvas_height = px(DEMO_HEIGHT);

        gallery_pane_with_inspector_description(
            "split-view-detached",
            "Split View: Detached",
            Some("Detached nav pane and independent content pane surface."),
            self.demo_body(
                look,
                demo_frame(
                    "Detached",
                    mockup_shell(
                        div().size_full().p(px(14.0)).child(sized_split_view(
                            self.detached.split_view.clone(),
                            (canvas_width - px(28.0)).max(px(1.0)),
                            (canvas_height - px(28.0)).max(px(1.0)),
                        )),
                        canvas_width,
                        canvas_height,
                    ),
                ),
                self.detached.separator_switch.clone(),
                telemetry_rows(
                    look,
                    "detached",
                    self.detached.separator_visibility,
                    self.detached.sidebar_width,
                    None,
                    None,
                    None,
                ),
            ),
            self.inspector.clone(),
            toggles,
            look,
        )
    }

    fn demo_body(
        &self,
        _look: &ShadcnLook,
        preview: AnyElement,
        actions: impl IntoElement,
        telemetry: impl IntoElement,
    ) -> AnyElement {
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_6()
            .pb_4()
            .child(preview)
            .child(div().flex().flex_wrap().items_center().gap_3().child(actions))
            .child(telemetry)
            .into_any_element()
    }
}

impl DemoState {
    fn apply_separator_toggle(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        apply_separator_visibility_toggle(&self.split_view, &self.separator_switch, event, cx);
        let read = self.split_view.read(cx);
        self.separator_visibility = read.separator_visibility();
        cx.notify();
    }

    fn apply_split_event(&mut self, event: &SplitViewEvent, cx: &mut Context<GalleryApp>) {
        if let SplitViewEvent::SidebarWidthChanged { width } | SplitViewEvent::ResizeEnd { width } = event {
            self.sidebar_width = width.as_f32();
            cx.notify();
        }
    }
}

impl IconRailDemo {
    fn apply_separator_toggle(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        apply_separator_visibility_toggle(&self.split_view, &self.separator_switch, event, cx);
        self.separator_visibility = self.split_view.read(cx).separator_visibility();
        cx.notify();
    }

    fn apply_split_event(&mut self, event: &SplitViewEvent, cx: &mut Context<GalleryApp>) {
        let read = self.split_view.read(cx);
        self.sidebar_width = read.sidebar_width().as_f32();
        self.effective_width = read.effective_sidebar_width().as_f32();
        self.collapsed = read.collapsed();
        self.last_event = match event {
            SplitViewEvent::CollapsedChanged { collapsed } => format!("CollapsedChanged: {collapsed}").into(),
            SplitViewEvent::SidebarWidthChanged { width } => {
                format!("SidebarWidthChanged: {:.0}px", width.as_f32()).into()
            }
            SplitViewEvent::ResizeStart | SplitViewEvent::ResizeEnd { .. } => self.last_event.clone(),
        };
        cx.notify();
    }
}

impl DetachedDemo {
    fn apply_separator_toggle(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        apply_separator_visibility_toggle(&self.split_view, &self.separator_switch, event, cx);
        self.separator_visibility = self.split_view.read(cx).separator_visibility();
        cx.notify();
    }

    fn apply_split_event(&mut self, event: &SplitViewEvent, cx: &mut Context<GalleryApp>) {
        if let SplitViewEvent::SidebarWidthChanged { width } | SplitViewEvent::ResizeEnd { width } = event {
            self.sidebar_width = width.as_f32();
            cx.notify();
        }
    }
}

fn sized_split_view(split_view: Entity<SplitView>, width: gpui::Pixels, height: gpui::Pixels) -> AnyElement {
    div().w(width).h(height).child(split_view).into_any_element()
}

fn telemetry_rows(
    look: &ShadcnLook,
    variant: &str,
    separator_visibility: SplitViewSeparatorVisibility,
    sidebar_width: f32,
    collapsed: Option<bool>,
    effective_width: Option<f32>,
    last_event: Option<SharedString>,
) -> AnyElement {
    let chrome = look.chrome();
    let mut rows = div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().text_sm().text_color(chrome.title_text).child("Telemetry"))
        .child(telemetry_row(chrome.muted_text, "Variant", variant))
        .child(telemetry_row(
            chrome.muted_text,
            "Separator visibility",
            separator_visibility_label(separator_visibility),
        ))
        .child(telemetry_row(chrome.muted_text, "Sidebar width", &format!("{sidebar_width:.0}px")));

    if let Some(collapsed) = collapsed {
        rows = rows.child(telemetry_row(chrome.muted_text, "Collapsed", &collapsed.to_string()));
    }
    if let Some(effective_width) = effective_width {
        rows = rows.child(telemetry_row(chrome.muted_text, "Effective width", &format!("{effective_width:.0}px")));
        rows = rows.child(telemetry_row(
            chrome.muted_text,
            "Collapsed width",
            &format!("{ICON_RAIL_COLLAPSED_WIDTH:.0}px"),
        ));
    }
    if let Some(last_event) = last_event {
        rows = rows.child(telemetry_row(chrome.muted_text, "Last event", last_event.as_ref()));
    }

    rows.into_any_element()
}

fn telemetry_row(color: gpui::Hsla, label: &str, value: &str) -> impl IntoElement {
    div().text_size(px(12.0)).text_color(color).child(format!("{label}: {value}"))
}

fn icon_rail_nav_pane() -> AnyElement {
    div()
        .size_full()
        .flex()
        .flex_col()
        .p_3()
        .bg(rgb(0x242835))
        .overflow_hidden()
        .child(
            div()
                .h(px(34.0))
                .w_full()
                .px_1()
                .rounded(px(8.0))
                .flex()
                .items_center()
                .gap_3()
                .overflow_hidden()
                .child(
                    div()
                        .w(px(32.0))
                        .flex_none()
                        .flex()
                        .justify_center()
                        .font_family("lucide")
                        .font_weight(FontWeight::NORMAL)
                        .text_size(px(20.0))
                        .line_height(px(20.0))
                        .child(char::from(LucideIcon::Menu).to_string()),
                ),
        )
        .into_any_element()
}

fn detached_nav_pane() -> AnyElement {
    div()
        .size_full()
        .p(px(10.0))
        .child(div().size_full().rounded(px(16.0)).bg(rgb(0x242835)))
        .into_any_element()
}

fn detached_content_pane(toggle_button: IconButton) -> AnyElement {
    content_pane_with_panel(
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(div().w_full().flex().items_center().child(toggle_button))
            .child(div().flex_1().mt_2().bg(rgb(0x800080)).rounded_md()),
    )
}
