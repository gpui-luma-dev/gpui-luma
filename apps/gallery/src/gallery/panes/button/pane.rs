#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, EventEmitter, FocusHandle, Focusable, FontWeight, IntoElement, MouseDownEvent,
    MouseUpEvent, Render, SharedString, Subscription, Window, div, prelude::*, px,
};
use gpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize, default_button_family_theme};
use gpui_luma::controls::command::button::{
    Button, ButtonEvent, ButtonRenderModel, ButtonTemplate, DefaultButtonTemplate, HasPresenter,
    default_button_template,
};
use gpui_luma::controls::command::icon_button::IconButton;
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnButtonStyle, ShadcnLook, ShadcnTextRole};
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use gpui_luma::controls::slide_panel::{
    SlidePanelEdge, SlidePanelOverlayHandlers, SlidePanelResizeDrag, SlidePanelResizeHandlers, SlidePanelSizeConfig,
    SlidePanelState, SlidePanelTopAnchor, render_slide_panel_inset,
};
use gpui_luma_look_shadcn::slide_panel_panels_look;
use super::super::shared::InspectorToggleRegistry;
use super::labeling::render_vertical_section_rail;
use super::theme_inspector::ThemeInspector;

const BUTTON_INSPECTOR_PANEL_WIDTH: f32 = 620.0;
const BUTTON_INSPECTOR_MIN_WIDTH: f32 = 420.0;
const BUTTON_INSPECTOR_MAX_WIDTH: f32 = 860.0;
const PANE_PADDING: f32 = 28.0;

#[derive(Clone, Debug)]
enum ButtonPaneViewEvent {
    InspectorDismissed,
}

#[derive(Clone)]
pub(in crate::gallery) struct ButtonPane {
    view: Entity<ButtonPaneView>,
}

struct ButtonPaneView {
    look: Arc<ShadcnLook>,
    inspector_toggle: IconButton,
    inspector_close: IconButton,
    secondary_button: Entity<Button>,
    outline_button: Entity<Button>,
    ghost_button: Entity<Button>,
    primary_button: Entity<Button>,
    state_preview: Entity<ButtonStatePreview>,
    inspector: Entity<ThemeInspector>,
    slide_state: SlidePanelState,
    inspector_visible: bool,
    secondary_clicks: usize,
    outline_clicks: usize,
    ghost_clicks: usize,
    primary_clicks: usize,
}

impl EventEmitter<ButtonPaneViewEvent> for ButtonPaneView {}

impl ButtonPane {
    pub(in crate::gallery) fn new(
        cx: &mut Context<GalleryApp>,
        look: Arc<ShadcnLook>,
        inspector_toggle: IconButton,
    ) -> Self {
        Self { view: cx.new(|cx| ButtonPaneView::new(look, inspector_toggle, cx)) }
    }

    pub(in crate::gallery) fn sync_inspector_visibility(
        &self,
        visible: bool,
        opener: FocusHandle,
        cx: &mut Context<GalleryApp>,
    ) {
        self.view.update(cx, |view, cx| {
            view.set_inspector_visibility(visible, opener, cx);
        });
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        let view = self.view.clone();
        let (secondary_button, outline_button, ghost_button, primary_button, inspector_close) = {
            let view = self.view.read(cx);
            (
                view.secondary_button.clone(),
                view.outline_button.clone(),
                view.ghost_button.clone(),
                view.primary_button.clone(),
                view.inspector_close.clone(),
            )
        };

        subscriptions.push(cx.subscribe(&view, |app, _, event: &ButtonPaneViewEvent, cx| {
            if matches!(event, ButtonPaneViewEvent::InspectorDismissed) {
                app.panes.inspector_toggles.set_visible("button", false);
                cx.notify();
            }
        }));
        subscriptions.push(cx.subscribe(&secondary_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.button.handle_secondary_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&outline_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.button.handle_outline_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&ghost_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.button.handle_ghost_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&primary_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.button.handle_primary_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&inspector_close, |app, _, event: &ButtonEvent, cx| {
            if !event.is_click() {
                return;
            }
            let toggle = app.panes.inspector_toggles.get("button").toggle.clone();
            let focus = toggle.read(cx).focus_handle(cx);
            app.panes.inspector_toggles.set_visible("button", false);
            app.panes.button.sync_inspector_visibility(false, focus, cx);
            cx.notify();
        }));
    }

    pub(in crate::gallery) fn render(&self, _look: &ShadcnLook, _toggles: &InspectorToggleRegistry) -> AnyElement {
        self.view.clone().into_any_element()
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.view.update(cx, |view, cx| {
            view.notify_controls(cx);
            cx.notify();
        });
    }

