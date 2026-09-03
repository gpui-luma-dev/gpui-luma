use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, FontWeight, IntoElement, Render, SharedString, Window, div, prelude::*, px};
use luma::controls::listbox::{ListBox, ListBoxItem};
use luma::controls::tabs::Tabs;
use luma::theme::ControlSize;
use luma_look_shadcn::{ShadcnLook, ShadcnLookControlExt};

use crate::studio::style::shared::shell::section_shell_with_width;

pub(crate) struct ListboxPreview {
    look: Arc<ShadcnLook>,
    sm: ListBox,
    md: ListBox,
    lg: ListBox,
    template: ListBox,
}

impl ListboxPreview {
    pub(crate) fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        Self {
            sm: fruit_listbox(&look, "luma-studio-listbox-sm", ControlSize::Sm, cx),
            md: fruit_listbox(&look, "luma-studio-listbox-md", ControlSize::Md, cx),
            lg: fruit_listbox(&look, "luma-studio-listbox-lg", ControlSize::Lg, cx),
            template: fruit_listbox(&look, "luma-studio-listbox-template", ControlSize::Md, cx),
            look,
        }
    }

    pub(crate) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for (listbox, size) in [
            (&self.sm, ControlSize::Sm),
            (&self.md, ControlSize::Md),
            (&self.lg, ControlSize::Lg),
            (&self.template, ControlSize::Md),
        ] {
            let template = look.listbox_template_for_size(size);
            listbox.update(cx, move |listbox, cx| listbox.set_template(template, cx));
        }
        cx.notify();
    }
}

impl Render for ListboxPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

pub(crate) fn render_listbox_template_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    preview: Entity<ListboxPreview>,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let active_tab =
        preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("template-preview"));
    let preview = preview.read(cx);

    section_shell_with_width(
        960.0,
        "Listbox",
        "Single-select list rows. Sizes tab: Sm/Md/Lg label typography beside matching geometry.",
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

fn render_template_body(preview: &ListboxPreview, muted: gpui::Hsla) -> AnyElement {
    size_column("Template", muted, preview.template.clone(), 200.0)
}

fn render_sizes_body(preview: &ListboxPreview, muted: gpui::Hsla) -> AnyElement {
    div()
        .flex()
        .flex_wrap()
        .items_start()
        .gap(px(16.0))
        .child(size_column("Sm", muted, preview.sm.clone(), 180.0))
        .child(size_column("Md", muted, preview.md.clone(), 180.0))
        .child(size_column("Lg", muted, preview.lg.clone(), 180.0))
        .into_any_element()
}

fn size_column(label: &'static str, muted: gpui::Hsla, listbox: ListBox, width: f32) -> AnyElement {
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
        .child(listbox)
        .into_any_element()
}

fn fruit_listbox(
    look: &Arc<ShadcnLook>,
    id: impl Into<SharedString>,
    size: ControlSize,
    cx: &mut impl gpui::AppContext,
) -> ListBox {
    look.listbox(id)
        .template(look.listbox_template_for_size(size))
        .items([
            ListBoxItem::new("apples", "apples").label("Apples"),
            ListBoxItem::new("oranges", "oranges").label("Oranges"),
            ListBoxItem::new("bananas", "bananas").label("Bananas"),
        ])
        .selected_ids(["oranges"])
        .spawn(cx)
}
