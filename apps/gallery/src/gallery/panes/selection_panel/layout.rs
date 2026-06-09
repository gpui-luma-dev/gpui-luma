use gpui::{AnyElement, FontWeight, div, hsla, prelude::*, px};
use gpui_luma_look_shadcn::ShadcnLook;

pub(super) const PAGE_SPEC: PageSpec = PageSpec {
    title: "Selection Panel",
    description: "Template preview + spawnable SDK control with live SelectionPanelEvent stream.",
    header: HeaderSpec {
        title: "SelectionPanel composition",
        description: "Template states, custom item rendering, keyboard flow, and theme tokens in one live surface.",
        metrics: &[
            MetricSpec { label: "3 templates" },
            MetricSpec { label: "2 live panels" },
            MetricSpec { label: "5 theme parts" },
        ],
    },
    sections: &[
        Section {
            title: "Template preview",
            description: "Default chrome across standard, active, and disabled rows.",
        },
        Section {
            title: "Interactive panels",
            description: "Both controls remain spawnable SDK entities and emit the shared event stream.",
        },
        Section {
            title: "Event stream",
            description: "Hover, click, Arrow keys, Home/End, and Enter emit SelectionPanelEvent updates.",
        },
    ],
};

#[derive(Clone, Copy)]
pub(super) struct PageSpec {
    pub(super) title: &'static str,
    pub(super) description: &'static str,
    pub(super) header: HeaderSpec,
    pub(super) sections: &'static [Section],
}

#[derive(Clone, Copy)]
pub(super) struct HeaderSpec {
    title: &'static str,
    description: &'static str,
    metrics: &'static [MetricSpec],
}

#[derive(Clone, Copy)]
struct MetricSpec {
    label: &'static str,
}

#[derive(Clone, Copy)]
pub(super) struct Section {
    title: &'static str,
    description: &'static str,
}

pub(super) fn render_page_header(header: HeaderSpec, look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();

    div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(16.0))
        .rounded(px(8.0))
        .border_1()
        .border_color(chrome.border)
        .bg(chrome.panel_background)
        .p(px(14.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .text_size(px(15.0))
                        .line_height(px(20.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(chrome.title_text)
                        .child(header.title),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(17.0))
                        .text_color(chrome.muted_text)
                        .child(header.description),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .children(header.metrics.iter().copied().map(|metric| render_metric_chip(metric, look))),
        )
        .into_any_element()
}

pub(super) fn render_section(section: Section, content: AnyElement, look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .rounded(px(8.0))
        .border_1()
        .border_color(chrome.border)
        .bg(chrome.panel_background)
        .p(px(14.0))
        .child(render_section_header(section, look))
        .child(content)
        .into_any_element()
}

pub(super) fn render_live_panel_sample(
    label: &'static str,
    panel: AnyElement,
    label_color: gpui::Hsla,
    border_color: gpui::Hsla,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(9.0))
        .rounded(px(8.0))
        .border_1()
        .border_color(border_color)
        .bg(hsla(0.0, 0.0, 1.0, 0.035))
        .p(px(10.0))
        .child(panel)
        .child(
            div()
                .text_size(px(11.0))
                .line_height(px(15.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(label_color)
                .child(label),
        )
        .into_any_element()
}

fn render_section_header(section: Section, look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();

    div()
        .flex()
        .flex_col()
        .items_start()
        .gap(px(3.0))
        .child(
            div()
                .text_size(px(13.0))
                .line_height(px(18.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(chrome.title_text)
                .child(section.title),
        )
        .child(
            div()
                .text_size(px(11.0))
                .line_height(px(16.0))
                .text_color(chrome.muted_text)
                .child(section.description),
        )
        .into_any_element()
}

fn render_metric_chip(metric: MetricSpec, look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();

    div()
        .px(px(8.0))
        .py(px(4.0))
        .rounded(px(999.0))
        .border_1()
        .border_color(chrome.border)
        .bg(hsla(0.60, 0.70, 0.52, 0.08))
        .text_size(px(10.0))
        .line_height(px(14.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(chrome.muted_text)
        .child(metric.label)
        .into_any_element()
}