    fn handle_secondary_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        self.view.update(cx, |view, cx| view.handle_secondary_event(event, cx));
    }

    fn handle_outline_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        self.view.update(cx, |view, cx| view.handle_outline_event(event, cx));
    }

    fn handle_ghost_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        self.view.update(cx, |view, cx| view.handle_ghost_event(event, cx));
    }

    fn handle_primary_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        self.view.update(cx, |view, cx| view.handle_primary_event(event, cx));
    }
}

impl ButtonPaneView {
    fn new(look: Arc<ShadcnLook>, inspector_toggle: IconButton, cx: &mut Context<Self>) -> Self {
        let secondary_button = look.secondary_button("button-secondary-example").label("Secondary").spawn(cx);
        let outline_button = look.outline_button("button-outline-example").label("Outline").spawn(cx);
        let ghost_button = look.ghost_button("button-ghost-example").label("Ghost").spawn(cx);
        let primary_button = look.primary_button("button-primary-example").label("Primary").spawn(cx);
        let state_preview = cx.new(|_| ButtonStatePreview::new(look.clone()));
        let inspector_close = look.ghost_icon_button("button-inspector-close", LucideIcon::X).spawn(cx);
        let inspector = cx.new(|cx| ThemeInspector::for_side_panel(look.clone(), inspector_close.clone(), cx));

        let mut slide_state = SlidePanelState::new(
            SlidePanelTopAnchor::WindowEdge,
            SlidePanelSizeConfig::new(
                BUTTON_INSPECTOR_PANEL_WIDTH,
                BUTTON_INSPECTOR_MIN_WIDTH,
                BUTTON_INSPECTOR_MAX_WIDTH,
            ),
        );
        slide_state.set_backdrop_click_closes(false);

        Self {
            look,
            inspector_toggle,
            inspector_close,
            secondary_button,
            outline_button,
            ghost_button,
            primary_button,
            state_preview,
            inspector,
            slide_state,
            inspector_visible: false,
            secondary_clicks: 0,
            outline_clicks: 0,
            ghost_clicks: 0,
            primary_clicks: 0,
        }
    }

    fn set_inspector_visibility(&mut self, visible: bool, opener: FocusHandle, cx: &mut Context<Self>) {
        if self.inspector_visible == visible {
            if visible && self.slide_state.active_edge().is_none() {
                self.slide_state.open(SlidePanelEdge::Right, opener);
                cx.notify();
            }
            return;
        }

        self.inspector_visible = visible;
        if visible {
            self.slide_state.open(SlidePanelEdge::Right, opener);
        } else {
            self.slide_state.request_close();
        }
        cx.notify();
    }

    fn notify_controls(&self, cx: &mut Context<Self>) {
        self.secondary_button.update(cx, |_, cx| cx.notify());
        self.outline_button.update(cx, |_, cx| cx.notify());
        self.ghost_button.update(cx, |_, cx| cx.notify());
        self.primary_button.update(cx, |_, cx| cx.notify());
        self.state_preview.update(cx, |_, cx| cx.notify());
        self.inspector.update(cx, |_, cx| cx.notify());
    }

    fn handle_secondary_event(&mut self, event: &ButtonEvent, cx: &mut Context<Self>) {
        if !event.is_click() {
            return;
        }

        self.secondary_clicks += 1;
        let label = format!("Secondary {}", self.secondary_clicks);
        self.secondary_button.update(cx, |button, cx| {
            button.set_label(label, cx);
        });
    }

    fn handle_outline_event(&mut self, event: &ButtonEvent, cx: &mut Context<Self>) {
        if !event.is_click() {
            return;
        }

        self.outline_clicks += 1;
        let label = format!("Outline {}", self.outline_clicks);
        self.outline_button.update(cx, |button, cx| {
            button.set_label(label, cx);
        });
    }

    fn handle_ghost_event(&mut self, event: &ButtonEvent, cx: &mut Context<Self>) {
        if !event.is_click() {
            return;
        }

        self.ghost_clicks += 1;
        let label = format!("Ghost {}", self.ghost_clicks);
        self.ghost_button.update(cx, |button, cx| {
            button.set_label(label, cx);
        });
    }

    fn handle_primary_event(&mut self, event: &ButtonEvent, cx: &mut Context<Self>) {
        if !event.is_click() {
            return;
        }

        self.primary_clicks += 1;
        let label = format!("Primary {}", self.primary_clicks);
        self.primary_button.update(cx, |button, cx| {
            button.set_label(label, cx);
        });
    }

