use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, FontWeight, IntoElement, Render, SharedString, Window, div, prelude::*, px};
use gpui_luma::controls::accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};
use gpui_luma::controls::tabs_navigation::TabsNavigation;
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::studio::style::shared::shell::section_shell_with_width;

pub(crate) struct AccordionPreview {
    look: Arc<ShadcnLook>,
    sm: Accordion,
    md: Accordion,
    lg: Accordion,
    template: Accordion,
}

impl AccordionPreview {
    pub(crate) fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        Self {
            sm: sample_accordion(&look, "theme-studio-accordion-sm", ControlSize::Sm, cx),
            md: sample_accordion(&look, "theme-studio-accordion-md", ControlSize::Md, cx),
            lg: sample_accordion(&look, "theme-studio-accordion-lg", ControlSize::Lg, cx),
            template: sample_accordion(&look, "theme-studio-accordion-template", ControlSize::Md, cx),
            look,
        }
    }

    pub(crate) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        let template = look.accordion_template();
        for accordion in [&self.sm, &self.md, &self.lg, &self.template] {
            let template = template.clone();
            accordion.update(cx, move |accordion, cx| accordion.set_template(template, cx));
        }
        cx.notify();
    }
}

impl Render for AccordionPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

pub(crate) fn render_accordion_template_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<TabsNavigation>,
    preview: Entity<AccordionPreview>,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let active_tab =
        preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("template-preview"));
    let preview = preview.read(cx);

    section_shell_with_width(
        960.0,
        "Accordion",
        "Collapsible triggers. Sizes tab: Sm/Md/Lg trigger label typography.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        div()
            .w_full()
            .flex()
            .flex_col()
            .child(div().w_full().flex().justify_start().child(preview_tabs))
            .child(div().w_full().h(px(1.0)).bg(chrome.border))
            .child(div().w_full().flex().justify_center().mt(px(16.0)).child(match active_tab.as_ref() {
                "sizes" => render_sizes_body(preview, chrome.muted_text),
                _ => render_template_body(preview, chrome.muted_text),
            }))
            .into_any_element(),
    )
}

fn render_template_body(preview: &AccordionPreview, muted: gpui::Hsla) -> AnyElement {
    size_column("Template", muted, preview.template.clone(), 280.0)
}

fn render_sizes_body(preview: &AccordionPreview, muted: gpui::Hsla) -> AnyElement {
    div()
        .flex()
        .flex_wrap()
        .items_start()
        .gap(px(16.0))
        .child(size_column("Sm", muted, preview.sm.clone(), 240.0))
        .child(size_column("Md", muted, preview.md.clone(), 240.0))
        .child(size_column("Lg", muted, preview.lg.clone(), 240.0))
        .into_any_element()
}

fn size_column(label: &'static str, muted: gpui::Hsla, accordion: Accordion, width: f32) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .w(px(width))
        .child(
            div()
                .text_size(px(11.0))
                .line_height(px(14.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(muted)
                .child(label),
        )
        .child(accordion)
        .into_any_element()
}

fn sample_accordion(
    look: &Arc<ShadcnLook>,
    id: impl Into<SharedString>,
    size: ControlSize,
    cx: &mut impl gpui::AppContext,
) -> Accordion {
    let id = id.into();
    look.accordion(id.clone())
        .single()
        .size(size)
        .item(demo_item(format!("{id}-account"), "Account", LucideIcon::User, "Profile and security.").expanded(true))
        .item(demo_item(format!("{id}-billing"), "Billing", LucideIcon::CreditCard, "Plan and invoices."))
        .item(demo_item(format!("{id}-team"), "Team", LucideIcon::Users, "Members and roles."))
        .spawn(cx)
}

fn demo_item(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    icon: LucideIcon,
    body: &'static str,
) -> AccordionItem {
    AccordionItem::new(
        id,
        AccordionTrigger::new(label).icon(icon),
        AccordionContent::custom(move |_, _| div().text_sm().child(body).into_any_element()),
    )
}
