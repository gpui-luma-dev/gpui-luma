#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, FontWeight, IntoElement, Render, SharedString, Subscription, Window, div,
    prelude::*, px,
};
use gpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize, default_button_family_theme};
use gpui_luma::controls::command::button::{
    Button, ButtonEvent, ButtonRenderModel, ButtonTemplate, DefaultButtonTemplate, HasPresenter,
    default_button_template,
};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma::theme::{InteractionState};
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::inspector_shell::ButtonInspectorShell;
use super::inspector_tree::spawn_button_inspector_tree;
use super::super::shared::{gallery_pane_with_inspector, notify_entity, InspectorToggleRegistry};
use super::labeling::render_vertical_section_rail;

#[derive(Clone)]
pub(in crate::gallery) struct ButtonPane {
    secondary_button: Entity<Button>,
    outline_button: Entity<Button>,
    ghost_button: Entity<Button>,
    primary_button: Entity<Button>,
    state_preview: Entity<ButtonStatePreview>,
    inspector: Entity<ButtonInspectorShell>,
    secondary_clicks: usize,
    outline_clicks: usize,
    ghost_clicks: usize,
    primary_clicks: usize,
}

impl ButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let tree = spawn_button_inspector_tree(look.clone(), cx);
        let inspector = cx.new(|cx| ButtonInspectorShell::new(look.clone(), tree, cx));

        Self {
            secondary_button: look.secondary_button("button-secondary-example").label("Secondary").spawn(cx),
            outline_button: look.outline_button("button-outline-example").label("Outline").spawn(cx),
            ghost_button: look.ghost_button("button-ghost-example").label("Ghost").spawn(cx),
            primary_button: look.primary_button("button-primary-example").label("Primary").spawn(cx),
            state_preview: cx.new(|_| ButtonStatePreview::new(look)),
            inspector,
            secondary_clicks: 0,
            outline_clicks: 0,
            ghost_clicks: 0,
            primary_clicks: 0,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.secondary_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.button.handle_secondary_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.outline_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.button.handle_outline_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.ghost_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.button.handle_ghost_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.primary_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.button.handle_primary_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook, toggles: &InspectorToggleRegistry) -> AnyElement {
        gallery_pane_with_inspector(
            "button",
            "Command (Text)",
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
                        .child(self.primary_button.clone())
                        .child(self.secondary_button.clone())
                        .child(self.outline_button.clone())
                        .child(self.ghost_button.clone()),
                )
                .child(self.state_preview.clone())
                .into_any_element(),
            self.inspector.clone(),
            toggles,
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.secondary_button, cx);
        notify_entity(&self.outline_button, cx);
        notify_entity(&self.ghost_button, cx);
        notify_entity(&self.primary_button, cx);
        notify_entity(&self.state_preview, cx);
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        let detail = self.inspector.read(cx).detail();
        notify_entity(&detail, cx);
        detail.update(cx, |detail, cx| detail.notify_preview_buttons(cx));
        notify_entity(&self.inspector.read(cx).split(), cx);
    }

    fn handle_secondary_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                self.secondary_clicks += 1;
                let label = format!("Secondary {}", self.secondary_clicks);
                self.secondary_button.update(cx, |button, cx| {
                    button.set_label(label, cx);
                });
            }
        }
    }

    fn handle_outline_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                self.outline_clicks += 1;
                let label = format!("Outline {}", self.outline_clicks);
                self.outline_button.update(cx, |button, cx| {
                    button.set_label(label, cx);
                });
            }
        }
    }

    fn handle_ghost_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                self.ghost_clicks += 1;
                let label = format!("Ghost {}", self.ghost_clicks);
                self.ghost_button.update(cx, |button, cx| {
                    button.set_label(label, cx);
                });
            }
        }
    }

    fn handle_primary_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                self.primary_clicks += 1;
                let label = format!("Primary {}", self.primary_clicks);
                self.primary_button.update(cx, |button, cx| {
                    button.set_label(label, cx);
                });
            }
        }
    }
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