    fn handle_overlay_key_down(&mut self, event: &gpui::KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if event.keystroke.key.as_str() == "escape" && self.slide_state.handle_escape(window, cx) {
            self.inspector_visible = false;
            cx.emit(ButtonPaneViewEvent::InspectorDismissed);
            cx.notify();
        }
    }

    fn handle_resize_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let Some(edge) = self.slide_state.active_edge() else {
            return;
        };

        if self.slide_state.begin_resize(event.position, edge) {
            window.prevent_default();
            cx.stop_propagation();
            cx.notify();
        }
    }

    fn handle_resize_drag_move(
        &mut self,
        event: &gpui::DragMoveEvent<SlidePanelResizeDrag>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(edge) = self.slide_state.active_edge() else {
            return;
        };

        if self.slide_state.update_resize(event.event.position, edge) {
            cx.notify();
        }
    }

    fn handle_resize_mouse_up(&mut self, event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if event.button != gpui::MouseButton::Left {
            return;
        }

        if self.slide_state.finish_resize() {
            cx.notify();
        }
    }

    fn handle_resize_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.slide_state.set_resize_handle_hovered(*hovered) {
            cx.notify();
        }
    }
}

impl Render for ButtonPaneView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _ = self.slide_state.sync_animation();
        self.slide_state.schedule_animation_frame(window, cx);

        let opener_focus = self.inspector_toggle.read(cx).focus_handle(cx);
        if self.inspector_visible || self.slide_state.active_edge().is_some() {
            let close_focus = self.inspector_close.read(cx).focus_handle(cx);
            self.slide_state.schedule_pending_focus(window, cx, close_focus);
        } else {
            self.slide_state.schedule_pending_focus(window, cx, opener_focus);
        }

        let chrome = self.look.chrome();
        let title_style = self.look.typography_role(ShadcnTextRole::H3);
        let panel_open = self.inspector_visible || self.slide_state.active_edge().is_some();
        let showcase = render_button_showcase(
            self.primary_button.clone(),
            self.secondary_button.clone(),
            self.outline_button.clone(),
            self.ghost_button.clone(),
            self.state_preview.clone(),
        );

        let mut pane = div()
            .id("button-pane")
            .size_full()
            .relative()
            .overflow_hidden()
            .bg(chrome.content_background)
            .child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .flex_none()
                            .w_full()
                            .px(px(PANE_PADDING))
                            .pt(px(PANE_PADDING))
                            .pb(px(12.0))
                            .flex()
                            .items_start()
                            .gap(px(12.0))
                            .when(!panel_open, |header| header.child(self.inspector_toggle.clone()))
                            .child(
                                div()
                                    .typography_style(title_style)
                                    .text_color(chrome.title_text)
                                    .child("Command (Text)"),
                            ),
                    )
                    .child(
                        div()
                            .id("button-pane-body")
                            .flex_1()
                            .min_h(px(0.0))
                            .relative()
                            .child(render_centered_button_showcase(showcase)),
                    ),
            );

        if panel_open {
            let handlers = SlidePanelOverlayHandlers {
                key_down: Box::new(cx.listener(Self::handle_overlay_key_down)),
                backdrop_mouse_down: Box::new(|_, _, _| {}),
            };
            let resize_handlers = SlidePanelResizeHandlers {
                mouse_down: Box::new(cx.listener(Self::handle_resize_mouse_down)),
                drag_move: Box::new(cx.listener(Self::handle_resize_drag_move)),
                mouse_up: Box::new(cx.listener(Self::handle_resize_mouse_up)),
                mouse_up_out: Box::new(cx.listener(Self::handle_resize_mouse_up)),
                hover: Box::new(cx.listener(Self::handle_resize_hover)),
            };
            let inspector_panel = div().size_full().min_h(px(0.0)).min_w(px(0.0)).child(self.inspector.clone());
            let panel = render_slide_panel_inset(
                &slide_panel_panels_look(&self.look),
                &self.slide_state,
                inspector_panel.into_any_element(),
                chrome.panel_background,
                handlers,
                resize_handlers,
            );
            pane = pane.child(panel);
        }

        pane
    }
}

fn render_centered_button_showcase(content: AnyElement) -> AnyElement {
    div()
        .size_full()
        .min_w(px(0.0))
        .min_h(px(0.0))
        .px(px(PANE_PADDING))
        .pb(px(PANE_PADDING))
        .flex()
        .items_center()
        .justify_center()
        .child(div().relative().flex().flex_col().items_center().justify_center().gap_4().occlude().child(content))
        .into_any_element()
}

