use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{
    Button, ButtonEvent, ButtonRenderModel, ButtonTemplate, DefaultButtonTemplate, HasContent,
};
use gpui_luma::controls::button_family::{ButtonKind, ButtonSize};
use gpui_luma::theme::{ButtonFamilyRole, InteractionState};
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage, notify_entity};
use super::labeling::render_vertical_section_rail;

#[derive(Clone)]
pub(in crate::gallery) struct ButtonPane {
    default_button: Entity<Button>,
    ghost_button: Entity<Button>,
    prominent_button: Entity<Button>,
    state_preview: Entity<ButtonStatePreview>,
    default_clicks: usize,
    ghost_clicks: usize,
    prominent_clicks: usize,
}

impl ButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            default_button: Button::new("button-default-example")
                .label("Standard")
                .kind(ButtonKind::Standard)
                .spawn(cx),
            ghost_button: Button::new("button-ghost-example").label("Ghost").kind(ButtonKind::Ghost).spawn(cx),
            prominent_button: Button::new("button-prominent-example")
                .label("Prominent")
                .kind(ButtonKind::Prominent)
                .spawn(cx),
            state_preview: cx.new(|_| ButtonStatePreview::new(theme)),
            default_clicks: 0,
            ghost_clicks: 0,
            prominent_clicks: 0,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.default_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.button.handle_default_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.ghost_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.button.handle_ghost_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.prominent_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.button.handle_prominent_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        gallery_pane_with_usage(
            "Command (Text)",
            "Button",
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
                        .child(self.default_button.clone())
                        .child(self.ghost_button.clone())
                        .child(self.prominent_button.clone()),
                )
                .child(self.state_preview.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.default_button, cx);
        notify_entity(&self.ghost_button, cx);
        notify_entity(&self.prominent_button, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn handle_default_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                self.default_clicks += 1;
                let label = format!("Default {}", self.default_clicks);

                self.default_button.update(cx, |button, cx| {
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

    fn handle_prominent_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                self.prominent_clicks += 1;
                let label = format!("Prominent {}", self.prominent_clicks);

                self.prominent_button.update(cx, |button, cx| {
                    button.set_label(label, cx);
                });
            }
        }
    }
}

#[derive(Clone)]
struct ButtonStatePreview {
    theme: GalleryThemePack,
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

    fn content(self) -> gpui_luma::controls::command::button::ControlContent<ButtonRenderModel<()>> {
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
    fn new(theme: &GalleryThemePack) -> Self {
        Self {
            theme: theme.clone(),
            template: gpui_luma::controls::command::button::default_button_template(),
            uniform_template: Arc::new(
                DefaultButtonTemplate::new(theme.button_family_theme()).with_modifier(|element, _| element.w_full()),
            ),
            use_uniform_sizing: true,
        }
    }
}

impl Render for ButtonStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
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
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(chrome.muted_text)
                    .child("Template matrix preview"),
            )
            .child(div().flex().flex_col().items_start().gap(px(20.0)).children([
                render_section(
                    &self.template,
                    &self.uniform_template,
                    "Prominent",
                    ButtonKind::Prominent,
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
                    "Standard",
                    ButtonKind::Standard,
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
                    "Ghost",
                    ButtonKind::Ghost,
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
    section_label: &'static str,
    kind: ButtonKind,
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
                        kind,
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
                .text_size(px(11.0))
                .line_height(px(15.0))
                .text_color(label_color)
                .child(sample.header)
        }))
        .into_any_element()
}

fn render_variant_row(
    template: &Arc<dyn ButtonTemplate<()>>,
    uniform_template: &Arc<dyn ButtonTemplate<()>>,
    kind: ButtonKind,
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
            render_state_sample(template, uniform_template, kind, variant, sample, use_uniform_sizing, window, cx)
        }))
        .into_any_element()
}

fn render_state_sample(
    template: &Arc<dyn ButtonTemplate<()>>,
    uniform_template: &Arc<dyn ButtonTemplate<()>>,
    kind: ButtonKind,
    variant: ButtonTemplateVariant,
    sample: &ButtonStateSample,
    use_uniform_sizing: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("button-preview-{}-{}-{}", button_kind_id(kind), variant.id(), sample.id));
    let model = ButtonRenderModel {
        id,
        data: (),
        content: variant.content(),
        kind,
        role: if matches!(variant, ButtonTemplateVariant::IconButton) {
            ButtonFamilyRole::Icon
        } else {
            ButtonFamilyRole::Text
        },
        size: ButtonSize::Md,
        state: sample.state,
        round: variant.round(),
        radius_override: std::cell::Cell::new(None),
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

    div().w(px(116.0)).child(rendered).into_any_element()
}

fn button_kind_id(kind: ButtonKind) -> &'static str {
    match kind {
        ButtonKind::Prominent => "prominent",
        ButtonKind::Standard => "standard",
        ButtonKind::Ghost => "ghost",
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
