use std::sync::Arc;

use gpui::{AnyElement, App, FontWeight, IntoElement, SharedString, Window, div, prelude::*, px};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};

use super::activity::{CyclingDynamicsSection, RideActivity, cycling_dynamics_sections};

pub fn render_cycling_dynamics_tab(
    ride: &RideActivity,
    look: &Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let section_title_style = look.typography_scale(ShadcnTextSize::Sm);
    let label_style = look.typography_scale(ShadcnTextSize::Xs);
    let value_style = look.typography_scale(ShadcnTextSize::Sm);

    let Some(dynamics) = ride.cycling_dynamics.as_ref() else {
        return look
            .card("graph-viz-cycling-dynamics-empty")
            .title("Cycling Dynamics")
            .description("No cycling dynamics data found in this activity.")
            .elevated(false)
            .render(window, cx)
            .into_any_element();
    };

    let sections = cycling_dynamics_sections(dynamics, &ride.stats);
    if sections.is_empty() {
        return look
            .card("graph-viz-cycling-dynamics-empty")
            .title("Cycling Dynamics")
            .description("No cycling dynamics data found in this activity.")
            .elevated(false)
            .render(window, cx)
            .into_any_element();
    }

    let section_elements: Vec<_> = sections
        .iter()
        .map(|section| {
            render_section(section, section_title_style, label_style, value_style, chrome.title_text, chrome.muted_text)
        })
        .collect();

    div()
        .id("graph-viz-cycling-dynamics-tab")
        .w_full()
        .flex()
        .flex_col()
        .gap(px(20.0))
        .children(section_elements)
        .into_any_element()
}

fn render_section(
    section: &CyclingDynamicsSection,
    section_title_style: gpui_luma::theme::LumaTextStyle,
    label_style: gpui_luma::theme::LumaTextStyle,
    value_style: gpui_luma::theme::LumaTextStyle,
    title_color: gpui::Hsla,
    label_color: gpui::Hsla,
) -> AnyElement {
    let rows: Vec<_> = section
        .fields
        .iter()
        .map(|entry| render_row(&entry.label, &entry.value, label_style, value_style, label_color, title_color))
        .collect();

    div()
        .id(SharedString::from(format!("graph-viz-cycling-dynamics-{}", section.title)))
        .w_full()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(
            div()
                .typography_style(section_title_style)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(title_color)
                .child(section.title),
        )
        .child(div().w_full().flex().flex_col().gap(px(6.0)).children(rows))
        .into_any_element()
}

fn render_row(
    label: &str,
    value: &str,
    label_style: gpui_luma::theme::LumaTextStyle,
    value_style: gpui_luma::theme::LumaTextStyle,
    label_color: gpui::Hsla,
    value_color: gpui::Hsla,
) -> AnyElement {
    div()
        .id(SharedString::from(format!("graph-viz-cycling-dynamics-{label}")))
        .w_full()
        .flex()
        .flex_row()
        .items_start()
        .justify_between()
        .gap(px(16.0))
        .child(
            div()
                .flex_shrink_0()
                .w(px(220.0))
                .typography_style(label_style)
                .text_color(label_color)
                .child(label.to_string()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .typography_style(value_style)
                .text_color(value_color)
                .child(value.to_string()),
        )
        .into_any_element()
}