fn render_button_showcase(
    primary_button: Entity<Button>,
    secondary_button: Entity<Button>,
    outline_button: Entity<Button>,
    ghost_button: Entity<Button>,
    state_preview: Entity<ButtonStatePreview>,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_5()
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(12.0))
                .child(primary_button)
                .child(secondary_button)
                .child(outline_button)
                .child(ghost_button),
        )
        .child(state_preview)
        .into_any_element()
}

#[derive(Clone)]
struct ButtonStatePreview {
    look: Arc<ShadcnLook>,
    template: Arc<dyn ButtonTemplate<()>>,
    uniform_template: Arc<dyn ButtonTemplate<()>>,
    use_uniform_sizing: bool,
}

struct ButtonStateSample {
    id: &'static str,
    header: &'static str,
    state: InteractionState,
}

#[derive(Clone, Copy)]
enum ButtonTemplateVariant {
    TextButton,
    TextButtonLeadingIcon,
    TextButtonTrailingIcon,
    IconButton,
}

impl ButtonTemplateVariant {
    fn id(self) -> &'static str {
        match self {
            Self::TextButton => "text-button",
            Self::TextButtonLeadingIcon => "text-button-leading-icon",
            Self::TextButtonTrailingIcon => "text-button-trailing-icon",
            Self::IconButton => "icon-button",
        }
    }

    fn round(self) -> bool {
        matches!(self, Self::IconButton)
    }

    fn content(self) -> gpui_luma::controls::command::button::ControlPresenter<ButtonRenderModel<()>> {
        let label = SharedString::from("Button");
        match self {
            Self::TextButton => Arc::new(move |_, _| div().child(label.clone()).into_any_element()),
            Self::TextButtonLeadingIcon => Arc::new(move |_, _| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .child(render_lucide_icon(LucideIcon::Heart))
                    .child(label.clone())
                    .into_any_element()
            }),
            Self::TextButtonTrailingIcon => Arc::new(move |_, _| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .child(label.clone())
                    .child(render_lucide_icon(LucideIcon::ChevronDown))
                    .into_any_element()
            }),
            Self::IconButton => Arc::new(move |_, _| render_lucide_icon(LucideIcon::Heart)),
        }
    }
}

impl ButtonStatePreview {
    fn new(look: Arc<ShadcnLook>) -> Self {
        Self {
            look,
            template: default_button_template(),
            uniform_template: Arc::new(
                DefaultButtonTemplate::new(default_button_family_theme()).with_modifier(|element, _| element.w_full()),
            ),
            use_uniform_sizing: true,
        }
    }
}

impl Render for ButtonStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let samples = [
            ButtonStateSample { id: "default", header: "default", state: InteractionState::default() },
            ButtonStateSample {
                id: "hover",
                header: "hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
            },
            ButtonStateSample {
                id: "focused",
                header: "focused",
                state: InteractionState { focused: true, ..InteractionState::default() },
            },
            ButtonStateSample {
                id: "pressed",
                header: "pressed",
                state: InteractionState { hovered: true, pressed: true, ..InteractionState::default() },
            },
            ButtonStateSample {
                id: "disabled",
                header: "disabled",
                state: InteractionState { disabled: true, ..InteractionState::default() },
            },
        ];
        let variants = [
            ButtonTemplateVariant::TextButton,
            ButtonTemplateVariant::TextButtonLeadingIcon,
            ButtonTemplateVariant::TextButtonTrailingIcon,
            ButtonTemplateVariant::IconButton,
        ];

        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(chrome.muted_text)
                    .child("Template matrix preview"),
            )
            .child(div().flex().flex_col().items_start().gap(px(20.0)).children([
                render_section(
                    &self.template,
                    &self.uniform_template,
                    &self.look,
                    "Primary",
                    ShadcnButtonStyle::Primary,
                    &variants,
                    &samples,
                    self.use_uniform_sizing,
                    chrome.muted_text,
                    window,
                    cx,
                ),
                render_section(
                    &self.template,
                    &self.uniform_template,
                    &self.look,
                    "Secondary",
                    ShadcnButtonStyle::Secondary,
                    &variants,
                    &samples,
                    self.use_uniform_sizing,
                    chrome.muted_text,
                    window,
                    cx,
                ),
                render_section(
                    &self.template,
                    &self.uniform_template,
                    &self.look,
                    "Outline",
                    ShadcnButtonStyle::Outline,
                    &variants,
                    &samples,
                    self.use_uniform_sizing,
                    chrome.muted_text,
                    window,
                    cx,
                ),
                render_section(
                    &self.template,
                    &self.uniform_template,
                    &self.look,
                    "Ghost",
                    ShadcnButtonStyle::Ghost,
                    &variants,
                    &samples,
                    self.use_uniform_sizing,
                    chrome.muted_text,
                    window,
                    cx,
                ),
            ]))
    }
}

fn render_section(
    template: &Arc<dyn ButtonTemplate<()>>,
    uniform_template: &Arc<dyn ButtonTemplate<()>>,
    look: &Arc<ShadcnLook>,
    section_label: &'static str,
    style: ShadcnButtonStyle,
    variants: &[ButtonTemplateVariant],
    samples: &[ButtonStateSample],
    use_uniform_sizing: bool,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .items_start()
        .gap(px(8.0))
        .child(render_vertical_section_rail(section_label, label_color))
        .child(
            div()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(8.0))
                .child(render_header_row(samples, label_color))
                .children(variants.iter().map(|variant| {
                    render_variant_row(
                        template,
                        uniform_template,
                        look,
                        style,
                        *variant,
                        samples,
                        use_uniform_sizing,
                        window,
                        cx,
                    )
                })),
        )
        .into_any_element()
}

fn render_header_row(samples: &[ButtonStateSample], label_color: gpui::Hsla) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .children(samples.iter().map(|sample| {
            div()
                .w(px(116.0))
                .flex()
                .justify_center()
                .text_xs()
                .line_height(px(15.0))
                .text_color(label_color)
                .child(sample.header)
        }))
        .into_any_element()
}

fn render_variant_row(
    template: &Arc<dyn ButtonTemplate<()>>,
    uniform_template: &Arc<dyn ButtonTemplate<()>>,
    look: &Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
    variant: ButtonTemplateVariant,
    samples: &[ButtonStateSample],
    use_uniform_sizing: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .children(samples.iter().map(|sample| {
            render_state_sample(
                template,
                uniform_template,
                look,
                style,
                variant,
                sample,
                use_uniform_sizing,
                window,
                cx,
            )
        }))
        .into_any_element()
}

fn render_state_sample(
    template: &Arc<dyn ButtonTemplate<()>>,
    uniform_template: &Arc<dyn ButtonTemplate<()>>,
    look: &Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
    variant: ButtonTemplateVariant,
    sample: &ButtonStateSample,
    use_uniform_sizing: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("button-preview-{}-{}-{}", shadcn_style_id(style), variant.id(), sample.id));
    let look = look_for_style(look.clone(), style);
    let model = ButtonRenderModel {
        id,
        data: (),
        content: variant.content(),
        role: if matches!(variant, ButtonTemplateVariant::IconButton) {
            ButtonFamilyRole::Icon
        } else {
            ButtonFamilyRole::Text
        },
        size: ButtonSize::Md,
        state: sample.state,
        round: variant.round(),
        radius_override: std::cell::Cell::new(None),
        elevation: true,
        compact: false,
        look: Some(look),
        ..Default::default()
    };

    let active_template = if use_uniform_sizing && !matches!(variant, ButtonTemplateVariant::IconButton) {
        uniform_template
    } else {
        template
    };

    let rendered = active_template.render(&model, window, cx);
    let rendered = if use_uniform_sizing && !matches!(variant, ButtonTemplateVariant::IconButton) {
        rendered.w_full()
    } else {
        rendered
    };

    div().w(px(116.0)).flex().justify_center().items_center().child(rendered).into_any_element()
}

fn look_for_style(
    theme: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
) -> gpui_luma::controls::command::button::ButtonLookSource<()> {
    Arc::new(move |model| match style {
        ShadcnButtonStyle::Primary => theme.as_ref().resolve_primary_button(model.role, model.size, model.state),
        ShadcnButtonStyle::Secondary => theme.as_ref().resolve_secondary_button(model.role, model.size, model.state),
        ShadcnButtonStyle::Outline => theme.as_ref().resolve_outline_button(model.role, model.size, model.state),
        ShadcnButtonStyle::Ghost => theme.as_ref().resolve_ghost_button(model.role, model.size, model.state),
        ShadcnButtonStyle::ContentOnly => {
            theme.as_ref().resolve_content_only_button(model.role, model.size, model.state)
        }
    })
}

fn shadcn_style_id(style: ShadcnButtonStyle) -> &'static str {
    match style {
        ShadcnButtonStyle::Primary => "primary",
        ShadcnButtonStyle::Secondary => "secondary",
        ShadcnButtonStyle::Outline => "outline",
        ShadcnButtonStyle::Ghost => "ghost",
        ShadcnButtonStyle::ContentOnly => "content-only",
    }
}

fn render_lucide_icon(icon: LucideIcon) -> AnyElement {
    div()
        .font_family("lucide")
        .text_size(px(16.0))
        .line_height(px(16.0))
        .child(char::from(icon).to_string())
        .into_any_element()
}
